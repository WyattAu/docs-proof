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
