import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

function findKlBinary(): string | null {
    // 1. Check explicit setting
    const config = vscode.workspace.getConfiguration('kl');
    const configured = config.get<string>('klcPath');
    if (configured && configured !== 'kl') {
        if (fs.existsSync(configured)) return configured;
    }

    // 2. Check PATH (works when launched from terminal)
    const envPath = process.env.PATH || '';
    const dirs = envPath.split(path.delimiter);
    for (const dir of dirs) {
        const candidate = path.join(dir, 'kl');
        if (fs.existsSync(candidate)) return candidate;
    }

    // 3. Common install locations
    const home = process.env.HOME || '';
    const locations = [
        path.join(home, '.kl', 'bin', 'kl'),
        '/usr/local/bin/kl',
        '/opt/homebrew/bin/kl',
        '/usr/bin/kl',
    ];
    for (const loc of locations) {
        if (fs.existsSync(loc)) return loc;
    }

    // 4. Fallback: just try "kl" in PATH via shell
    return 'kl';
}

export function activate(context: vscode.ExtensionContext) {
    console.log('KL Language Support activating...');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('kl.run', () => runFile('run')),
        vscode.commands.registerCommand('kl.build', () => runFile('build')),
        vscode.commands.registerCommand('kl.check', () => runFile('check'))
    );

    // Start LSP client
    const klPath = findKlBinary();
    if (klPath && fs.existsSync(klPath)) {
        try {
            startLanguageClient(context, klPath);
            console.log('KL language server started:', klPath);
        } catch (err) {
            console.error('Failed to start KL language server:', err);
            vscode.window.showWarningMessage(
                'KL language server could not start. Check that kl is installed and try again.'
            );
        }
    } else {
        console.warn('kl binary not found in PATH or common install locations');
        vscode.window.showWarningMessage(
            'KL language server not available. Install kl or set "kl.klcPath" in settings.'
        );
    }
}

function startLanguageClient(context: vscode.ExtensionContext, klPath: string) {
    const serverOptions: ServerOptions = {
        command: klPath,
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

    const klPath = findKlBinary();
    if (!klPath) {
        vscode.window.showErrorMessage('kl binary not found. Install kl or set "kl.klcPath" in settings.');
        return;
    }

    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(`${klPath} ${subcommand} "${filePath}"`);
}

export function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}