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
        for (const name of ['kl', 'klc']) {
            const candidate = path.join(dir, name);
            if (fs.existsSync(candidate)) return candidate;
        }
    }

    // 3. Common install locations (search kl + klc)
    const home = process.env.HOME || '';
    const locations = [
        path.join(home, '.kl', 'bin', 'kl'),
        path.join(home, '.kl', 'bin', 'klc'),
        path.join(home, '.cargo', 'bin', 'kl'),
        path.join(home, '.cargo', 'bin', 'klc'),
        '/usr/local/bin/kl',
        '/usr/local/bin/klc',
        '/opt/homebrew/bin/kl',
        '/opt/homebrew/bin/klc',
        '/usr/bin/kl',
    ];
    for (const loc of locations) {
        if (fs.existsSync(loc)) return loc;
    }

    // 4. Try `which kl` as last resort
    try {
        const which = require('child_process').execSync('which kl', { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
        if (which && fs.existsSync(which)) return which;
    } catch (_) {}

    // 5. Not found — return null (caller handles gracefully)
    return null;
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
    let klPath = findKlBinary();
    if (!klPath) {
        // Fallback: try bare "kl" command name (resolved by system PATH)
        klPath = 'kl';
    }
    if (fs.existsSync(klPath) || klPath === 'kl') {
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

    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'kl'; // let shell resolve via PATH
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