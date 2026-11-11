//! The certificate record, its builder, and the verifier.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::VerifyError;

/// The kernel verdict recorded for one theorem.
///
/// 0.1 records only successful verification: a failed proof is a build
/// error in the compiler (REQ-P2), never an embedded certificate. The
/// enum exists so future kernel outcomes can be added without breaking
/// the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelResult {
    /// The Lean kernel accepted the proof.
    Verified,
}

/// The format version of the certificate record.
pub const CERTIFICATE_VERSION: u32 = 1;

/// A versioned record of one kernel-checked theorem (REQ-P3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCertificate {
    /// Record format version.
    pub certificate_version: u32,
    /// Fully qualified Lean theorem name.
    pub theorem: String,
    /// SHA-256 (lowercase hex) of the exact Lean source bytes.
    pub source_hash: String,
    /// Toolchain identifier that ran the kernel (opaque; the compiler
    /// records the `lean-toolchain` value or `lean --version` output).
    pub lean_toolchain: String,
    /// The kernel verdict.
    pub kernel_result: KernelResult,
}

impl ProofCertificate {
    /// The badge payload: compact JSON, one line, suitable for embedding
    /// in a `<docs-island>` props block or a
    /// `<script type="application/json">`. Hand-rolled (serde_json is a
    /// dev-dependency) so the runtime floor stays at serde alone.
    pub fn badge_json(&self) -> String {
        format!(
            r#"{{"certificate_version":{v},"theorem":"{t}","source_hash":"{h}","lean_toolchain":"{c}","kernel_result":"verified"}}"#,
            v = self.certificate_version,
            t = json_escape(&self.theorem),
            h = self.source_hash,
            c = json_escape(&self.lean_toolchain),
        )
    }
}

/// Minimal JSON string escaping for the badge (quotes, backslash, and the
/// control characters JSON forbids inside strings).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// SHA-256 over the exact source bytes, lowercase hex.
pub fn source_hash(source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Records what the compiler's out-of-process Lean run actually produced.
#[derive(Debug)]
pub struct CertificateBuilder {
    theorem: String,
    source_hash: String,
    lean_toolchain: String,
}

impl CertificateBuilder {
    /// Start a certificate for `theorem`, compiled from `source` under
    /// `lean_toolchain`.
    pub fn new(theorem: &str, source: &str, lean_toolchain: &str) -> Self {
        Self {
            theorem: theorem.to_string(),
            source_hash: source_hash(source),
            lean_toolchain: lean_toolchain.to_string(),
        }
    }

    /// Record a successful kernel verification and finish the certificate.
    pub fn verified(self) -> ProofCertificate {
        ProofCertificate {
            certificate_version: CERTIFICATE_VERSION,
            theorem: self.theorem,
            source_hash: self.source_hash,
            lean_toolchain: self.lean_toolchain,
            kernel_result: KernelResult::Verified,
        }
    }
}

/// Re-check a certificate against the *current* source and toolchain.
///
/// A mismatch means the page's badge claims proof of something the
/// present source (or toolchain) no longer says; callers treat that as a
/// build failure, not a warning (REQ-P3's drift contract).
pub fn verify(
    cert: &ProofCertificate,
    current_source: &str,
    current_toolchain: &str,
) -> Result<(), VerifyError> {
    if cert.certificate_version != CERTIFICATE_VERSION {
        return Err(VerifyError::Version {
            found: cert.certificate_version,
            supported: CERTIFICATE_VERSION,
        });
    }
    let now_hash = source_hash(current_source);
    if now_hash != cert.source_hash {
        return Err(VerifyError::StaleSource {
            theorem: cert.theorem.clone(),
            recorded: cert.source_hash.clone(),
            current: now_hash,
        });
    }
    if cert.lean_toolchain != current_toolchain {
        return Err(VerifyError::StaleToolchain {
            theorem: cert.theorem.clone(),
            recorded: cert.lean_toolchain.clone(),
            current: current_toolchain.to_string(),
        });
    }
    if cert.kernel_result != KernelResult::Verified {
        return Err(VerifyError::NotVerified {
            theorem: cert.theorem.clone(),
        });
    }
    Ok(())
}
