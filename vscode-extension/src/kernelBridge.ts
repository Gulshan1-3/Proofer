import * as vscode from 'vscode';
import * as http from 'http';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';
import * as readline from 'readline';

export interface ProofStep {
  id: number;
  text: string;
  status: 'Valid' | 'Rejected' | 'Pending';
  rule?: string;
  conclusion?: string;
}

export interface TheoremResult {
  name: string;
  status: 'Verified' | 'Rejected';
  proven?: string;
  errors: string[];
  steps: ProofStep[];
}

export interface VerificationResponse {
  status: 'Ok' | 'Error';
  verified: boolean;
  theorems?: TheoremResult[];
  errors?: string[];
  geometryScene?: {
    name: string;
    points: Array<{ id: string; x: number; y: number }>;
  };
}

export class KernelBridge {
  private serverUrl: string = 'http://127.0.0.1:8086';
  private stdioProc: ChildProcess | null = null;
  private stdioReader: readline.Interface | null = null;
  private pendingRequests: Array<{ resolve: (v: VerificationResponse) => void; reject: (e: any) => void }> = [];

  constructor() {
    this.updateConfig();
    this.initStdioDaemon();
    vscode.workspace.onDidChangeConfiguration(e => {
      if (e.affectsConfiguration('proofer')) {
        this.updateConfig();
      }
    });
  }

  private initStdioDaemon() {
    const binPath = this.findCliBinary();
    if (!binPath) return;

    try {
      this.stdioProc = spawn(binPath, ['daemon']);
      if (this.stdioProc.stdout && this.stdioProc.stdin) {
        this.stdioReader = readline.createInterface({
          input: this.stdioProc.stdout,
          crlfDelay: Infinity,
        });

        this.stdioReader.on('line', (line) => {
          const cb = this.pendingRequests.shift();
          if (cb) {
            try {
              const res = JSON.parse(line.trim());
              cb.resolve(res);
            } catch (e) {
              cb.reject(e);
            }
          }
        });

        this.stdioProc.on('error', () => {
          this.disposeStdio();
        });

        this.stdioProc.on('exit', () => {
          this.disposeStdio();
        });
      }
    } catch (_) {
      this.disposeStdio();
    }
  }

  private disposeStdio() {
    if (this.stdioReader) {
      this.stdioReader.close();
      this.stdioReader = null;
    }
    if (this.stdioProc) {
      try { this.stdioProc.kill(); } catch (_) {}
      this.stdioProc = null;
    }
    while (this.pendingRequests.length > 0) {
      const cb = this.pendingRequests.shift();
      if (cb) cb.reject(new Error('Stdio daemon terminated'));
    }
  }

  private updateConfig() {
    const config = vscode.workspace.getConfiguration('proofer');
    this.serverUrl = config.get<string>('serverUrl') || 'http://127.0.0.1:8086';
  }

  /**
   * Primary verification entry point:
   * 1. Try persistent stdio daemon (0 ports, instant streaming).
   * 2. Fall back to local daemon HTTP if running.
   * 3. Fall back to direct single-shot CLI invocation.
   */
  public async verifyProof(code: string): Promise<VerificationResponse> {
    if (this.stdioProc && this.stdioProc.stdin && !this.stdioProc.killed) {
      try {
        return await this.verifyViaStdio(code);
      } catch (_) {
        // Fall through
      }
    }

    try {
      return await this.verifyViaHttp(code);
    } catch {
      return await this.verifyViaCli(code);
    }
  }

  private verifyViaStdio(code: string): Promise<VerificationResponse> {
    return new Promise((resolve, reject) => {
      if (!this.stdioProc || !this.stdioProc.stdin) {
        return reject(new Error('Stdio daemon unavailable'));
      }
      const timer = setTimeout(() => {
        const idx = this.pendingRequests.findIndex(p => p.resolve === resolve);
        if (idx !== -1) {
          this.pendingRequests.splice(idx, 1);
          reject(new Error('Stdio daemon timeout'));
        }
      }, 1000);

      this.pendingRequests.push({
        resolve: (val) => {
          clearTimeout(timer);
          resolve(val);
        },
        reject: (err) => {
          clearTimeout(timer);
          reject(err);
        },
      });

      const payload = JSON.stringify({ code }) + '\n';
      this.stdioProc.stdin.write(payload);
    });
  }

  private verifyViaHttp(code: string): Promise<VerificationResponse> {
    return new Promise((resolve, reject) => {
      try {
        const url = new URL(this.serverUrl);
        const postData = JSON.stringify({ code });

        const req = http.request({
          hostname: url.hostname,
          port: url.port || 8086,
          path: '/',
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData),
            'Connection': 'close',
          },
          timeout: 1000,
        }, (res) => {
          let body = '';
          res.on('data', chunk => body += chunk);
          res.on('end', () => {
            try {
              const parsed: VerificationResponse = JSON.parse(body);
              resolve(parsed);
            } catch (err) {
              reject(err);
            }
          });
        });

        req.on('error', reject);
        req.on('timeout', () => {
          req.destroy();
          reject(new Error('HTTP verification timed out'));
        });

        req.write(postData);
        req.end();
      } catch (err) {
        reject(err);
      }
    });
  }

  private findCliBinary(): string | null {
    const isTrusted = vscode.workspace.isTrusted;
    const config = vscode.workspace.getConfiguration('proofer');
    const inspect = config.inspect<string>('executablePath');
    // Security: Only accept workspace-level executable override if the workspace is explicitly trusted
    const customPath = isTrusted ? config.get<string>('executablePath') : inspect?.globalValue;
    if (customPath && fs.existsSync(customPath)) {
      return customPath;
    }

    // Security: Only search workspace folders if the workspace is explicitly trusted by the user
    if (isTrusted && vscode.workspace.workspaceFolders) {
      for (const folder of vscode.workspace.workspaceFolders) {
        const releasePath = path.join(folder.uri.fsPath, 'proof', 'target', 'release', 'proof');
        if (fs.existsSync(releasePath)) return releasePath;

        const debugPath = path.join(folder.uri.fsPath, 'proof', 'target', 'debug', 'proof');
        if (fs.existsSync(debugPath)) return debugPath;

        const rootRelease = path.join(folder.uri.fsPath, 'target', 'release', 'proof');
        if (fs.existsSync(rootRelease)) return rootRelease;
      }
    }

    // Check system PATH
    const isWin = process.platform === 'win32';
    const binName = isWin ? 'proof.exe' : 'proof';
    const pathEnv = process.env.PATH || '';
    for (const dir of pathEnv.split(path.delimiter)) {
      const candidate = path.join(dir, binName);
      if (fs.existsSync(candidate)) return candidate;
    }

    return null;
  }

  private verifyViaCli(code: string): Promise<VerificationResponse> {
    return new Promise((resolve, reject) => {
      const binPath = this.findCliBinary();
      if (!binPath) {
        resolve({
          status: 'Error',
          verified: false,
          errors: ['No Proofer kernel daemon or binary found. Start proofer daemon or configure proofer.executablePath.'],
        });
        return;
      }

      const proc = spawn(binPath, ['verify', '-']);
      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', chunk => stdout += chunk);
      proc.stderr.on('data', chunk => stderr += chunk);

      proc.on('close', (exitCode) => {
        try {
          const parsed = JSON.parse(stdout.trim());
          resolve(parsed);
        } catch (e) {
          resolve({
            status: 'Error',
            verified: false,
            errors: [stderr.trim() || `CLI exited with code ${exitCode}`],
          });
        }
      });

      proc.on('error', (err) => {
        reject(err);
      });

      proc.stdin.write(code);
      proc.stdin.end();
    });
  }

  public async synthesizeStep(code: string): Promise<{ step: any; candidates: any[] }> {
    // 1. Try HTTP server
    try {
      const url = new URL(this.serverUrl);
      const postData = JSON.stringify({ code, action: 'synthesize' });

      return await new Promise((resolve, reject) => {
        const req = http.request({
          hostname: url.hostname,
          port: url.port || 8086,
          path: '/api/synthesize',
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData),
            'Connection': 'close',
          },
          timeout: 2000,
        }, (res) => {
          let body = '';
          res.on('data', chunk => body += chunk);
          res.on('end', () => {
            try {
              const parsed = JSON.parse(body);
              resolve(parsed);
            } catch (err) {
              reject(err);
            }
          });
        });

        req.on('error', reject);
        req.on('timeout', () => {
          req.destroy();
          reject(new Error('Synthesis HTTP request timed out'));
        });

        req.write(postData);
        req.end();
      });
    } catch {
      // 2. Fall back to CLI
      return new Promise((resolve) => {
        const binPath = this.findCliBinary();
        if (!binPath) {
          resolve({ step: null, candidates: [] });
          return;
        }

        const os = require('os');
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'proofer-synth-'));
        const tmpFile = path.join(tmpDir, 'synth.proof');
        fs.writeFileSync(tmpFile, code, { mode: 0o600 });

        const proc = spawn(binPath, ['synthesize', tmpFile]);
        let stdout = '';
        proc.stdout.on('data', chunk => stdout += chunk);
        proc.on('close', () => {
          try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch (_) {}
          // Parse CLI output lines
          const stepMatch = stdout.match(/Synthesized Step \(Kernel Verified: (true|false)\):\s*\n\s*(.+)/);
          if (stepMatch) {
            resolve({
              step: {
                text: stepMatch[2].trim(),
                verified_by_kernel: stepMatch[1] === 'true',
                kind: stepMatch[2].trim().startsWith('therefore') ? 'therefore' : 'derive',
                explanation: 'Kernel verified step infill',
              },
              candidates: [],
            });
          } else {
            resolve({ step: null, candidates: [] });
          }
        });
      });
    }
  }
}
