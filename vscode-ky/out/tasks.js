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
exports.KyleTaskProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
function findKyBinary() {
    const config = vscode.workspace.getConfiguration('ky');
    const configured = config.get('kycPath');
    if (configured && configured !== 'ky') {
        if (require('fs').existsSync(configured))
            return configured;
    }
    return 'ky';
}
class KyleTaskProvider {
    async provideTasks() {
        const tasks = [];
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders)
            return tasks;
        for (const folder of workspaceFolders) {
            const klFiles = await vscode.workspace.findFiles('**/*.ky', '**/node_modules/**');
            if (klFiles.length === 0)
                continue;
            const uniqueFiles = new Set();
            for (const file of klFiles) {
                const filePath = file.fsPath;
                if (uniqueFiles.has(filePath))
                    continue;
                uniqueFiles.add(filePath);
                const relativePath = path.relative(folder.uri.fsPath, filePath);
                tasks.push(this.makeTask('Run', 'run', filePath, relativePath, folder));
                tasks.push(this.makeTask('Build', 'build', filePath, relativePath, folder));
                tasks.push(this.makeTask('Check', 'check', filePath, relativePath, folder));
            }
        }
        return tasks;
    }
    makeTask(kind, subcommand, filePath, label, folder) {
        const kyPath = findKyBinary() || 'ky';
        const execution = new vscode.ProcessExecution(kyPath, [subcommand, filePath]);
        const task = new vscode.Task({ type: 'ky', subcommand }, folder, `${kind}: ${label}`, 'KL', execution);
        task.group = subcommand === 'run'
            ? vscode.TaskGroup.Build
            : subcommand === 'check'
                ? vscode.TaskGroup.Test
                : vscode.TaskGroup.Rebuild;
        task.problemMatchers = [''];
        return task;
    }
    async resolveTask(task) {
        return task;
    }
}
exports.KyleTaskProvider = KyleTaskProvider;
//# sourceMappingURL=tasks.js.map