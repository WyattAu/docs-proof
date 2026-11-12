//! End-to-end oracle over the shipped browser artifact (REQ-P4
//! packaging): the committed `assets/docs-proof.wasm` is loaded with
//! wasmtime and driven through the exact call sequence the JS glue
//! makes - alloc, copy, verify, read error slot, free.
//!
//! Skipped unless `DOCS_PROOF_WASM` points at the module (CI's
//! checker-bundle job sets it after building the cdylib and diffing it
//! against the committed asset). The test target alone may lift the
//! denies; library paths keep them.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use docs_proof::{
    CertificateBuilder, KernelResult, DPS_BAD_SOURCE_HASH, DPS_BAD_UTF8, DPS_MALFORMED, DPS_OK,
    DPS_UNSUPPORTED_VERSION,
};
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

const TOOLCHAIN: &str = "leanprover/lean4:v4.35.0";

/// A loaded checker module with the exports the JS glue uses.
struct Checker {
    store: Store<()>,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    verify: TypedFunc<(u32, u32), u32>,
    error_ptr: TypedFunc<(), u32>,
    error_len: TypedFunc<(), u32>,
    memory: Memory,
}

impl Checker {
    fn load(path: &str) -> Self {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path).expect("wasm module loads");
        let mut store = Store::new(&engine, ());
        let instance =
            Instance::new(&mut store, &module, &[]).expect("module instantiates with no imports");
        Self {
            alloc: instance
                .get_typed_func::<u32, u32>(&mut store, "dps_alloc")
                .expect("dps_alloc exported"),
            free: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "dps_free")
                .expect("dps_free exported"),
            verify: instance
                .get_typed_func::<(u32, u32), u32>(&mut store, "dps_verify")
                .expect("dps_verify exported"),
            error_ptr: instance
                .get_typed_func::<(), u32>(&mut store, "dps_error_ptr")
                .expect("dps_error_ptr exported"),
            error_len: instance
                .get_typed_func::<(), u32>(&mut store, "dps_error_len")
                .expect("dps_error_len exported"),
            memory: instance
                .get_memory(&mut store, "memory")
                .expect("exported memory"),
            store,
        }
    }

    /// The glue's exact sequence: alloc, copy, verify, read slot, free.
    fn check(&mut self, json: &str) -> (u32, String) {
        let bytes = json.as_bytes();
        let ptr = self
            .alloc
            .call(&mut self.store, bytes.len() as u32)
            .expect("alloc");
        {
            let raw = self.memory.data_mut(&mut self.store);
            let dst = &mut raw[ptr as usize..ptr as usize + bytes.len()];
            dst.copy_from_slice(bytes);
        }
        let status = self
            .verify
            .call(&mut self.store, (ptr, bytes.len() as u32))
            .expect("verify");
        let len = self.error_len.call(&mut self.store, ()).expect("error_len");
        let message = if len > 0 {
            let error_ptr = self.error_ptr.call(&mut self.store, ()).expect("error_ptr");
            let raw = self.memory.data(&self.store);
            let start = error_ptr as usize;
            String::from_utf8_lossy(&raw[start..start + len as usize]).into_owned()
        } else {
            String::new()
        };
        self.free
            .call(&mut self.store, (ptr, bytes.len() as u32))
            .expect("free");
        (status, message)
    }
}

#[test]
fn browser_artifact_accepts_a_real_badge_and_rejects_tampering() {
    let Some(path) = std::env::var_os("DOCS_PROOF_WASM") else {
        eprintln!("skipped: DOCS_PROOF_WASM not set (no wasm artifact to drive)");
        return;
    };
    let mut checker = Checker::load(&path.to_string_lossy());

    let cert = CertificateBuilder::new("no_arbitrage", "lean source", TOOLCHAIN).verified();
    assert!(matches!(cert.kernel_result, KernelResult::Verified));
    let badge = cert.badge_json();

    // A real badge: status 0, empty error slot.
    let (status, message) = checker.check(&badge);
    assert_eq!(status, DPS_OK, "a real badge must verify");
    assert_eq!(message, "", "success clears the error slot");

    // Uppercase hash breaks the lowercase-hex contract.
    let tampered = badge.replace(&cert.source_hash, &"A".repeat(64));
    assert_ne!(tampered, badge, "tamper must change the badge");
    let (status, message) = checker.check(&tampered);
    assert_eq!(status, DPS_BAD_SOURCE_HASH, "uppercase hash rejected");
    assert!(message.contains("64 lowercase hex"), "reason: {message}");

    // Future version.
    let (status, message) =
        checker.check(&badge.replace("certificate_version\":1", "certificate_version\":99"));
    assert_eq!(status, DPS_UNSUPPORTED_VERSION);
    assert!(message.contains("unsupported"), "reason: {message}");

    // Truncated JSON.
    let (status, message) = checker.check(&badge[..badge.len() / 2]);
    assert_eq!(status, DPS_MALFORMED);
    assert!(message.contains("malformed"), "reason: {message}");

    // Not UTF-8.
    let bytes = vec![0xff, 0xfe, b'{'];
    let ptr = checker
        .alloc
        .call(&mut checker.store, bytes.len() as u32)
        .expect("alloc");
    checker.memory.data_mut(&mut checker.store)[ptr as usize..ptr as usize + bytes.len()]
        .copy_from_slice(&bytes);
    let status = checker
        .verify
        .call(&mut checker.store, (ptr, bytes.len() as u32))
        .expect("verify");
    checker
        .free
        .call(&mut checker.store, (ptr, bytes.len() as u32))
        .expect("free");
    assert_eq!(status, DPS_BAD_UTF8);
}
