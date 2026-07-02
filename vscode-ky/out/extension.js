"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const node_1 = require("vscode-languageclient/node");
const tasks_1 = require("./tasks");
const testUI_1 = require("./testUI");
let client = null;
let testController = null;
function findKlBinary() {
    const config = vscode.workspace.getConfiguration('kl');
    const configured = config.get('klcPath');
    if (configured && configured !== 'kl') {
        if (fs.existsSync(configured))
            return configured;
    }
    const envPath = process.env.PATH || '';
    const dirs = envPath.split(path.delimiter);
    for (const dir of dirs) {
        for (const name of ['kl', 'ky']) {
            const candidate = path.join(dir, name);
            if (fs.existsSync(candidate))
                return candidate;
        }
    }
    const home = process.env.HOME || '';
    const locations = [
        path.join(home, '.ky', 'bin', 'kl'),
        path.join(home, '.ky', 'bin', 'ky'),
        path.join(home, '.cargo', 'bin', 'kl'),
        path.join(home, '.cargo', 'bin', 'ky'),
        '/usr/local/bin/kl',
        '/usr/local/bin/ky',
        '/opt/homebrew/bin/kl',
        '/opt/homebrew/bin/ky',
        '/usr/bin/kl',
    ];
    for (const loc of locations) {
        if (fs.existsSync(loc))
            return loc;
    }
    try {
        const which = require('child_process').execSync('which kl', { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
        if (which && fs.existsSync(which))
            return which;
    }
    catch (_) { }
    return null;
}
function activate(context) {
    console.log('KL Language Support activating...');
    // Register test controller (Testing UI)
    testController = new testUI_1.KyleTestController();
    context.subscriptions.push({ dispose: () => testController?.dispose() });
    // Register task provider
    context.subscriptions.push(vscode.tasks.registerTaskProvider('kl', new tasks_1.KyleTaskProvider()));
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('ky.run', () => runFile('run')), vscode.commands.registerCommand('ky.build', () => runFile('build')), vscode.commands.registerCommand('ky.check', () => runFile('check')), vscode.commands.registerCommand('ky.test', () => runFile('test')), vscode.commands.registerCommand('ky.runTest', (fileUri, testName) => {
        runSpecificTest(fileUri, testName);
    }));
    // Register diagnostics collection
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('kl');
    context.subscriptions.push(diagnosticCollection);
    // Parse output for diagnostics
    context.subscriptions.push(vscode.commands.registerCommand('ky.handleOutput', (output) => {
        parseAndSetDiagnostics(output, diagnosticCollection);
    }));
    // Start LSP client
    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'kl';
    }
    if (fs.existsSync(klPath) || klPath === 'kl') {
        try {
            startLanguageClient(context, klPath);
            console.log('KY language server started:', klPath);
        }
        catch (err) {
            console.error('Failed to start KL language server:', err);
            vscode.window.showWarningMessage('KY language server could not start. Check that kl is installed and try again.');
        }
    }
    else {
        console.warn('kl binary not found in PATH or common install locations');
        vscode.window.showWarningMessage('KY language server not available. Install kl or set "ky.klcPath" in settings.');
    }
}
function startLanguageClient(context, klPath) {
    const serverOptions = {
        command: klPath,
        args: ['lsp'],
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'kl' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{*.ky,ky.toml}'),
        },
    };
    const lspClient = new node_1.LanguageClient('klLanguageServer', 'KL Language Server', serverOptions, clientOptions);
    client = lspClient;
    lspClient.start();
    context.subscriptions.push({ dispose: () => lspClient.stop() });
}
async function runFile(subcommand) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }
    const filePath = editor.document.uri.fsPath;
    if (!filePath.endsWith('.ky')) {
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
function runSpecificTest(fileUri, testName) {
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
function parseAndSetDiagnostics(output, collection) {
    collection.clear();
    const diagnosticRegex = /^(.+?):(\d+):(\d+):\s*(error|warning)\[([^\]]+)\]:\s*(.+)$/gm;
    let match;
    const diagnosticsByFile = new Map();
    while ((match = diagnosticRegex.exec(output)) !== null) {
        const [, file, lineStr, colStr, severity, code, message] = match;
        const line = parseInt(lineStr) - 1;
        const col = parseInt(colStr) - 1;
        const range = new vscode.Range(line, col, line, col + 1);
        const diagnostic = new vscode.Diagnostic(range, `[${code}] ${message}`, severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning);
        diagnostic.code = code;
        diagnostic.source = 'ky';
        const filePath = path.resolve(vscode.workspace.rootPath || '', file);
        const existing = diagnosticsByFile.get(filePath) || [];
        existing.push(diagnostic);
        diagnosticsByFile.set(filePath, existing);
    }
    for (const [file, diags] of diagnosticsByFile) {
        collection.set(vscode.Uri.file(file), diags);
    }
}
function deactivate() {
    if (testController) {
        testController.dispose();
        testController = null;
    }
    if (client) {
        return client.stop();
    }
    return undefined;
}
//# sourceMappingURL=extension.js.map