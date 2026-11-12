// docs-proof browser checker glue (REQ-P4).
//
// ES module over the hand-rolled C ABI exported by `docs_proof.wasm`
// (see src/wasm.rs in the docs-proof repo). No build step, no bindgen
// runtime: load the module, write badge JSON bytes into its memory, call
// `dps_verify`, map the status.
//
// Usage:
//   import { loadChecker } from "./docs-proof.js";
//   const checker = await loadChecker();            // fetches docs-proof.wasm
//   const verdict = checker.verifyBadge(badgeJson);
//   if (!verdict.ok) console.warn(verdict.statusName, verdict.message);
//
// The wasm half is the authority on structure, version, hash shape,
// toolchain presence, and the kernel result. When it accepts a badge,
// the parsed record is re-read here for display fields only.

const STATUS_NAMES = {
  0: "ok",
  1: "malformed",
  2: "unsupported-version",
  3: "bad-source-hash",
  4: "empty-toolchain",
  5: "not-verified",
  6: "invalid-utf8",
};

/**
 * Load and instantiate the checker.
 * @param {string|URL} [wasmUrl] defaults to docs-proof.wasm beside this file
 * @returns {Promise<{verifyBadge: (json: string) => object}>}
 */
export async function loadChecker(wasmUrl) {
  const url =
    wasmUrl ?? new URL("docs-proof.wasm", import.meta.url);
  const { instance } = await WebAssembly.instantiateStreaming(
    fetch(url),
    {},
  );
  const ex = instance.exports;
  const memory = ex.memory;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  /**
   * Structurally validate one badge's JSON.
   * @param {string} json badge JSON as embedded in the page
   * @returns {{ok: true, theorem: string, sourceHash: string, leanToolchain: string}
   *         | {ok: false, status: number, statusName: string, message: string}}
   */
  function verifyBadge(json) {
    const bytes = encoder.encode(json);
    const ptr = ex.dps_alloc(bytes.length);
    try {
      new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
      const status = ex.dps_verify(ptr, bytes.length);
      const len = ex.dps_error_len();
      const message =
        len > 0
          ? decoder.decode(
              new Uint8Array(memory.buffer, ex.dps_error_ptr(), len),
            )
          : "";
      if (status !== 0) {
        return {
          ok: false,
          status,
          statusName: STATUS_NAMES[status] ?? `status-${status}`,
          message: message || STATUS_NAMES[status] || `status ${status}`,
        };
      }
      // Display fields come from the page's own JSON; the wasm check is
      // the authority on whether that JSON is trustworthy.
      const record = JSON.parse(json);
      return {
        ok: true,
        theorem: record.theorem,
        sourceHash: record.source_hash,
        leanToolchain: record.lean_toolchain,
      };
    } finally {
      ex.dps_free(ptr, bytes.length);
    }
  }

  return { verifyBadge };
}
