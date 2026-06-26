// KL Language Support — LSP client + commands
const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client = null;

function activate(context) {
    console.log('KL Language Support activating...');

    context.subscriptions.push(
        vscode.commands.registerCommand('kl.run', () => runFile('run')),
        vscode.commands.registerCommand('kl.build', () => runFile('build')),
        vscode.commands.registerCommand('kl.check', () => runFile('check'))
    );

    const config = vscode.workspace.getConfiguration('kl');
    const klcPath = config.get('klcPath') || 'klc';

    try {
        startLanguageClient(context, klcPath);
    } catch (err) {
        console.error('Failed to start KL language server:', err);
        vscode.window.showWarningMessage(
            'KL language server not available. Install klc or set "kl.klcPath".'
        );
    }
}

function startLanguageClient(context, klcPath) {
    const serverOptions = {
        command: klcPath,
        args: ['lsp'],
    };

    const clientOptions = {
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

    client = lspClient.start();
    context.subscriptions.push(client);
}

async function runFile(subcommand) {
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
    const klcPath = config.get('klcPath') || 'klc';
    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(klcPath + ' ' + subcommand + ' "' + filePath + '"');
}

function deactivate() {
    if (client) {
        client.then(function(c) { c.stop(); });
    }
}

module.exports = { activate, deactivate };
