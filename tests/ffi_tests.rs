#![cfg(feature = "ffi")]

use jianyi::ffi::{jianyi_iching_cast, jianyi_iching_hexagram, jianyi_result_free, JianyiResult};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Call an FFI fn with a JSON request; return (success, payload-or-error-string).
fn call(f: unsafe extern "C" fn(*const c_char) -> JianyiResult, req: &str) -> (bool, String) {
    let c = CString::new(req).unwrap();
    let r = unsafe { f(c.as_ptr()) };
    let ok = r.success;
    let s = if ok {
        unsafe { CStr::from_ptr(r.data) }.to_str().unwrap().to_owned()
    } else {
        unsafe { CStr::from_ptr(r.error) }.to_str().unwrap().to_owned()
    };
    unsafe { jianyi_result_free(r) };
    (ok, s)
}

/// The jianyi known vector: this seed, no question → 45 → 16, nuclear 53, line 5 changing.
const VECTOR_SEED: &str = "2026-06-12T21:51:54+00:00";

#[test]
fn known_vector_cast() {
    let req = format!(r#"{{"timestamp": "{VECTOR_SEED}"}}"#);
    let (ok, body) = call(jianyi_iching_cast, &req);
    assert!(ok, "error: {body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(v["seed"], VECTOR_SEED, "seed echoed verbatim");
    assert_eq!(v["question"], serde_json::Value::Null);
    assert_eq!(v["primary"]["number"], 45);
    assert_eq!(v["resulting"]["number"], 16);
    assert_eq!(v["nuclear"]["number"], 53);
    assert_eq!(v["changing_lines"], serde_json::json!([5]));
    assert_eq!(v["fingerprint"], "v2_ich_45.c5");
    let lines: Vec<u64> = v["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_u64().unwrap())
        .collect();
    assert_eq!(lines.len(), 6);
    assert!(lines.iter().all(|l| (6..=9).contains(l)));
    assert!(matches!(lines[4], 6 | 9), "changing line 5 must be old");
    assert!(!v["primary"]["judgment"]["chinese"].as_str().unwrap().is_empty());
    assert!(!v["primary"]["judgment"]["english"].as_str().unwrap().is_empty());
}

#[test]
fn cast_is_deterministic() {
    let req = format!(r#"{{"timestamp": "{VECTOR_SEED}"}}"#);
    let (ok1, a) = call(jianyi_iching_cast, &req);
    let (ok2, b) = call(jianyi_iching_cast, &req);
    assert!(ok1 && ok2, "errors: {a} / {b}");
    assert_eq!(a, b);
}

#[test]
fn seed_is_normalized_to_utc() {
    let (ok, body) = call(
        jianyi_iching_cast,
        r#"{"timestamp": "2026-06-12T23:51:54+02:00"}"#,
    );
    assert!(ok, "error: {body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["seed"], VECTOR_SEED);
    assert_eq!(v["primary"]["number"], 45);
}

#[test]
fn no_changing_lines_has_null_resulting_and_bare_fingerprint() {
    for i in 0..200 {
        let seed = format!("2026-01-01T00:{:02}:{:02}+00:00", i / 60, i % 60);
        let req = format!(r#"{{"timestamp": "{seed}"}}"#);
        let (ok, body) = call(jianyi_iching_cast, &req);
        assert!(ok, "error: {body}");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        if v["changing_lines"].as_array().unwrap().is_empty() {
            assert_eq!(v["resulting"], serde_json::Value::Null);
            let fp = v["fingerprint"].as_str().unwrap();
            assert!(!fp.contains(".c"), "bare fingerprint expected, got {fp}");
            assert!(fp.starts_with("v2_ich_"));
            return;
        }
    }
    panic!("no unchanging cast in 200 deterministic seeds — statistically impossible");
}

#[test]
fn invalid_timestamp_is_an_error() {
    let (ok, body) = call(jianyi_iching_cast, r#"{"timestamp": "not-a-time"}"#);
    assert!(!ok);
    assert!(body.contains("invalid timestamp"), "got: {body}");
}

#[test]
fn null_request_is_an_error() {
    let r = unsafe { jianyi_iching_cast(std::ptr::null()) };
    assert!(!r.success);
    unsafe { jianyi_result_free(r) };
}

#[test]
fn hexagram_lookup_by_number_and_binary_agree() {
    let (ok, by_num) = call(jianyi_iching_hexagram, r#"{"number": 45}"#);
    assert!(ok, "error: {by_num}");
    let v: serde_json::Value = serde_json::from_str(&by_num).unwrap();
    assert_eq!(v["number"], 45);
    let binary = v["binary"].as_str().unwrap().to_string();
    assert_eq!(v["lines"].as_array().unwrap().len(), 6);

    let (ok2, by_bin) = call(
        jianyi_iching_hexagram,
        &format!(r#"{{"binary": "{binary}"}}"#),
    );
    assert!(ok2, "error: {by_bin}");
    assert_eq!(by_num, by_bin);
}

#[test]
fn hexagram_lookup_requires_exactly_one_selector() {
    let (ok, _) = call(jianyi_iching_hexagram, r#"{}"#);
    assert!(!ok);
    let (ok2, _) = call(
        jianyi_iching_hexagram,
        r#"{"number": 45, "binary": "000110"}"#,
    );
    assert!(!ok2);
}
