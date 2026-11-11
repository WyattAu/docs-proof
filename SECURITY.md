# Security Policy

## Reporting a vulnerability

Email wyatt_au@protonmail.com with the repo name in the subject. Please
include reproduction steps; do not open a public issue for an unfixed
vulnerability.

## Content discipline

A certificate claims a fact about a source file; the verifier is total
and every mismatch is a typed error (`VerifyError`), never a panic. The
source hash is SHA-256 over the exact bytes; verification fails on any
byte difference, including trailing whitespace. Badges hand-roll JSON
escaping and the escaping is differential-tested against serde's own
serializer. If you find an input that panics, hangs, or lets a stale
certificate verify, that is a security bug.

## Trust model

This crate records and checks claims; it does not create them. The
kernel verdict comes from an out-of-process Lean run performed by the
docs compiler (REQ-P1: no Lean linkage in this crate or anywhere in its
tree). The badge is therefore as trustworthy as the compiler's toolchain
provenance; see docs-core SPEC.md REQ-P for the full chain.

## Supported versions

Only the latest released `0.1.x` is supported with fixes.
