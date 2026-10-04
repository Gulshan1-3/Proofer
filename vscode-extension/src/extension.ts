import * as vscode from 'vscode';
import { KernelBridge, VerificationResponse } from './kernelBridge';
import { WorkstationWebviewPanel } from './workstationWebview';
import { ProoferCustomEditorProvider } from './customEditor';
import { startLspClient, stopLspClient } from './lspClient';

export function activate(context: vscode.ExtensionContext) {
  const kernel = new KernelBridge();
  const diagnosticCollection = vscode.languages.createDiagnosticCollection('proofer');
  context.subscriptions.push(diagnosticCollection);

  // Start official LSP Client
  startLspClient(context, (uri, verification) => {
    updateStatusBar(verification);
    const activeEditor = vscode.window.activeTextEditor;
    if (activeEditor && activeEditor.document.uri.toString() === uri) {
      const activeLine = activeEditor.selection.active.line + 1;
      WorkstationWebviewPanel.currentPanel?.updateState(activeEditor.document.getText(), verification, activeLine);
    }
  });

  // Status Bar Indicator
  const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.command = 'proofer.openWorkstation';
  context.subscriptions.push(statusBar);

  // Register Custom Editor
  context.subscriptions.push(ProoferCustomEditorProvider.register(context, kernel));

  let debounceTimer: NodeJS.Timeout | null = null;

  async function verifyDocument(document: vscode.TextDocument) {
    if (document.languageId !== 'proof' && !document.fileName.endsWith('.proof')) {
      return;
    }

    const code = document.getText();
    statusBar.text = '$(sync~spin) Proofer: Verifying...';
    statusBar.show();

    const result = await kernel.verifyProof(code);
    updateDiagnostics(document, result);
    updateStatusBar(result);

    const activeEditor = vscode.window.activeTextEditor;
    const activeLine = (activeEditor && activeEditor.document.uri.toString() === document.uri.toString())
      ? activeEditor.selection.active.line + 1
      : 1;

    if (WorkstationWebviewPanel.currentPanel) {
      WorkstationWebviewPanel.currentPanel.updateState(code, result, activeLine);
    }
  }

  function updateDiagnostics(document: vscode.TextDocument, res: VerificationResponse) {
    diagnosticCollection.delete(document.uri);
    const diagnostics: vscode.Diagnostic[] = [];

    if (!res.verified) {
      const errList = res.errors || [];
      if (res.theorems) {
        for (const thm of res.theorems) {
          errList.push(...thm.errors);
        }
      }

      for (const err of errList) {
        // Find line number in error message if present (e.g. "Line 4:" or search text)
        let targetLine = 0;
        const lineMatch = err.match(/line\s*([0-9]+)/i);
        if (lineMatch) {
          targetLine = Math.max(0, parseInt(lineMatch[1], 10) - 1);
        } else {
          // If no line number, place on the last non-empty line or line 0
          targetLine = Math.max(0, document.lineCount - 1);
        }

        const lineRange = document.lineAt(Math.min(targetLine, document.lineCount - 1)).range;
        const diag = new vscode.Diagnostic(
          lineRange,
          `Proofer Kernel: ${err}`,
          vscode.DiagnosticSeverity.Error
        );
        diag.source = 'Proofer';
        diagnostics.push(diag);
      }
    }

    diagnosticCollection.set(document.uri, diagnostics);
  }

  function updateStatusBar(res: VerificationResponse) {
    if (res.verified) {
      const stepCount = (res.theorems && res.theorems[0] && res.theorems[0].steps.length) || 0;
      statusBar.text = `$(pass-filled) Proofer: Verified (${stepCount}/${stepCount} steps)`;
      statusBar.tooltip = 'Proofer formal verification succeeded. Theorem discharged.';
      statusBar.color = '#10b981';
    } else {
      statusBar.text = '$(error) Proofer: Rejected';
      statusBar.tooltip = 'Proofer kernel rejected one or more proof steps.';
      statusBar.color = '#ef4444';
    }
    statusBar.show();
  }

  // Document change events
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument(e => {
      const config = vscode.workspace.getConfiguration('proofer');
      if (config.get<boolean>('verifyOnChange', true)) {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          verifyDocument(e.document);
        }, 200);
      }
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument(doc => {
      const config = vscode.workspace.getConfiguration('proofer');
      if (config.get<boolean>('verifyOnSave', true)) {
        verifyDocument(doc);
      }
    })
  );

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(editor => {
      if (editor && (editor.document.languageId === 'proof' || editor.document.fileName.endsWith('.proof'))) {
        verifyDocument(editor.document);
      } else {
        statusBar.hide();
      }
    })
  );

  context.subscriptions.push(
    vscode.window.onDidChangeTextEditorSelection(e => {
      if (e.textEditor.document.languageId === 'proof' || e.textEditor.document.fileName.endsWith('.proof')) {
        const activeLine = e.selections[0].active.line + 1;
        if (WorkstationWebviewPanel.currentPanel) {
          const code = e.textEditor.document.getText();
          kernel.verifyProof(code).then(res => {
            WorkstationWebviewPanel.currentPanel?.updateState(code, res, activeLine);
          });
        }
      }
    })
  );

  // Commands
  context.subscriptions.push(
    vscode.commands.registerCommand('proofer.openWorkstation', () => {
      const panel = WorkstationWebviewPanel.createOrShow(context.extensionUri);
      const editor = vscode.window.activeTextEditor;
      if (editor) {
        verifyDocument(editor.document);
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('proofer.verifyProof', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showInformationMessage('No active Proofer file to verify.');
        return;
      }
      await verifyDocument(editor.document);
      vscode.window.showInformationMessage('Proofer verification completed.');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('proofer.synthesizeStep', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showInformationMessage('No active editor open to synthesize proof step.');
        return;
      }
      const document = editor.document;
      const code = document.getText();

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: 'Proofer Co-Prover',
          cancellable: false,
        },
        async (progress) => {
          progress.report({ message: 'Synthesizing next kernel-verified step...' });
          try {
            const res = await kernel.synthesizeStep(code);
            if (res && res.step && res.step.verified_by_kernel) {
              const text = document.getText();
              const endIdx = text.lastIndexOf('end');
              await editor.edit(editBuilder => {
                if (endIdx !== -1) {
                  const pos = document.positionAt(endIdx);
                  editBuilder.insert(pos, `    ${res.step.text}\n`);
                } else {
                  const lastLine = document.lineCount - 1;
                  editBuilder.insert(new vscode.Position(lastLine, 0), `    ${res.step.text}\n`);
                }
              });
              vscode.window.showInformationMessage(`Infilled Step: "${res.step.text}" (Kernel Verified)`);
              await verifyDocument(editor.document);
            } else {
              vscode.window.showWarningMessage('No sound step could be kernel-verified for this proof context.');
            }
          } catch (err: any) {
            vscode.window.showErrorMessage(`Synthesis failed: ${err.message || err}`);
          }
        }
      );
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('proofer.buildPackage', async () => {
      const folders = vscode.workspace.workspaceFolders;
      if (!folders || folders.length === 0) {
        vscode.window.showInformationMessage('No open workspace to build.');
        return;
      }
      const rootPath = folders[0].uri.fsPath;
      // Sanitize rootPath for shell execution to prevent command injection
      const sanitizedPath = rootPath.replace(/'/g, "'\\''");
      const terminal = vscode.window.createTerminal('Proofer Build');
      terminal.show();
      terminal.sendText(`proof build '${sanitizedPath}'`);
    })
  );

  // If there's an active editor on launch
  if (vscode.window.activeTextEditor) {
    verifyDocument(vscode.window.activeTextEditor.document);
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (WorkstationWebviewPanel.currentPanel) {
    WorkstationWebviewPanel.currentPanel.dispose();
  }
  return stopLspClient();
}
