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
const node_1 = require("vscode-languageclient/node");
let client = null;
function activate(context) {
    console.log('KL Language Support activating...');
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('kl.run', () => runFile('run')), vscode.commands.registerCommand('kl.build', () => runFile('build')), vscode.commands.registerCommand('kl.check', () => runFile('check')));
    // Start LSP client if kl binary is available
    const config = vscode.workspace.getConfiguration('kl');
    const klcPath = config.get('klcPath') || 'kl';
    try {
        startLanguageClient(context, klcPath);
    }
    catch (err) {
        console.error('Failed to start KL language server:', err);
        vscode.window.showWarningMessage('KL language server not available. Install kl or set "kl.klcPath" in settings.');
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
    const config = vscode.workspace.getConfiguration('kl');
    const klcPath = config.get('klcPath') || 'kl';
    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(`${klcPath} ${subcommand} "${filePath}"`);
}
function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}
//# sourceMappingURL=extension.js.map