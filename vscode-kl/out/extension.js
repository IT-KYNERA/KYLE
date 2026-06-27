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
let client = null;
function findKlBinary() {
    // 1. Check explicit setting
    const config = vscode.workspace.getConfiguration('kl');
    const configured = config.get('klcPath');
    if (configured && configured !== 'kl') {
        if (fs.existsSync(configured))
            return configured;
    }
    // 2. Check PATH (works when launched from terminal)
    const envPath = process.env.PATH || '';
    const dirs = envPath.split(path.delimiter);
    for (const dir of dirs) {
        const candidate = path.join(dir, 'kl');
        if (fs.existsSync(candidate))
            return candidate;
    }
    // 3. Common install locations
    const home = process.env.HOME || '';
    const locations = [
        path.join(home, '.kl', 'bin', 'kl'),
        path.join(home, '.cargo', 'bin', 'kl'),
        '/usr/local/bin/kl',
        '/opt/homebrew/bin/kl',
        '/usr/bin/kl',
    ];
    for (const loc of locations) {
        if (fs.existsSync(loc))
            return loc;
    }
    // 4. Try `which kl` as last resort
    try {
        const which = require('child_process').execSync('which kl', { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
        if (which && fs.existsSync(which))
            return which;
    }
    catch (_) { }
    // 5. Not found — return null (caller handles gracefully)
    return null;
}
function activate(context) {
    console.log('KL Language Support activating...');
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('kl.run', () => runFile('run')), vscode.commands.registerCommand('kl.build', () => runFile('build')), vscode.commands.registerCommand('kl.check', () => runFile('check')));
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
        }
        catch (err) {
            console.error('Failed to start KL language server:', err);
            vscode.window.showWarningMessage('KL language server could not start. Check that kl is installed and try again.');
        }
    }
    else {
        console.warn('kl binary not found in PATH or common install locations');
        vscode.window.showWarningMessage('KL language server not available. Install kl or set "kl.klcPath" in settings.');
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
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.kl'),
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
function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}
//# sourceMappingURL=extension.js.map