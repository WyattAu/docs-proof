//! Browser-facing C ABI for the badge checker (REQ-P4 packaging).
//!
//! This module compiles only for wasm32: it is the export surface of the
//! `docs_proof.wasm` cdylib that pages load (see `assets/docs-proof.js`
//! for the glue). Every export takes and returns `u32`/pointers-as-u32,
//! so the ABI is writable from any language with typed arrays and needs
//! no bindgen runtime.
//!
//! Status codes: 0 ok; 1 malformed badge JSON; 2 unsupported
//! certificate version; 3 malformed source hash; 4 empty toolchain; 5
//! kernel result not verified; 6 input not valid UTF-8 (the `DPS_*`
//! constants in the crate root). On failure the human-readable reason is
//! available through `dps_error_ptr` / `dps_error_len` (NUL-terminated
//! UTF-8, truncated to fit; cleared on success).
//!
//! The rest of the crate is native-first and `unsafe`-free; this file is
//! the only place `unsafe` appears, because raw pointers crossing the
//! wasm boundary are unavoidable. Every use carries its safety argument.
#![allow(unsafe_code)]

use crate::{
    verify_badge, BadgeError, DPS_BAD_SOURCE_HASH, DPS_BAD_UTF8, DPS_EMPTY_TOOLCHAIN,
    DPS_MALFORMED, DPS_NOT_VERIFIED, DPS_OK, DPS_UNSUPPORTED_VERSION,
};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Error-sink capacity in bytes, including the trailing NUL.
const ERR_CAP: usize = 192;

/// Single-slot error message. wasm32-unknown-unknown is single-threaded
/// per instance, and the JS glue writes the badge, calls `dps_verify`,
/// then reads the slot back before any other call - so a plain unsynchronized
/// cell plus a release-store length is a sufficient protocol.
struct ErrSink {
    buf: UnsafeCell<[u8; ERR_CAP]>,
    len: AtomicUsize,
}

// Safety: see the type doc - the wasm ABI is a sequential protocol; no
// two threads ever touch the sink in the supported embedding.
unsafe impl Sync for ErrSink {}

static ERR: ErrSink = ErrSink {
    buf: UnsafeCell::new([0; ERR_CAP]),
    len: AtomicUsize::new(0),
};

impl ErrSink {
    fn set(&self, msg: &str) {
        // SAFETY: single-threaded wasm protocol per the type doc.
        let buf = unsafe { &mut *self.buf.get() };
        let mut n = 0usize;
        {
            let bytes = buf.iter_mut().take(ERR_CAP - 1).zip(msg.bytes());
            for (slot, byte) in bytes {
                *slot = byte;
                n += 1;
            }
        }
        if let Some(slot) = buf.get_mut(n) {
            *slot = 0;
        }
        self.len.store(n, Ordering::Release);
    }

    fn clear(&self) {
        self.len.store(0, Ordering::Release);
    }
}

/// Allocate exactly `len` bytes for the host to write badge JSON into.
/// Returns the pointer as u32; free it with `dps_free(ptr, len)`.
#[no_mangle]
pub extern "C" fn dps_alloc(len: u32) -> u32 {
    // A boxed slice has capacity == len, so `dps_free` can reconstruct
    // the allocation from (ptr, len) alone.
    Box::into_raw(vec![0u8; len as usize].into_boxed_slice()) as *mut u8 as u32
}

/// Free an allocation made by `dps_alloc`.
///
/// # Safety
/// `ptr` must be a live `dps_alloc` result and `len` its original length;
/// the pointer must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn dps_free(ptr: u32, len: u32) {
    if ptr == 0 {
        return;
    }
    // SAFETY: caller contract above; boxed-slice round trip with the
    // exact allocation length.
    drop(Vec::from_raw_parts(
        ptr as *mut u8,
        len as usize,
        len as usize,
    ));
}

/// Verify badge JSON: `len` bytes of UTF-8 JSON at `ptr`. Returns one of
/// the `DPS_*` status codes; `DPS_OK` (0) means the badge is
/// structurally valid and records a kernel-verified proof.
///
/// The error slot carries short static messages rather than the typed
/// error's Display output: formatting machinery costs tens of KB in the
/// browser artifact, and the page already holds the badge JSON the
/// diagnostics would quote.
#[no_mangle]
pub extern "C" fn dps_verify(ptr: u32, len: u32) -> u32 {
    let bytes =
        // SAFETY: the host wrote `len` badge bytes at `ptr` per the glue
        // contract; reads never outlive the call.
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        ERR.set("badge input is not valid UTF-8");
        return DPS_BAD_UTF8;
    };
    match verify_badge(text) {
        Ok(_) => {
            ERR.clear();
            DPS_OK
        }
        Err(e) => {
            ERR.set(message_of(&e));
            status_of(&e)
        }
    }
}

/// Map a badge failure to its status code.
fn status_of(e: &BadgeError) -> u32 {
    match e {
        BadgeError::Malformed(_) => DPS_MALFORMED,
        BadgeError::Version { .. } => DPS_UNSUPPORTED_VERSION,
        BadgeError::BadSourceHash { .. } => DPS_BAD_SOURCE_HASH,
        BadgeError::EmptyToolchain => DPS_EMPTY_TOOLCHAIN,
        BadgeError::NotVerified { .. } => DPS_NOT_VERIFIED,
    }
}

/// Static per-category reason (no formatting: see the `dps_verify` doc).
fn message_of(e: &BadgeError) -> &'static str {
    match e {
        BadgeError::Malformed(_) => "badge JSON is malformed or off the record shape",
        BadgeError::Version { .. } => "badge version unsupported by this checker",
        BadgeError::BadSourceHash { .. } => "badge source hash is not 64 lowercase hex",
        BadgeError::EmptyToolchain => "badge records no toolchain",
        BadgeError::NotVerified { .. } => "badge does not record kernel verification",
    }
}

/// Pointer to the last error message (NUL-terminated UTF-8), valid until
/// the next `dps_verify` call. Length 0 (via `dps_error_len`) means the
/// last call succeeded.
#[no_mangle]
pub extern "C" fn dps_error_ptr() -> u32 {
    // SAFETY: sequential wasm protocol per the ErrSink doc.
    (unsafe { (*ERR.buf.get()).as_ptr() }) as u32
}

/// Length of the last error message, excluding the NUL; 0 on success.
#[no_mangle]
pub extern "C" fn dps_error_len() -> u32 {
    ERR.len.load(Ordering::Acquire) as u32
}
