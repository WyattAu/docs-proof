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
run), the packaged wasm checker bundle (REQ-P4: the *core* has landed -
`verify_badge` plus a wasm32-clean build, CI-enforced, measuring 4 KB
raw cdylib against the 80 KB budget - while the packaging and JS glue
remain future work), and Lean-typed frontmatter schemas
(REQ-P5: Rust-first; see SPEC).


## Browser checker bundle (REQ-P4 packaging)

The crate compiles into a hand-rolled-C-ABI wasm module plus a
dependency-free ES module glue (`CHECKER_JS`, shipped in
`assets/docs-proof.js`):

```text
assets/docs-proof.wasm  63.6 KB raw against the 80 KB budget (CI gate)
assets/docs-proof.js    loadChecker() -> verifyBadge(json) -> verdict
```

The wasm half is the authority on structure, version, hash shape,
toolchain presence, and the kernel result; the glue only re-reads
display fields from the page's own JSON after the wasm check accepts.
Status codes are the crate-root `DPS_*` constants. Build the module with
`bash scripts/build-wasm.sh` (cargo rustc --crate-type cdylib); CI
rebuilds it, holds it to the budget, diffs it against the committed
asset, and drives it through the glue's exact call sequence with
wasmtime (`tests/wasm_abi.rs`). The error slot carries short static
messages: formatting machinery would cost tens of KB, and the page
already holds the JSON the diagnostics would quote.

## Where the truth lives

| Concern | Truth | Path |
|---|---|---|
| Record shape, verify, badge validation | source of record | `src/certificate.rs` |
| Typed failures (`VerifyError`, `BadgeError`) | source of record | `src/error.rs` |
| Browser ABI + status codes (REQ-P4) | source of record | `src/wasm.rs`, `assets/docs-proof.js` |
| REQ-P anchors (proof-carrying docs, REQ-P1 no Lean linkage) | SPEC | `docs-core/SPEC.md` |
| Layer declaration + cdylib budget profile | manifest metadata | `Cargo.toml` |

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
