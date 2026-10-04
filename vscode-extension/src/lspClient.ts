import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function findCliBinary(): string | null {
  const isTrusted = vscode.workspace.isTrusted;
  const config = vscode.workspace.getConfiguration('proofer');
  const inspect = config.inspect<string>('executablePath');
  // Security: Only accept workspace-level executable override if the workspace is explicitly trusted
  const customPath = isTrusted ? config.get<string>('executablePath') : inspect?.globalValue;
  if (customPath && fs.existsSync(customPath)) {
    return customPath;
  }

  // Security: Only search workspace folders if the workspace is explicitly trusted by the user
  if (isTrusted && vscode.workspace.workspaceFolders) {
    for (const folder of vscode.workspace.workspaceFolders) {
      const releasePath = path.join(folder.uri.fsPath, 'proof', 'target', 'release', 'proof');
      if (fs.existsSync(releasePath)) return releasePath;

      const debugPath = path.join(folder.uri.fsPath, 'proof', 'target', 'debug', 'proof');
      if (fs.existsSync(debugPath)) return debugPath;

      const rootRelease = path.join(folder.uri.fsPath, 'target', 'release', 'proof');
      if (fs.existsSync(rootRelease)) return rootRelease;
    }
  }

  // Check system PATH
  const isWin = process.platform === 'win32';
  const binName = isWin ? 'proof.exe' : 'proof';
  const pathEnv = process.env.PATH || '';
  for (const dir of pathEnv.split(path.delimiter)) {
    const candidate = path.join(dir, binName);
    if (fs.existsSync(candidate)) return candidate;
  }

  return null;
}

export function startLspClient(
  _context: vscode.ExtensionContext,
  onWorkstationUpdate: (uri: string, verification: any) => void
): LanguageClient | null {
  const command = findCliBinary();
  if (!command) {
    return null;
  }

  const serverOptions: ServerOptions = {
    command,
    args: ['lsp'],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'proof' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.proof'),
    },
  };

  client = new LanguageClient(
    'prooferLsp',
    'Proofer Language Server',
    serverOptions,
    clientOptions
  );

  client.start().then(() => {
    client?.onNotification('proofer/workstationUpdate', (params: { uri: string; verification: any }) => {
      onWorkstationUpdate(params.uri, params.verification);
    });
  });

  return client;
}

export function stopLspClient(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
