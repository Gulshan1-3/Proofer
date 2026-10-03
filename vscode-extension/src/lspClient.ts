import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function findCliBinary(): string {
  const config = vscode.workspace.getConfiguration('proofer');
  const customPath = config.get<string>('executablePath');
  if (customPath && fs.existsSync(customPath)) {
    return customPath;
  }

  if (vscode.workspace.workspaceFolders) {
    for (const folder of vscode.workspace.workspaceFolders) {
      const releasePath = path.join(folder.uri.fsPath, 'proof', 'target', 'release', 'proof');
      if (fs.existsSync(releasePath)) return releasePath;

      const debugPath = path.join(folder.uri.fsPath, 'proof', 'target', 'debug', 'proof');
      if (fs.existsSync(debugPath)) return debugPath;

      const rootRelease = path.join(folder.uri.fsPath, 'target', 'release', 'proof');
      if (fs.existsSync(rootRelease)) return rootRelease;
    }
  }

  return '/home/gulshansharma/Proofer/proof/target/release/proof';
}

export function startLspClient(
  _context: vscode.ExtensionContext,
  onWorkstationUpdate: (uri: string, verification: any) => void
): LanguageClient {
  const command = findCliBinary();

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
