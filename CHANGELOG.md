# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-11-11

### Added

- `ProofCertificate`: versioned record (theorem, SHA-256 source hash,
  toolchain identifier, kernel result; REQ-P3), serde-serializable with
  a hand-rolled one-line badge JSON whose escaping is verified against
  serde itself.
- `CertificateBuilder`: records what the compiler's out-of-process Lean
  run produced; 0.1 emits `KernelResult::Verified` only (failed proofs
  are build errors, not badges).
- `verify` + `VerifyError`: total re-check against current source and
  toolchain (`StaleSource`, `StaleToolchain`, `Version`, `NotVerified`).
- `source_hash`: byte-exact SHA-256, lowercase hex.
- Tests: record shape, verify pass/fail on both drift axes, badge
  escaping vs serde, round-trip, future-version rejection, byte-exact
  hashing.

## [0.1.1] - 2026-11-12

### Added

- `verify_badge` + `BadgeError`: structural validation of badge JSON as
  it arrives on a page (version, 64-lowercase-hex source hash, non-empty
  toolchain, verified kernel result) - the client-side checker's core
  (REQ-P4). The crate is wasm32-clean; the packaged wasm checker bundle
  remains future work (recorded in SPEC REQ-P4).

### Changed

- `serde_json` moved from dev-dependency to dependency: badges arrive as
  JSON and the client checker parses them. `badge_json` stays hand-rolled
  with its differential test against serde.

## [0.1.2] - 2026-11-12

### Added

- **Browser checker packaging (REQ-P4):** the `wasm` module (wasm32-only)
  exports a hand-rolled C ABI - `dps_alloc`/`dps_free`/`dps_verify`
  plus `dps_error_ptr`/`dps_error_len` - with the status codes as
  crate-root `DPS_*` constants, and `CHECKER_JS` ships the dependency-free
  ES-module glue (`assets/docs-proof.js`). The committed
  `assets/docs-proof.wasm` measures 63.6 KB raw against the 80 KB budget;
  CI's new `checker-bundle` job rebuilds it, enforces the budget, diffs
  it against the committed asset, and drives it through the glue's exact
  call sequence with wasmtime (`tests/wasm_abi.rs`, wasmtime 36 dev-only).
  The earlier "4 KB raw" figure predated the checker's real work
  (serde_json + sha2 + parsing); 63.6 KB is the honest shipped number.

### Changed

- `[profile.release]` now sets `opt-level = "z"`, `lto = "fat"`, and
  `panic = "abort"` to size the browser cdylib (unwinding machinery is
  tens of KB). Library consumers' own profiles always override these for
  the native dependency; nothing downstream changes behavior.
- `#![forbid(unsafe_code)]` became `deny` + a documented module-level
  allow in `src/wasm.rs` only: raw pointers crossing the wasm boundary
  are unavoidable, every use carries a SAFETY comment, and the rest of
  the crate stays unsafe-free.
