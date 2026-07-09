//! C ABI for host apps (zhouyi's Expo module on iOS/Android).
//!
//! JSON strings in, JSON strings out — contract-identical to kairos-ffi's
//! `kairos_iching_*` surface, so clients written against one engine binding
//! decode the other unchanged. The engine performs seed normalization
//! (parse RFC 3339 → UTC → re-serialize) and returns the seed it actually
//! hashed; hosts store that verbatim and never format timestamps.
//!
//! Memory: every returned `JianyiResult` must be released with
//! `jianyi_result_free` exactly once.

use crate::iching::data::{lookup_hexagram_by_binary, lookup_trigram_by_binary, HexagramRecord};
use crate::iching::{cast, lookup_hexagram, LineValue};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
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

unsafe fn parse_request<T: for<'de> Deserialize<'de>>(
    request_json: *const c_char,
) -> Result<T, Box<dyn Error>> {
    if request_json.is_null() {
        return Err("request is null".into());
    }
    let s = CStr::from_ptr(request_json).to_str()?;
    Ok(serde_json::from_str(s)?)
}

fn ffi_wrapper<F>(f: F) -> JianyiResult
where
    F: FnOnce() -> Result<String, Box<dyn Error>>,
{
    match f() {
        Ok(data) => JianyiResult::success(data),
        Err(e) => JianyiResult::error(format!("{e}")),
    }
}

// ── figure fingerprint ────────────────────────────────────────────────────────
//
// Format canonical in kairos-core (`v2_ich_{primary}` / `v2_ich_{primary}.c{lines}`,
// lines ascending). Re-stated here in ten lines so the lean jianyi artifact can
// mint it without dragging kairos-core; the vector test pins the exact string.

fn figure_fingerprint(primary: u8, changing_lines: &[u8]) -> Result<String, Box<dyn Error>> {
    if !(1..=64).contains(&primary) {
        return Err(format!("hexagram number out of range: {primary}").into());
    }
    if changing_lines.iter().any(|l| !(1..=6).contains(l)) {
        return Err("changing line out of range".into());
    }
    if changing_lines.is_empty() {
        Ok(format!("v2_ich_{primary}"))
    } else {
        let digits: String = changing_lines.iter().map(|l| l.to_string()).collect();
        Ok(format!("v2_ich_{primary}.c{digits}"))
    }
}

// ── request/response contracts (kairos-ffi-identical) ───────────────────────

#[derive(Deserialize)]
struct CastRequest {
    /// RFC-3339 seed timestamp; omitted → now. Normalized to UTC by the engine.
    #[serde(default)]
    timestamp: Option<String>,
    /// Optional question (question-in-seed mode). Omit for an open cast.
    #[serde(default)]
    question: Option<String>,
}

#[derive(Serialize)]
struct TrigramInfo {
    binary: String,
    symbol: String,
    pinyin: String,
    english: String,
    quality: String,
    family: String,
}

#[derive(Serialize)]
struct Rendering {
    chinese: String,
    english: String,
}

#[derive(Serialize)]
struct LineInfo {
    position: u8,
    chinese: String,
    english: String,
}

#[derive(Serialize)]
struct HexagramInfo {
    number: u8,
    binary: String,
    chinese: String,
    pinyin: String,
    english: String,
    upper_trigram: TrigramInfo,
    lower_trigram: TrigramInfo,
    judgment: Rendering,
    image: Rendering,
    lines: Vec<LineInfo>,
}

#[derive(Serialize)]
struct CastResponse {
    /// The normalized seed string the engine hashed. Store verbatim.
    seed: String,
    question: Option<String>,
    /// Six line values 6/7/8/9, bottom-up (index 0 = line 1).
    lines: Vec<u8>,
    /// Changing-line positions, 1-indexed from the bottom.
    changing_lines: Vec<u8>,
    /// Figure fingerprint, e.g. `v2_ich_45.c5`.
    fingerprint: String,
    primary: HexagramInfo,
    resulting: Option<HexagramInfo>,
    nuclear: HexagramInfo,
}

fn trigram_info(binary: &str) -> Result<TrigramInfo, Box<dyn Error>> {
    let t = lookup_trigram_by_binary(binary)
        .ok_or_else(|| format!("unknown trigram binary '{binary}'"))?;
    Ok(TrigramInfo {
        binary: t.binary.clone(),
        symbol: t.symbol.clone(),
        pinyin: t.pinyin.clone(),
        english: t.english.clone(),
        quality: t.quality.clone(),
        family: t.family.clone(),
    })
}

fn hexagram_info(record: &HexagramRecord) -> Result<HexagramInfo, Box<dyn Error>> {
    Ok(HexagramInfo {
        number: record.number,
        binary: record.binary.clone(),
        chinese: record.chinese.clone(),
        pinyin: record.pinyin.clone(),
        english: record.english.clone(),
        upper_trigram: trigram_info(&record.upper_trigram)?,
        lower_trigram: trigram_info(&record.lower_trigram)?,
        judgment: Rendering {
            chinese: record.judgment.chinese.clone(),
            english: record.judgment.english.clone(),
        },
        image: Rendering {
            chinese: record.image.chinese.clone(),
            english: record.image.english.clone(),
        },
        lines: record
            .lines
            .iter()
            .map(|l| LineInfo {
                position: l.position,
                chinese: l.chinese.clone(),
                english: l.english.clone(),
            })
            .collect(),
    })
}

fn hexagram_info_by_number(number: u8) -> Result<HexagramInfo, Box<dyn Error>> {
    let record =
        lookup_hexagram(number).ok_or_else(|| format!("unknown hexagram number {number}"))?;
    hexagram_info(record)
}

fn line_number(value: &LineValue) -> u8 {
    match value {
        LineValue::OldYin => 6,
        LineValue::YoungYang => 7,
        LineValue::YoungYin => 8,
        LineValue::OldYang => 9,
    }
}

/// Normalize a seed exactly as `jianyi iching cast --seed` does: parse RFC 3339,
/// convert to UTC, re-serialize via chrono. Omitted → now.
fn normalize_seed(seed: Option<&str>) -> Result<String, Box<dyn Error>> {
    match seed {
        Some(s) => Ok(chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| format!("invalid timestamp '{s}': {e}"))?
            .with_timezone(&Utc)
            .to_rfc3339()),
        None => Ok(Utc::now().to_rfc3339()),
    }
}

/// Cast the I Ching. Deterministic given `timestamp`; open cast when `question`
/// is omitted. Returns the full reading payload: seed, lines, changing lines,
/// figure fingerprint, and resolved primary/resulting/nuclear hexagrams.
#[no_mangle]
pub unsafe extern "C" fn jianyi_iching_cast(request_json: *const c_char) -> JianyiResult {
    ffi_wrapper(|| {
        let req: CastRequest = unsafe { parse_request(request_json) }?;
        let seed = normalize_seed(req.timestamp.as_deref())?;
        let result = cast(req.question.as_deref(), &seed);

        let resp = CastResponse {
            seed: result.timestamp.clone(),
            question: result.question.clone(),
            lines: result.lines.iter().map(line_number).collect(),
            changing_lines: result.changing_lines.clone(),
            fingerprint: figure_fingerprint(result.primary_number, &result.changing_lines)?,
            primary: hexagram_info_by_number(result.primary_number)?,
            resulting: result
                .resulting_number
                .map(hexagram_info_by_number)
                .transpose()?,
            nuclear: hexagram_info_by_number(result.nuclear_number)?,
        };
        Ok(serde_json::to_string(&resp)?)
    })
}

#[derive(Deserialize)]
struct HexagramRequest {
    #[serde(default)]
    number: Option<u8>,
    #[serde(default)]
    binary: Option<String>,
}

/// Look up a single hexagram by King Wen `number` or by 6-char bottom-up `binary`.
/// Exactly one selector must be provided.
#[no_mangle]
pub unsafe extern "C" fn jianyi_iching_hexagram(request_json: *const c_char) -> JianyiResult {
    ffi_wrapper(|| {
        let req: HexagramRequest = unsafe { parse_request(request_json) }?;
        let record = match (req.number, req.binary.as_deref()) {
            (Some(n), None) => {
                lookup_hexagram(n).ok_or_else(|| format!("unknown hexagram number {n}"))?
            }
            (None, Some(b)) => lookup_hexagram_by_binary(b)
                .ok_or_else(|| format!("unknown hexagram binary '{b}'"))?,
            _ => return Err("provide exactly one of 'number' or 'binary'".into()),
        };
        Ok(serde_json::to_string(&hexagram_info(record)?)?)
    })
}
