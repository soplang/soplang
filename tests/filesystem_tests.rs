//! Path and filesystem management builtins (3.3 stage).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn sop_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn run_program_in(source: &str, dir: &Path) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .current_dir(dir)
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
fn filesystem_metadata_paths_and_directory_management() {
    let dir = temp_workdir("fs_manage");
    let nested = dir.join("data").join("nested");
    let file = nested.join("note.txt");
    let root = sop_path(&dir);
    let nested_path = sop_path(&nested);
    let file_path = sop_path(&file);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
hubi(!fayl_jiraa("{file_path}"), "missing file")
abuur_gal("{nested_path}")
qor_fayl("{file_path}", "nux")
door info = fayl_xog("{file_path}")
hubi(info.jir && info.nooc == "fayl" && info.cabbir == 3, "file info")
door joined = isku_xidh("{root}", "data", "nested", "note.txt")
hubi(fayl_jiraa(joined) && akhrifayl(joined) == "nux", "join")
door absolute = baabi_jid("data/nested/note.txt")
hubi(fayl_jiraa(absolute), "absolute")
door names = tir_gal("{nested_path}")
hubi(dherer(names) == 1 && names[0] == "note.txt", "list dir")
door copy_path = isku_xidh("{nested_path}", "copy.txt")
nuqul_fayl("{file_path}", copy_path)
hubi(akhrifayl(copy_path) == "nux", "copy")
wareeji_fayl(copy_path, isku_xidh("{nested_path}", "moved.txt"))
hubi(fayl_jiraa(isku_xidh("{nested_path}", "moved.txt")), "move")
tirtir_fayl("{file_path}")
tirtir_fayl(isku_xidh("{nested_path}", "moved.txt"))
tirtir_gal("{nested_path}")
tirtir_gal(isku_xidh("{root}", "data"), run)
qor("filesystem ok")
"#
    );
    assert_eq!(run_program_in(&source, &dir), "filesystem ok\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn copy_without_overwrite_is_catchable() {
    let dir = temp_workdir("fs_copy_guard");
    let file = dir.join("one.txt");
    fs::write(&file, "a").unwrap();
    let other = dir.join("two.txt");
    fs::write(&other, "b").unwrap();
    let one = sop_path(&file);
    let two = sop_path(&other);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door failed = been
tijaabi {{
    nuqul_fayl("{one}", "{two}")
}} qabo (e) {{
    failed = run
}}
hubi(failed, "exists")
qor("copy guard ok")
"#
    );
    assert_eq!(run_program_in(&source, &dir), "copy guard ok\n");
    let _ = fs::remove_dir_all(&dir);
}
