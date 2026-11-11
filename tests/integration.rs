//! REQ-P3 oracles: record shape, round-trip, verify pass/fail on drift.

// Test-only lifts (docs-core decoder-pattern precedent): expectations
// carry context on the setup half and match-arm panics are the failure
// voice. Library paths keep the denies.
#![allow(clippy::expect_used, clippy::panic)]

use docs_proof::{
    source_hash, verify, CertificateBuilder, KernelResult, ProofCertificate, VerifyError,
    CERTIFICATE_VERSION,
};

const LEAN_SRC: &str = "theorem no_arbitrage : True := trivial\n";
const TOOLCHAIN: &str = "leanprover/lean4:v4.21.0";

fn cert() -> ProofCertificate {
    CertificateBuilder::new("no_arbitrage", LEAN_SRC, TOOLCHAIN).verified()
}

#[test]
fn record_has_the_req_p3_fields() {
    let c = cert();
    assert_eq!(c.theorem, "no_arbitrage");
    assert_eq!(c.certificate_version, CERTIFICATE_VERSION);
    assert_eq!(c.kernel_result, KernelResult::Verified);
    assert_eq!(c.source_hash, source_hash(LEAN_SRC));
    assert_eq!(c.source_hash.len(), 64, "sha256 lowercase hex");
    assert!(c
        .source_hash
        .chars()
        .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()));
    assert_eq!(c.lean_toolchain, TOOLCHAIN);
}

#[test]
fn verify_passes_on_fresh_source_and_toolchain() {
    let c = cert();
    assert!(verify(&c, LEAN_SRC, TOOLCHAIN).is_ok());
}

#[test]
fn verify_fails_loudly_on_source_drift() {
    let c = cert();
    let drifted = format!("-- touched\n{LEAN_SRC}");
    match verify(&c, &drifted, TOOLCHAIN) {
        Err(VerifyError::StaleSource { theorem, .. }) => {
            assert_eq!(theorem, "no_arbitrage");
            assert!(VerifyError::StaleSource {
                theorem: theorem.clone(),
                recorded: c.source_hash.clone(),
                current: source_hash(&drifted),
            }
            .to_string()
            .contains("stale proof badge"));
        }
        other => panic!("expected StaleSource, got {other:?}"),
    }
}

#[test]
fn verify_fails_loudly_on_toolchain_drift() {
    let c = cert();
    match verify(&c, LEAN_SRC, "leanprover/lean4:v4.99.0") {
        Err(VerifyError::StaleToolchain {
            recorded, current, ..
        }) => {
            assert_eq!(recorded, TOOLCHAIN);
            assert_eq!(current, "leanprover/lean4:v4.99.0");
        }
        other => panic!("expected StaleToolchain, got {other:?}"),
    }
}

#[test]
fn badge_json_is_compact_and_escaped() {
    let mut c = cert();
    c.theorem = "no_\"arbi\\trage\"\n".into();
    let json = c.badge_json();
    assert!(!json.contains('\n'), "one line");
    assert!(json.contains("no_\\\"arbi\\\\trage\\\"\\n"));
    assert!(json.starts_with('{'));

    // The hand-rolled badge must agree with serde's own serialization.
    let via_serde = serde_json::to_string(&c).expect("serializes");
    assert_eq!(json, via_serde);
}

#[test]
fn payload_round_trips_through_json() {
    let c = cert();
    let json = serde_json::to_string(&c).expect("serializes");
    let back: ProofCertificate = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(c, back);
    assert!(verify(&back, LEAN_SRC, TOOLCHAIN).is_ok());
}

#[test]
fn future_versions_are_rejected_typed() {
    let mut c = cert();
    c.certificate_version = CERTIFICATE_VERSION + 1;
    match verify(&c, LEAN_SRC, TOOLCHAIN) {
        Err(VerifyError::Version { found, supported }) => {
            assert_eq!(found, CERTIFICATE_VERSION + 1);
            assert_eq!(supported, CERTIFICATE_VERSION);
        }
        other => panic!("expected Version, got {other:?}"),
    }
}

#[test]
fn hashing_is_byte_exact_and_stable() {
    let a = source_hash(LEAN_SRC);
    let b = source_hash(LEAN_SRC);
    assert_eq!(a, b);
    let c = source_hash("theorem no_arbitrage : True := trivial"); // no trailing \n
    assert_ne!(a, c, "byte-exact: a trailing newline changes the hash");
}
