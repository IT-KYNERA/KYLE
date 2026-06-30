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
exports.KyleTestController = void 0;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const child_process_1 = require("child_process");
function findKlBinary() {
    const config = vscode.workspace.getConfiguration('kl');
    const configured = config.get('klcPath');
    if (configured && configured !== 'kl') {
        if (fs.existsSync(configured))
            return configured;
    }
    return 'kl';
}
const testFnRegex = /#\[test\]\s*\n\s*fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;
class KyleTestController {
    constructor() {
        this.watchDisposable = null;
        this.fileTestMap = new Map();
        this.controller = vscode.tests.createTestController('klTests', 'KL Tests');
        this.controller.resolveHandler = (item) => {
            if (!item) {
                this.discoverAllTests();
            }
        };
        this.controller.createRunProfile('Run Tests', vscode.TestRunProfileKind.Run, (request, token) => this.runHandler(request, token), true);
        this.controller.createRunProfile('Debug Test', vscode.TestRunProfileKind.Debug, (request, token) => this.runHandler(request, token), false);
        this.watchFiles();
    }
    watchFiles() {
        if (this.watchDisposable) {
            this.watchDisposable.dispose();
        }
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.kl');
        watcher.onDidCreate((uri) => this.onFileChange(uri));
        watcher.onDidChange((uri) => this.onFileChange(uri));
        watcher.onDidDelete((uri) => this.onFileDelete(uri));
        this.watchDisposable = watcher;
        this.discoverAllTests();
    }
    async onFileChange(uri) {
        if (!uri.fsPath.endsWith('.kl'))
            return;
        this.discoverTestsInFile(uri);
    }
    async onFileDelete(uri) {
        if (!uri.fsPath.endsWith('.kl'))
            return;
        this.fileTestMap.delete(uri.fsPath);
        this.controller.items.delete(uri.toString());
    }
    async discoverAllTests() {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders)
            return;
        const klFiles = await vscode.workspace.findFiles('**/*.kl', '**/node_modules/**');
        for (const file of klFiles) {
            await this.discoverTestsInFile(file);
        }
    }
    async discoverTestsInFile(uri) {
        try {
            const content = fs.readFileSync(uri.fsPath, 'utf-8');
            const testNames = this.findTestFunctions(content);
            const existing = this.fileTestMap.get(uri.fsPath) || new Set();
            const testIds = new Set();
            let fileItem = this.controller.items.get(uri.toString());
            if (testNames.size === 0) {
                this.fileTestMap.delete(uri.fsPath);
                if (fileItem) {
                    this.controller.items.delete(uri.toString());
                }
                return;
            }
            if (!fileItem) {
                fileItem = this.controller.createTestItem(uri.toString(), path.basename(uri.fsPath), uri);
                fileItem.canResolveChildren = true;
                this.controller.items.add(fileItem);
            }
            const children = new Map();
            for (const testName of testNames) {
                const testId = `${uri.toString()}::${testName}`;
                testIds.add(testId);
                const testItem = this.controller.createTestItem(testId, testName, uri);
                testItem.tags = [new vscode.TestTag('kl:test')];
                children.set(testId, testItem);
            }
            fileItem.children.replace([...children.values()]);
            const updated = new Set(testNames);
            this.fileTestMap.set(uri.fsPath, updated);
        }
        catch (err) {
            console.error(`Failed to discover tests in ${uri.fsPath}:`, err);
        }
    }
    findTestFunctions(content) {
        const names = new Set();
        let match;
        const re = new RegExp(testFnRegex);
        while ((match = re.exec(content)) !== null) {
            names.add(match[1]);
        }
        return names;
    }
    async runHandler(request, token) {
        const run = this.controller.createTestRun(request);
        const queue = [];
        if (request.include) {
            request.include.forEach((item) => queue.push(item));
        }
        else {
            this.controller.items.forEach((item) => queue.push(item));
        }
        for (const item of queue) {
            if (token.isCancellationRequested) {
                break;
            }
            if (item.children.size > 0) {
                item.children.forEach((child) => queue.push(child));
                continue;
            }
            await this.runSingleTest(item, run, token);
        }
        run.end();
    }
    async runSingleTest(item, run, token) {
        run.started(item);
        const uri = item.uri;
        if (!uri) {
            run.failed(item, new vscode.TestMessage('No file URI'));
            return;
        }
        const filePath = uri.fsPath;
        if (!fs.existsSync(filePath)) {
            run.failed(item, new vscode.TestMessage(`File not found: ${filePath}`));
            return;
        }
        const testName = item.label;
        const source = fs.readFileSync(filePath, 'utf-8');
        const tempDir = path.join(path.dirname(filePath), '.kl-test');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }
        const sanitizedName = testName.replace(/[^a-zA-Z0-9_]/g, '_');
        const wrapperSource = `${source}\nfn main() i32:\n    ${testName}()\n    println("PASS")\n    0\n`;
        const wrapperFile = path.join(tempDir, `${sanitizedName}_test.kl`);
        fs.writeFileSync(wrapperFile, wrapperSource);
        const outputFile = path.join(tempDir, sanitizedName);
        const klPath = findKlBinary();
        try {
            const buildResult = (0, child_process_1.execSync)(`"${klPath}" build "${wrapperFile}" -o "${outputFile}"`, { encoding: 'utf-8', timeout: 30000, stdio: ['pipe', 'pipe', 'pipe'] });
            console.log('Build output:', buildResult);
        }
        catch (buildErr) {
            const msg = buildErr.stderr || buildErr.stdout || buildErr.message || 'Build failed';
            run.failed(item, new vscode.TestMessage(`Build error:\n${msg}`));
            this.cleanup(tempDir, sanitizedName, wrapperFile);
            return;
        }
        if (token.isCancellationRequested) {
            run.skipped(item);
            this.cleanup(tempDir, sanitizedName, wrapperFile);
            return;
        }
        return new Promise((resolve) => {
            const runProc = (0, child_process_1.spawn)(outputFile, [], {
                stdio: ['pipe', 'pipe', 'pipe'],
                timeout: 30000,
            });
            let stdout = '';
            let stderr = '';
            runProc.stdout.on('data', (data) => {
                stdout += data.toString();
            });
            runProc.stderr.on('data', (data) => {
                stderr += data.toString();
            });
            runProc.on('close', (code) => {
                if (code === 0) {
                    run.passed(item, stdout ? undefined : undefined);
                }
                else if (stderr.includes('KL ASSERT FAILED') || stderr.includes('KL PANIC')) {
                    const msg = stderr.replace('KL ASSERT FAILED: ', '').replace('KL PANIC: ', '').trim();
                    run.failed(item, new vscode.TestMessage(msg || `Test failed with exit code ${code}`));
                }
                else if (stderr) {
                    run.failed(item, new vscode.TestMessage(stderr.trim()));
                }
                else {
                    run.failed(item, new vscode.TestMessage(`Test failed with exit code ${code}`));
                }
                if (stdout.trim()) {
                    run.appendOutput(`${stdout}\r\n`);
                }
                this.cleanup(tempDir, sanitizedName, wrapperFile);
                resolve();
            });
            runProc.on('error', (err) => {
                run.errored(item, new vscode.TestMessage(err.message));
                this.cleanup(tempDir, sanitizedName, wrapperFile);
                resolve();
            });
        });
    }
    cleanup(tempDir, name, wrapperFile) {
        try {
            if (fs.existsSync(wrapperFile))
                fs.unlinkSync(wrapperFile);
            const outputFile = path.join(tempDir, name);
            if (fs.existsSync(outputFile))
                fs.unlinkSync(outputFile);
            if (fs.existsSync(tempDir)) {
                const remaining = fs.readdirSync(tempDir);
                if (remaining.length === 0)
                    fs.rmdirSync(tempDir);
            }
        }
        catch (_) { }
    }
    dispose() {
        if (this.watchDisposable) {
            this.watchDisposable.dispose();
        }
        this.controller.dispose();
    }
}
exports.KyleTestController = KyleTestController;
//# sourceMappingURL=testUI.js.map