import initWasm, { verify_proof_wasm, proofer_version } from '../wasm/proof_wasm.js';
import { VerificationResponse } from '../types';

let wasmInitialized = false;
let initPromise: Promise<boolean> | null = null;

export async function initProoferWasm(): Promise<boolean> {
  if (wasmInitialized) return true;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      await initWasm();
      wasmInitialized = true;
      console.log(`[Proofer] WebAssembly Kernel initialized: ${proofer_version()}`);
      return true;
    } catch (err) {
      console.warn('[Proofer] WebAssembly Kernel initialization failed, falling back to HTTP:', err);
      return false;
    }
  })();

  return initPromise;
}

export function isWasmReady(): boolean {
  return wasmInitialized;
}

export function verifyProofWasm(code: string): VerificationResponse | null {
  if (!wasmInitialized) return null;
  try {
    const jsonStr = verify_proof_wasm(code);
    return JSON.parse(jsonStr) as VerificationResponse;
  } catch (err) {
    console.error('[Proofer] WebAssembly verification error:', err);
    return null;
  }
}
