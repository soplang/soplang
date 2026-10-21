//! UTC time parse, format, and duration (3.3 stage).

use std::process::Command;

fn run_program(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg("-c")
        .arg(source)
        .output()
        .expect("run soplang");
    assert!(
        out.status.success(),
        "program failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn utc_parse_format_parts_and_duration() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door instant = waqti_akhris("2026-10-21T08:12:00Z")
hubi(instant == 1792570320, "parse")
hubi(waqti_qor(instant) == "2026-10-21T08:12:00Z", "format")
hubi(waqti_akhris("1970-01-01") == 0, "date")
door parts = waqti_qayb(instant)
hubi(parts.sanad == 2026 && parts.bil == 10 && parts.maalin == 21, "date parts")
hubi(parts.saac == 8 && parts.daqiiqad == 12 && parts.ilbiriqsi == 0, "time parts")
hubi(waqti_ka_qayb(parts) == instant, "from parts")
hubi(waqti_qor(waqti_kudar(0, 86400)) == "1970-01-02T00:00:00Z", "duration")
door now = waqti()
hubi(waqti_akhris(waqti_qor(now)) == now, "now")
qor("time ok")
"#;
    assert_eq!(run_program(source), "time ok\n");
}

#[test]
fn invalid_dates_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { waqti_akhris("2023-02-29") } qabo (e) { failed = run }
hubi(failed, "leap")
failed = been
tijaabi { waqti_akhris("2026-10-21T08:12:00+03:00") } qabo (e) { failed = run }
hubi(failed, "offset")
qor("time errors ok")
"#;
    assert_eq!(run_program(source), "time errors ok\n");
}
