# docs-proof

Lean proof certificates for documentation sites: a compact, versioned
record of one kernel-checked theorem (theorem name, SHA-256 of the exact
Lean source, toolchain identifier, kernel result) plus a verifier that
re-checks the record against the *current* source and toolchain, so a
drifted page fails loudly instead of claiming stale proof. Part of the
docs-core extension program (`docs-core/SPEC.md` REQ-P).

## Module map

| Module | Contents |
|---|---|
| `CertificateBuilder` / `ProofCertificate` | the record (REQ-P3): serde-serializable, one-line badge JSON |
| `source_hash` | SHA-256 over exact source bytes, lowercase hex |
| `verify` | total re-check: version, source hash, toolchain, kernel result |
| `VerifyError` | `StaleSource` / `StaleToolchain` / `Version` / `NotVerified`, each naming the theorem |

## Guarantees (0.1, honest)

- **Zero Lean linkage, ever** (REQ-P1): Lean runs out-of-process in the
  docs compiler; this crate only describes and checks the record. The
  dependency tree proves it: serde, sha2, thiserror.
- **Byte-exact hashing**: a one-byte change (even a trailing newline)
  changes the hash; verification treats that as stale, not close enough.
- **Failures never become badges**: 0.1 records `KernelResult::Verified`
  only; a failed proof is a build error in the compiler (REQ-P2). The
  enum exists so future kernel outcomes do not break the wire format.

## Scope (0.1, honest)

Out: creating certificates (the compiler's job, after a real kernel
run), the client-side wasm re-checker (REQ-P4: future phase, recorded
here as absent by decision), and Lean-typed frontmatter schemas
(REQ-P5: Rust-first; see SPEC).

## Layer

L0 - leaf. Zero estate-internal dependencies and zero Lean linkage;
external floor: serde, sha2, thiserror.

## Tests

`cargo test` under clippy `-D warnings` with the estate decoder-pattern
denies lifted in test targets only: record shape (REQ-P3), verify
pass/fail on source drift and toolchain drift, badge JSON correctness
(hand-rolled escaping verified against serde), JSON round-trip, future
version rejection, byte-exact hashing.

Licensed under MIT OR Apache-2.0.
