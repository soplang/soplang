use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use soplang::run_source;

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "soplang_async_{name}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[test]
fn file_tasks_read_write_and_append_text_and_bytes() {
    let file = temp_path("content");
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
        sug(qor_fayl_shaqo("{path}", "kow"))
        sug(kudar_fayl_shaqo("{path}", "-laba"))
        hubi(sug(akhrifayl_shaqo("{path}")) == "kow-laba")
        sug(qor_baytyo_shaqo("{path}", baytyo([65, 66])))
        sug(kudar_baytyo_shaqo("{path}", baytyo([67])))
        hubi(dherer(sug(akhrif_baytyo_shaqo("{path}"))) == 3)
        hubi(sug(akhrifayl_shaqo("{path}")) == "ABC")
        "#
    );
    run_source(&source, None, false, false, false).unwrap();
    assert_eq!(fs::read(&file).unwrap(), b"ABC");
    let _ = fs::remove_file(file);
}

#[test]
fn file_tasks_report_errors_and_cancellation() {
    let file = temp_path("errors");
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
        door missing = been
        tijaabi {{ sug(akhrifayl_shaqo("{path}")) }} qabo (e) {{ missing = run }}
        hubi(missing)
        door task = akhrifayl_shaqo("{path}")
        baaji(task)
        door cancelled = been
        tijaabi {{ sug(task) }} qabo (e) {{ cancelled = run }}
        hubi(cancelled)
        "#
    );
    run_source(&source, None, false, false, false).unwrap();

    fs::write(&file, [0xff]).unwrap();
    let source = format!(
        r#"
        door invalid = been
        tijaabi {{ sug(akhrifayl_shaqo("{path}")) }} qabo (e) {{ invalid = run }}
        hubi(invalid)
        hubi(dherer(sug(akhrif_baytyo_shaqo("{path}"))) == 1)
        "#
    );
    run_source(&source, None, false, false, false).unwrap();
    let _ = fs::remove_file(file);
}

#[test]
fn file_tasks_bound_read_size() {
    let file = temp_path("large");
    fs::write(&file, vec![b'x'; 16 * 1024 * 1024 + 1]).unwrap();
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
        door large = been
        tijaabi {{ sug(akhrif_baytyo_shaqo("{path}")) }} qabo (e) {{ large = run }}
        hubi(large)
        "#
    );
    run_source(&source, None, false, false, false).unwrap();
    let _ = fs::remove_file(file);
}
