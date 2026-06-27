import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function activate(context: vscode.ExtensionContext) {
    console.log('KL Language Support activating...');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('kl.run', () => runFile('run')),
        vscode.commands.registerCommand('kl.build', () => runFile('build')),
        vscode.commands.registerCommand('kl.check', () => runFile('check'))
    );

    // Start LSP client if kl binary is available
    const config = vscode.workspace.getConfiguration('kl');
    const klcPath = config.get<string>('klcPath') || 'kl';

    try {
        startLanguageClient(context, klcPath);
    } catch (err) {
        console.error('Failed to start KL language server:', err);
        vscode.window.showWarningMessage(
            'KL language server not available. Install kl or set "kl.klcPath" in settings.'
        );
    }
}

function startLanguageClient(context: vscode.ExtensionContext, klcPath: string) {
    const serverOptions: ServerOptions = {
        command: klcPath,
        args: ['lsp'],
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'kl' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.kl'),
        },
    };

    const lspClient = new LanguageClient(
        'klLanguageServer',
        'KL Language Server',
        serverOptions,
        clientOptions
    );

    client = lspClient;
    lspClient.start();
    context.subscriptions.push({ dispose: () => lspClient.stop() });
}

async function runFile(subcommand: string) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const filePath = editor.document.uri.fsPath;
    if (!filePath.endsWith('.kl')) {
        vscode.window.showErrorMessage('Not a KL file');
        return;
    }

    const config = vscode.workspace.getConfiguration('kl');
    const klcPath = config.get<string>('klcPath') || 'kl';
    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(`${klcPath} ${subcommand} "${filePath}"`);
}

export function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}