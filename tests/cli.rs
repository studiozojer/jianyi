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
    let out = bin()
        .args(["--json", "iching", "hexagram", "1"])
        .output()
        .unwrap();
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

fn stdout_of(args: &[&str]) -> String {
    let out = bin().args(args).output().unwrap();
    assert!(out.status.success(), "command failed: {args:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn tarot_oneshot_draw_seeded_is_deterministic() {
    let run = || {
        stdout_of(&[
            "--json",
            "tarot",
            "draw",
            "-n",
            "5",
            "--seed",
            "2026-06-02T11:00:00+00:00",
        ])
    };
    assert_eq!(run(), run());
}

#[test]
fn tarot_new_emits_versioned_code() {
    let v: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    assert_eq!(v["count"], 78);
    assert!(v["deck_code"].as_str().unwrap().starts_with("v1."));
}

#[test]
fn tarot_deck_lifecycle_roundtrips() {
    // new -> capture code -> shuffle -> draw top -> the returned code drives the next draw
    let new: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    let code = new["deck_code"].as_str().unwrap().to_string();

    let shuffled: serde_json::Value = serde_json::from_str(&stdout_of(&[
        "--json",
        "tarot",
        "shuffle",
        &code,
        "--seed",
        "2026-06-02T11:00:00+00:00",
    ]))
    .unwrap();
    let scode = shuffled["deck_code"].as_str().unwrap().to_string();

    let draw: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "draw", &scode, "-n", "1"])).unwrap();
    assert_eq!(draw["remaining"], 77);
    assert!(draw["deck_code"].as_str().unwrap().starts_with("v1."));
    assert_eq!(draw["drawn"].as_array().unwrap().len(), 1);
}

#[test]
fn tarot_draw_at_out_of_range_is_usage_error() {
    let new: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    let code = new["deck_code"].as_str().unwrap();
    let out = bin()
        .args(["tarot", "draw", code, "--at", "78"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2)); // Error::Usage
}

#[test]
fn tarot_bad_deck_code_is_usage_error() {
    let out = bin()
        .args(["tarot", "inspect", "not-a-code"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
