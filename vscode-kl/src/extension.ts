import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { KyleTaskProvider } from './tasks';
import { KyleTestController } from './testUI';

let client: LanguageClient | null = null;
let testController: KyleTestController | null = null;

function findKlBinary(): string | null {
    const config = vscode.workspace.getConfiguration('kl');
    const configured = config.get<string>('klcPath');
    if (configured && configured !== 'kl') {
        if (fs.existsSync(configured)) return configured;
    }

    const envPath = process.env.PATH || '';
    const dirs = envPath.split(path.delimiter);
    for (const dir of dirs) {
        for (const name of ['kl', 'klc']) {
            const candidate = path.join(dir, name);
            if (fs.existsSync(candidate)) return candidate;
        }
    }

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

    try {
        const which = require('child_process').execSync('which kl', { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
        if (which && fs.existsSync(which)) return which;
    } catch (_) {}

    return null;
}

export function activate(context: vscode.ExtensionContext) {
    console.log('KL Language Support activating...');

    // Register test controller (Testing UI)
    testController = new KyleTestController();
    context.subscriptions.push({ dispose: () => testController?.dispose() });

    // Register task provider
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider('kl', new KyleTaskProvider())
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('kl.run', () => runFile('run')),
        vscode.commands.registerCommand('kl.build', () => runFile('build')),
        vscode.commands.registerCommand('kl.check', () => runFile('check')),
        vscode.commands.registerCommand('kl.test', () => runFile('test')),
        vscode.commands.registerCommand('kl.runTest', (fileUri: string, testName: string) => {
            runSpecificTest(fileUri, testName);
        })
    );

    // Register diagnostics collection
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('kl');
    context.subscriptions.push(diagnosticCollection);

    // Parse output for diagnostics
    context.subscriptions.push(
        vscode.commands.registerCommand('kl.handleOutput', (output: string) => {
            parseAndSetDiagnostics(output, diagnosticCollection);
        })
    );

    // Start LSP client
    let klPath = findKlBinary();
    if (!klPath) {
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
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{*.kl,kl.toml}'),
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
        klPath = 'kl';
    }

    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(`${klPath} ${subcommand} "${filePath}"`);
}

function runSpecificTest(fileUri: string, testName: string) {
    const filePath = fileUri.replace(/^file:\/\//, '');
    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'kl';
    }

    // Create wrapper to run just this test
    const fs = require('fs');
    const path = require('path');
    const source = fs.readFileSync(filePath, 'utf-8');
    const dir = path.dirname(filePath);
    const ext = path.extname(filePath);

    const tempDir = path.join(dir, '.kl-test');
    if (!fs.existsSync(tempDir)) {
        fs.mkdirSync(tempDir, { recursive: true });
    }

    const wrapperSource = source + `\nfn main() i32:\n    ${testName}()\n    0\n`;
    const wrapperFile = path.join(tempDir, `run_${testName}${ext}`);
    fs.writeFileSync(wrapperFile, wrapperSource);

    const terminal = vscode.window.createTerminal(`KL Test: ${testName}`);
    terminal.show();
    terminal.sendText(`${klPath} run "${wrapperFile}"`);
}

function parseAndSetDiagnostics(output: string, collection: vscode.DiagnosticCollection) {
    collection.clear();

    const diagnosticRegex = /^(.+?):(\d+):(\d+):\s*(error|warning)\[([^\]]+)\]:\s*(.+)$/gm;
    let match: RegExpExecArray | null;

    const diagnosticsByFile = new Map<string, vscode.Diagnostic[]>();

    while ((match = diagnosticRegex.exec(output)) !== null) {
        const [, file, lineStr, colStr, severity, code, message] = match;
        const line = parseInt(lineStr) - 1;
        const col = parseInt(colStr) - 1;
        const range = new vscode.Range(line, col, line, col + 1);
        const diagnostic = new vscode.Diagnostic(
            range,
            `[${code}] ${message}`,
            severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning
        );
        diagnostic.code = code;
        diagnostic.source = 'klc';

        const filePath = path.resolve(vscode.workspace.rootPath || '', file);
        const existing = diagnosticsByFile.get(filePath) || [];
        existing.push(diagnostic);
        diagnosticsByFile.set(filePath, existing);
    }

    for (const [file, diags] of diagnosticsByFile) {
        collection.set(vscode.Uri.file(file), diags);
    }
}

export function deactivate() {
    if (testController) {
        testController.dispose();
        testController = null;
    }
    if (client) {
        return client.stop();
    }
    return undefined;
}
