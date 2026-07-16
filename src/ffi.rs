//! C ABI for host apps (zhouyi's Expo module on iOS/Android).
//!
//! JSON strings in, JSON strings out — contract-identical to kairos-ffi's
//! `kairos_iching_*` surface, so clients written against one engine binding
//! decode the other unchanged. The pure JSON transforms live in
//! `crate::json_api`; this module only handles the C boundary (pointer
//! validation, string marshaling, memory ownership).
//!
//! Memory: every returned `JianyiResult` must be released with
//! `jianyi_result_free` exactly once.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Result of an FFI call. On success `data` holds a JSON payload; on failure
/// `error` holds a message. Exactly one of the two is non-null.
#[repr(C)]
pub struct JianyiResult {
    pub success: bool,
    pub data: *mut c_char,
    pub error: *mut c_char,
}

fn to_c_string(s: String) -> *mut c_char {
    // Interior NULs cannot occur in our serde output; fall back defensively.
    CString::new(s)
        .unwrap_or_else(|_| CString::new("{\"error\":\"interior NUL\"}").expect("static"))
        .into_raw()
}

impl JianyiResult {
    fn success(data: String) -> Self {
        Self {
            success: true,
            data: to_c_string(data),
            error: std::ptr::null_mut(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: std::ptr::null_mut(),
            error: to_c_string(message),
        }
    }
}

/// Free a `JianyiResult` returned by any `jianyi_*` function.
#[no_mangle]
pub unsafe extern "C" fn jianyi_result_free(result: JianyiResult) {
    if !result.data.is_null() {
        drop(CString::from_raw(result.data));
    }
    if !result.error.is_null() {
        drop(CString::from_raw(result.error));
    }
}

/// Borrow the incoming request pointer as a `&str`, validating it is
/// non-null and valid UTF-8.
unsafe fn cstr_to_str<'a>(request_json: *const c_char) -> Result<&'a str, String> {
    if request_json.is_null() {
        return Err("request is null".to_string());
    }
    unsafe { CStr::from_ptr(request_json) }
        .to_str()
        .map_err(|e| e.to_string())
}

/// Cast the I Ching. Deterministic given `timestamp`; open cast when `question`
/// is omitted. Returns the full reading payload: seed, lines, changing lines,
/// figure fingerprint, and resolved primary/resulting/nuclear hexagrams.
#[no_mangle]
pub unsafe extern "C" fn jianyi_iching_cast(request_json: *const c_char) -> JianyiResult {
    match unsafe { cstr_to_str(request_json) }.and_then(crate::json_api::cast_json) {
        Ok(json) => JianyiResult::success(json),
        Err(e) => JianyiResult::error(e),
    }
}

/// Look up a single hexagram by King Wen `number` or by 6-char bottom-up `binary`.
/// Exactly one selector must be provided.
#[no_mangle]
pub unsafe extern "C" fn jianyi_iching_hexagram(request_json: *const c_char) -> JianyiResult {
    match unsafe { cstr_to_str(request_json) }.and_then(crate::json_api::hexagram_json) {
        Ok(json) => JianyiResult::success(json),
        Err(e) => JianyiResult::error(e),
    }
}
