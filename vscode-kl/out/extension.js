// KL Language Support — syntax highlighting only (no LSP)
const vscode = require('vscode');
const path = require('path');

function activate(context) {
    console.log('KL Language Support activating (syntax only)...');

    context.subscriptions.push(
        vscode.commands.registerCommand('kl.run', () => runFile('run')),
        vscode.commands.registerCommand('kl.build', () => runFile('build')),
        vscode.commands.registerCommand('kl.check', () => runFile('check'))
    );
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

function deactivate() {}

module.exports = { activate, deactivate };
