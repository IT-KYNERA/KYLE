"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.KyleDebugSession = void 0;
const events_1 = require("events");
class KyleDebugSession extends events_1.EventEmitter {
    constructor() {
        super();
        this.seq = 1;
        this.isRunning = false;
        this.breakpoints = new Map();
        this.childProcess = null;
    }
    start() {
        const buffer = [];
        process.stdin.setEncoding('utf8');
        process.stdin.on('data', (chunk) => {
            buffer.push(chunk);
            const content = buffer.join('');
            const parts = content.split('\r\n\r\n');
            while (parts.length >= 2) {
                const headers = parts.shift();
                const bodyStr = parts.join('\r\n\r\n');
                const match = headers.match(/Content-Length:\s*(\d+)/i);
                if (match) {
                    const length = parseInt(match[1], 10);
                    if (bodyStr.length >= length) {
                        const raw = bodyStr.substring(0, length);
                        buffer.length = 0;
                        buffer.push(bodyStr.substring(length));
                        try {
                            const msg = JSON.parse(raw);
                            this.handleMessage(msg);
                        }
                        catch { }
                        break;
                    }
                }
                buffer.length = 0;
                buffer.push(bodyStr);
                break;
            }
        });
    }
    send(body) {
        const json = JSON.stringify(body);
        const header = `Content-Length: ${Buffer.byteLength(json, 'utf8')}\r\n\r\n`;
        process.stdout.write(header + json);
    }
    handleMessage(msg) {
        const cmd = msg.command;
        const args = msg.arguments || {};
        switch (cmd) {
            case 'initialize':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: {
                        supportsConfigurationDoneRequest: true,
                        supportsSetVariable: false,
                        supportsExceptionInfoRequest: false,
                        supportsTerminateRequest: true,
                        supportTerminateDebuggee: true,
                    },
                });
                break;
            case 'launch':
                this.doLaunch(args);
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'setBreakpoints': {
                const file = args.source?.path || '';
                const lines = (args.breakpoints || []).map((bp) => bp.line);
                if (file)
                    this.breakpoints.set(file, lines);
                const bps = lines.map((line) => ({
                    verified: true,
                    line,
                    id: this.seq++,
                }));
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { breakpoints: bps },
                });
                break;
            }
            case 'setFunctionBreakpoints':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { breakpoints: [] },
                });
                break;
            case 'setExceptionBreakpoints':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'configurationDone':
                this.runProgram();
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'continue':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { allThreadsContinued: true },
                });
                break;
            case 'next':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'stepIn':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'stepOut':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                break;
            case 'stackTrace':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: {
                        stackFrames: [
                            {
                                id: 1,
                                name: 'main',
                                line: args.startFrame || 1,
                                column: 1,
                                source: args.source || undefined,
                            },
                        ],
                        totalFrames: 1,
                    },
                });
                break;
            case 'scopes':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { scopes: [] },
                });
                break;
            case 'variables':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { variables: [] },
                });
                break;
            case 'threads':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                    body: { threads: [{ id: 1, name: 'main' }] },
                });
                break;
            case 'evaluate':
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: false,
                    message: 'Evaluation not supported',
                });
                break;
            case 'terminate':
            case 'disconnect':
                this.cleanup();
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: true,
                });
                this.send({ seq: this.seq++, type: 'event', event: 'terminated' });
                break;
            default:
                this.send({
                    seq: this.seq++, type: 'response', request_seq: msg.seq,
                    command: cmd, success: false,
                    message: `Unknown command: ${cmd}`,
                });
        }
    }
    doLaunch(args) {
        const program = args.program || '';
        const klcPath = args.klcPath || 'kl';
        const { execSync } = require('child_process');
        try {
            execSync(`${klcPath} build "${program}"`, { stdio: 'pipe' });
            this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: `Compiled: ${program}\n` } });
        }
        catch (e) {
            this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: `Compile error: ${e.message}\n`, category: 'stderr' } });
        }
    }
    runProgram() {
        const { spawn } = require('child_process');
        const binary = './a.out';
        this.childProcess = spawn(binary, [], { stdio: ['pipe', 'pipe', 'pipe'] });
        this.childProcess.stdout.on('data', (data) => {
            this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: data.toString(), category: 'stdout' } });
        });
        this.childProcess.stderr.on('data', (data) => {
            this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: data.toString(), category: 'stderr' } });
        });
        this.childProcess.on('exit', () => {
            this.send({ seq: this.seq++, type: 'event', event: 'exited', body: { exitCode: 0 } });
            this.send({ seq: this.seq++, type: 'event', event: 'terminated' });
        });
    }
    cleanup() {
        if (this.childProcess) {
            this.childProcess.kill();
            this.childProcess = null;
        }
    }
}
exports.KyleDebugSession = KyleDebugSession;
if (require.main === module) {
    const session = new KyleDebugSession();
    session.start();
}
//# sourceMappingURL=debugger.js.map