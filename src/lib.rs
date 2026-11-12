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
//! embedding, verification against current source + toolchain, structural
//! badge validation (`verify_badge`) for pages and the wasm client bundle,
//! and the browser ABI + JS glue (`CHECKER_JS`, `assets/docs-proof.wasm`).
//! Out: creating certificates (that is the compiler's, after a real
//! kernel run - this crate never claims verification by itself), running
//! Lean (REQ-P1: never linked), and Lean-typed frontmatter schemas
//! (REQ-P5: Rust-first).

#![deny(missing_docs)]
// `unsafe` is forbidden everywhere except src/wasm.rs, the wasm32-only
// C ABI where raw pointers cross the browser boundary; that module
// re-allows it locally and documents every use.
#![deny(unsafe_code)]

mod certificate;
mod error;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use certificate::{
    source_hash, verify, verify_badge, CertificateBuilder, KernelResult, ProofCertificate,
    CERTIFICATE_VERSION,
};
pub use error::{BadgeError, VerifyError};

/// Browser ABI status: badge parsed, validated, kernel-verified.
pub const DPS_OK: u32 = 0;
/// Browser ABI status: input was not valid JSON or off the record shape.
pub const DPS_MALFORMED: u32 = 1;
/// Browser ABI status: certificate format newer than this checker.
pub const DPS_UNSUPPORTED_VERSION: u32 = 2;
/// Browser ABI status: source hash is not 64 lowercase hex characters.
pub const DPS_BAD_SOURCE_HASH: u32 = 3;
/// Browser ABI status: badge records no toolchain.
pub const DPS_EMPTY_TOOLCHAIN: u32 = 4;
/// Browser ABI status: badge does not record kernel verification.
pub const DPS_NOT_VERIFIED: u32 = 5;
/// Browser ABI status: input bytes were not valid UTF-8.
pub const DPS_BAD_UTF8: u32 = 6;

/// The checker's JS glue, as shipped in `assets/docs-proof.js`. Native
/// hosts embed it next to the wasm module; the module itself is a build
/// artifact (`cargo build --release --target wasm32-unknown-unknown`,
/// or the committed `assets/docs-proof.wasm`, which CI keeps fresh and
/// under the 80 KB budget).
pub const CHECKER_JS: &str = include_str!("../assets/docs-proof.js");

/// The crate's version, for callers that report it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
