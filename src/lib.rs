//! docs-proof: Lean proof certificates for documentation sites
//! (docs-core extension program, `SPEC.md` REQ-P).
//!
//! A certificate is a compact, versioned record of one kernel-checked
//! theorem: its name, the SHA-256 of the exact Lean source that produced
//! it, the toolchain that ran the kernel, and the kernel result. The
//! compiler creates certificates after an out-of-process `lean` run
//! (REQ-P1: Lean is never linked); pages embed them as static badges;
//! [`verify`] re-checks a certificate against the *current* source and
//! toolchain so a drifted page fails loudly instead of claiming stale
//! proof (REQ-P3).
//!
//! ```
//! use docs_proof::{CertificateBuilder, verify};
//!
//! let source = "theorem no_arbitrage : True := trivial\n";
//! let cert = CertificateBuilder::new("no_arbitrage", source, "leanprover/lean4:v4.21.0")
//!     .verified();
//! assert!(verify(&cert, source, "leanprover/lean4:v4.21.0").is_ok());
//! assert!(verify(&cert, source, "leanprover/lean4:v4.22.0").is_err());
//! ```
//!
//! # Scope (0.1, honest)
//!
//! In: the record, SHA-256 source hashing, serde round-trip for static
//! embedding, verification against current source + toolchain, and
//! structural badge validation (`verify_badge`) so a page (or the wasm
//! client bundle this crate compiles into) can reject a malformed or
//! stale-versioned badge before trusting it.
//! Out: creating certificates (that is the compiler's, after a real
//! kernel run - this crate never claims verification by itself), the
//! packaged wasm checker bundle itself (REQ-P4: the crate is
//! wasm32-proven in CI and the badge validator is the checker's core;
//! packaging remains a future phase), and Lean-typed frontmatter schemas
//! (REQ-P5: Rust-first).

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod certificate;
mod error;

pub use certificate::{
    source_hash, verify, verify_badge, CertificateBuilder, KernelResult, ProofCertificate,
    CERTIFICATE_VERSION,
};
pub use error::{BadgeError, VerifyError};

/// The crate's version, for callers that report it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
