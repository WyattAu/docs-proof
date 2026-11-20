# Pattern adoption

docs-proof is the L0 proof-certificate crate for docs sites: a versioned
record (theorem, source hash, toolchain, kernel result) plus a verifier
that flags stale sources or toolchains. Declared per
references/adoption-format.md in pattern-standards.

| Pattern | Status | Evidence |
|---|---|---|
| T1 | adopted | src/certificate.rs:161 (badge JSON is parsed once into a typed certificate; wrong versions and malformed hashes are typed errors at src/certificate.rs:170, not later re-checks) |
| T49 | partial | src/certificate.rs:120 (the verifier half of the pipeline: a recorded certificate is re-verified against the current source hash and toolchain; src/certificate.rs:133 and src/certificate.rs:140 name the stale cases) |
