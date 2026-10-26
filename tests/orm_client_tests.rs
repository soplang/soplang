//! Generated client contracts work as source modules in both execution paths.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct Tree(PathBuf);
impl Tree {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "soplang_orm_client_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SCHEMA: &str = "jadwal qof {\n id abn furaha\n magac qoraal\n da abn?\n firfircoon bool\n miisaan jajab\n}\njadwal qor {\n id abn furaha\n hawl qoraal\n}\n";
const PROGRAM: &str = r#"
keen "client.sop" sida c
c.MacmiilXog db = cusub c.MacmiilXog("sqlite", ":memory:", 2)
db.guuri()
hubi(db.qof.geli(cusub c.QofGelin(magac: "Amina", firfircoon: run, miisaan: 1.5, da: 22)) == 1)
hubi(db.qof.geli(cusub c.QofGelin("Cali", been, 2.0)) == 1)
teed[c.Qof] rows = db.qof.raadi(cusub c.QofShaandhe().halkaMagac("Amina"))
hubi(dherer(rows) == 1 && rows[0].helMagac() == "Amina")
hubi(rows[0].helDa() == 22 && rows[0].helFirfircoon() == run && rows[0].helMiisaan() == 1.5)
walax copied = rows[0].xog()
copied.magac = "Changed"
hubi(rows[0].helMagac() == "Amina")
hubi(json_akhris(json_qor(rows[0].xog())).magac == "Amina")
c.Qof? missing = db.qof.mid(cusub c.QofShaandhe().halkaId(999))
hubi(missing == maran)
hubi(dherer(db.qof.raadi(cusub c.QofShaandhe().halkaDa(maran))) == 1)
hubi(db.qof.beddel(cusub c.QofShaandhe().halkaMagac("Cali"), cusub c.QofBeddel().dhigDa(30).dhigFirfircoon(run)) == 1)
hubi(db.qof.mid(cusub c.QofShaandhe().halkaMagac("Cali")).helDa() == 30)
hubi(db.jadwalQor().geli(cusub c.QorGelin("reserved name")) == 1)
hubi(db.jadwalQor().raadi()[0].helHawl() == "reserved name")
abn errors = 0
tijaabi {
    db.macaamil(hawl(): maran {
        db.qof.geli(cusub c.QofGelin("rollback", run, 9.0))
        tuur "rollback"
    })
} qabo (e) { errors = errors + 1 }
hubi(dherer(db.qof.raadi(cusub c.QofShaandhe().halkaMagac("rollback"))) == 0)
db.macaamil(hawl(): maran { db.qof.geli(cusub c.QofGelin("committed", run, 8.0)) })
hubi(dherer(db.qof.raadi()) == 3)
tijaabi { db.qof.tirtir(cusub c.QofShaandhe()) } qabo (e, details) {
    hubi(details.summad == "E059")
    errors = errors + 1
}
walax wrong = { value: 7 }
tijaabi { db.qof.geli(wrong.value) } qabo (e) { errors = errors + 1 }
tijaabi { (cusub c.QofShaandhe()).halkaMagac(wrong.value) } qabo (e) { errors = errors + 1 }
hubi(errors == 4)
hubi(db.qof.tirtir(cusub c.QofShaandhe().halkaMagac("Amina")) == 1)
db.guuri()
hubi(dherer(db.qof.raadi()) == 2)
hubi(db.qof.tirtirDhammaan() == 2)
hubi(dherer(db.qof.raadi()) == 0)
db.xidh()
db.xidh()
qor("orm client ok")
"#;

#[test]
fn generated_clients_compile_strictly_and_execute_through_jit_and_llvm() {
    let tree = Tree::new();
    let schema = soplang::data::parse_schema(SCHEMA).unwrap();
    let generated = soplang::data::generate_client(&schema).unwrap();
    assert_eq!(generated, soplang::data::generate_client(&schema).unwrap());
    fs::write(tree.0.join("schema.sop"), SCHEMA).unwrap();
    fs::write(tree.0.join("client.sop"), generated).unwrap();
    let main = tree.0.join("main.sop");
    fs::write(&main, PROGRAM).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("--strict")
        .arg(&main)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"orm client ok\n");
    let binary = tree
        .0
        .join(format!("client-bin{}", std::env::consts::EXE_SUFFIX));
    for opt in [0, 2] {
        soplang::build_source(PROGRAM, Some(&main), &binary, opt, true).unwrap();
        let output = Command::new(&binary).output().unwrap();
        assert!(
            output.status.success(),
            "opt {opt}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"orm client ok\n");
    }
}

#[test]
fn generation_does_not_connect_and_only_overwrites_generated_files() {
    let tree = Tree::new();
    let schema = tree.0.join("schema.sop");
    let client = tree.0.join("client.sop");
    fs::write(&schema, SCHEMA).unwrap();
    let generate = || {
        Command::new(env!("CARGO_BIN_EXE_soplang"))
            .args(["xog", "abuur"])
            .arg(&schema)
            .arg("--qor")
            .arg(&client)
            .output()
            .unwrap()
    };
    fs::write(&client, "door original = 7\n").unwrap();
    assert!(!generate().status.success());
    assert_eq!(fs::read_to_string(&client).unwrap(), "door original = 7\n");
    fs::remove_file(&client).unwrap();
    let output = generate();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = fs::read_to_string(&client).unwrap();
    assert!(generate().status.success());
    assert_eq!(fs::read_to_string(&client).unwrap(), first);
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["xog", "abuur"])
        .arg(&schema)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), first);
    let check = || {
        Command::new(env!("CARGO_BIN_EXE_soplang"))
            .args(["xog", "abuur"])
            .arg(&schema)
            .arg("--hubi")
            .arg(&client)
            .output()
            .unwrap()
    };
    let before = fs::metadata(&client).unwrap().modified().unwrap();
    assert!(check().status.success());
    assert_eq!(fs::metadata(&client).unwrap().modified().unwrap(), before);
    fs::write(
        &schema,
        SCHEMA.replace("da abn?", "da abn?\n city qoraal? # inline comment"),
    )
    .unwrap();
    assert!(!check().status.success());
    assert_eq!(fs::read_to_string(&client).unwrap(), first);
    let mut invalid = soplang::data::parse_schema(SCHEMA).unwrap();
    invalid.tables[0].columns.clear();
    assert!(soplang::data::generate_client(&invalid).is_err());
}

#[test]
fn generated_models_expose_query_and_record_types_to_the_editor() {
    use serde_json::json;
    let tree = Tree::new();
    let generated =
        soplang::data::generate_client(&soplang::data::parse_schema(SCHEMA).unwrap()).unwrap();
    fs::write(tree.0.join("client.sop"), generated).unwrap();
    let main = tree.0.join("main.sop");
    let uri = url::Url::from_file_path(&main).unwrap().to_string();
    let source = "keen \"client.sop\" sida c\ndoor db = cusub c.MacmiilXog(\"sqlite\", \":memory:\")\ndb.qof.";
    let mut server = soplang::cli::lsp::Server::default();
    server.handle(&json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"languageId":"soplang","text":source}}}).to_string());
    let result = server.handle(&json!({"jsonrpc":"2.0","id":1,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":2,"character":7}}}).to_string());
    let items = result[0]["result"].as_array().unwrap();
    for method in ["geli", "raadi", "mid", "beddel", "tirtir"] {
        assert!(
            items.iter().any(|item| item["label"] == method),
            "{result:?}"
        );
    }
    assert!(
        items.iter().find(|item| item["label"] == "geli").unwrap()["detail"]
            .as_str()
            .unwrap()
            .contains("QofGelin")
    );
    let source =
        "keen \"client.sop\" sida c\nc.QofShaandhe filter = cusub c.QofShaandhe()\nfilter.";
    server.handle(&json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":source}]}}).to_string());
    let result = server.handle(&json!({"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":2,"character":7}}}).to_string());
    assert!(
        result[0]["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "halkaMagac"
                && item["detail"].as_str().unwrap().contains("qoraal")),
        "{result:?}"
    );
}
