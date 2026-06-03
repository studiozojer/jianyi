use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jianyi"))
}

#[test]
fn cast_with_seed_is_deterministic() {
    let run = || {
        let out = bin()
            .args(["iching", "cast", "--seed", "2026-06-02T11:00:00+00:00"])
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    assert_eq!(run(), run());
}

#[test]
fn hexagram_json_has_rendering_pair() {
    let out = bin().args(["--json", "iching", "hexagram", "1"]).output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["judgment"]["chinese"].is_string());
    assert!(v["judgment"]["english"].is_string());
    assert!(v["hexagram_reading"].is_null());
}

#[test]
fn hexagram_out_of_range_is_usage_error() {
    let out = bin().args(["iching", "hexagram", "99"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2)); // Error::Usage exit code
}

#[test]
fn tarot_draw_seeded_is_deterministic() {
    let run = || {
        let out = bin()
            .args(["--json", "tarot", "draw", "-n", "5", "--seed", "2026-06-02T11:00:00+00:00"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    assert_eq!(run(), run());
}
