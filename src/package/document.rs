//! TOML syntax and the manifest/lock schemas. Resolution never parses text itself.

use toml::{Table, Value};

use super::{
    classify_at, manifest_error, validate_language_requirement, validate_name,
    validate_package_version, validate_source, Dependency, LockEntry, Manifest,
};
use crate::error::SoplangError;

fn document(text: &str) -> Result<Table, SoplangError> {
    text.parse::<Table>().map_err(|failure| {
        let mut offset = failure
            .span()
            .map(|span| span.start)
            .unwrap_or(0)
            .min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        let prefix = &text[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        crate::error::runtime_error_ex(
            "Qaabka TOML ee faylka xirmadu ma saxna.",
            line,
            column,
            crate::error::ErrorMeta::default()
                .with_code(crate::error::codes::E057_PACKAGE)
                .with_details(failure.to_string()),
        )
    })
}

fn keys(table: &Table, allowed: &[&str]) -> Result<(), SoplangError> {
    for name in table.keys() {
        if !allowed.contains(&name.as_str()) {
            return Err(manifest_error(format!("Furaha '{name}' lama aqoonsan."), 1));
        }
    }
    Ok(())
}

fn string(table: &Table, key: &str) -> Result<String, SoplangError> {
    let value = table
        .get(key)
        .ok_or_else(|| manifest_error(format!("{key} waa ka maqan yahay xirmada."), 1))?;
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| manifest_error(format!("Qiimaha '{key}' waa inuu noqdaa qoraal."), 1))
}

pub(super) fn manifest(text: &str) -> Result<Manifest, SoplangError> {
    let table = document(text)?;
    keys(
        &table,
        &["magac", "nusxa", "soplang", "gelid", "ku-tiirsan"],
    )?;
    let name = string(&table, "magac")?;
    let version = string(&table, "nusxa")?;
    let soplang = string(&table, "soplang")?;
    let source = if table.contains_key("gelid") {
        string(&table, "gelid")?
    } else {
        ".".into()
    };
    validate_name(&name)?;
    validate_package_version(&version)?;
    validate_language_requirement(&soplang)?;
    validate_source(&source)?;
    let mut dependencies = Vec::new();
    if let Some(value) = table.get("ku-tiirsan") {
        let table = value
            .as_table()
            .ok_or_else(|| manifest_error("ku-tiirsan waa inay noqotaa qayb xirmooyin ah.", 1))?;
        for (name, value) in table {
            validate_name(name)?;
            let spec = match value {
                Value::String(spec) => spec.clone(),
                Value::Table(table) => {
                    if table.contains_key("git") {
                        keys(table, &["git", "rev"])?;
                        let url = string(table, "git")?;
                        let rev = if table.contains_key("rev") {
                            string(table, "rev")?
                        } else {
                            "HEAD".into()
                        };
                        super::git::GitSource::new(&url, &rev)?.spec()
                    } else {
                        keys(table, &["jid"])?;
                        let path = string(table, "jid")?;
                        if !matches!(classify_at(&path, 1)?, super::Spec::Path(_)) {
                            return Err(manifest_error(
                                "jid waa inuu noqdaa jid ku-tiirsan oo maxalli ah.",
                                1,
                            ));
                        }
                        path
                    }
                }
                _ => {
                    return Err(manifest_error(
                        format!("Ku-tiirsiga '{name}' waa inuu noqdaa qoraal ama qayb jid leh."),
                        1,
                    ))
                }
            };
            let spec = match classify_at(&spec, 1)? {
                super::Spec::Git(source) => source.spec(),
                _ => spec,
            };
            dependencies.push(Dependency {
                name: name.clone(),
                spec,
            });
        }
    }
    Ok(Manifest {
        name,
        version,
        soplang,
        source,
        dependencies,
    })
}

pub(super) fn lock(text: &str) -> Result<Vec<LockEntry>, SoplangError> {
    let table = document(text)?;
    keys(&table, &["xirmo"])?;
    let Some(value) = table.get("xirmo") else {
        return Ok(Vec::new());
    };
    let entries = value
        .as_array()
        .ok_or_else(|| manifest_error("xirmo waa inay noqotaa liis qaybo ah.", 1))?;
    let mut result: Vec<LockEntry> = Vec::new();
    for entry in entries {
        let table = entry
            .as_table()
            .ok_or_else(|| manifest_error("Qaybta soplang.lock ma saxna.", 1))?;
        keys(
            table,
            &["magac", "nusxa", "jid", "sumad", "commit", "codsiyo"],
        )?;
        let name = string(table, "magac")?;
        let version = string(table, "nusxa")?;
        let path = string(table, "jid")?;
        validate_name(&name)?;
        validate_package_version(&version)?;
        if path.is_empty() || result.iter().any(|entry| entry.name == name) {
            return Err(manifest_error(
                "soplang.lock wuxuu leeyahay jid madhan ama xirmo lagu celiyey.",
                1,
            ));
        }
        let sumad = if table.contains_key("sumad") {
            let hash = string(table, "sumad")?;
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(manifest_error(
                    "Summadda soplang.lock waa inay noqotaa SHA-256 sax ah.",
                    1,
                ));
            }
            Some(hash)
        } else {
            None
        };
        let commit = if table.contains_key("commit") {
            Some(string(table, "commit")?)
        } else {
            None
        };
        let mut selectors = Vec::new();
        if let Some(value) = table.get("codsiyo") {
            let values = value
                .as_array()
                .ok_or_else(|| manifest_error("codsiyo waa inay noqotaa liis qoraallo ah.", 1))?;
            for value in values {
                let value = value
                    .as_str()
                    .ok_or_else(|| manifest_error("Tixraaca Git waa inuu noqdaa qoraal.", 1))?;
                if selectors.iter().any(|item| item == value) {
                    return Err(manifest_error(
                        "Tixraac Git ayaa lagu celiyey soplang.lock.",
                        1,
                    ));
                }
                selectors.push(value.to_string());
            }
        }
        if path.starts_with("git+") {
            if sumad.is_none()
                || commit
                    .as_deref()
                    .is_none_or(|value| !super::git::valid_oid(value))
                || selectors.is_empty()
            {
                return Err(manifest_error(
                    "soplang.lock wuxuu u baahan yahay kaydinta Git, summad iyo tixraacyo sax ah.",
                    1,
                ));
            }
            for selector in &selectors {
                super::git::GitSource::new(&path[4..], selector)?;
            }
        } else if commit.is_some() || !selectors.is_empty() {
            return Err(manifest_error("Qaybtani ma aha xirmo Git ah.", 1));
        }
        selectors.sort();
        result.push(LockEntry {
            name,
            version,
            path,
            sumad,
            commit,
            selectors,
        });
    }
    result.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(result)
}

pub(super) fn render_manifest(manifest: &Manifest) -> String {
    let mut table = Table::new();
    table.insert("magac".into(), Value::String(manifest.name.clone()));
    table.insert("nusxa".into(), Value::String(manifest.version.clone()));
    table.insert("soplang".into(), Value::String(manifest.soplang.clone()));
    if manifest.source != "." {
        table.insert("gelid".into(), Value::String(manifest.source.clone()));
    }
    if !manifest.dependencies.is_empty() {
        let mut deps = Table::new();
        for dep in &manifest.dependencies {
            deps.insert(dep.name.clone(), Value::String(dep.spec.clone()));
        }
        table.insert("ku-tiirsan".into(), Value::Table(deps));
    }
    // This schema contains only TOML string/table values and cannot fail serialization.
    toml::to_string(&table).expect("manifest schema is serializable")
}

pub(super) fn quote(text: &str) -> String {
    Value::String(text.into()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_toml_strings_comments_and_dependency_tables() {
        let package = manifest(
            r#"
"mag\u0061c" = 'app' # identity
nusxa = """0.1.0"""
soplang = '3'
gelid = 'src\library'
[ku-tiirsan.xisaab]
jid = '../xisaab'
"#,
        )
        .unwrap();
        assert_eq!(package.name, "app");
        assert_eq!(package.source, r"src\library");
        assert_eq!(package.dependencies[0].spec, "../xisaab");
        assert_eq!(manifest(&render_manifest(&package)).unwrap(), package);
    }

    #[test]
    fn rejects_toml_duplicates_and_schema_types() {
        let head = "magac='app'\nnusxa='0.1.0'\nsoplang='3'\n";
        for suffix in [
            "magac='again'",
            "[ku-tiirsan]\n[ku-tiirsan]",
            "ku-tiirsan=[]",
            "gelid=7",
            "[unknown]",
            "[ku-tiirsan]\nxisaab=7",
            "[ku-tiirsan.xisaab]\nunknown='x'",
        ] {
            assert_eq!(
                manifest(&(head.to_owned() + suffix)).unwrap_err().code(),
                Some("E057"),
                "{suffix}"
            );
        }
        let error = manifest("magac='app'\nnusxa='0.1.0'\nsoplang=\"unterminated\n").unwrap_err();
        assert_eq!(error.location().0, 3);
        assert!(error.meta().technical_details.is_some());
        assert!(!error.message().contains("invalid"));
    }

    #[test]
    fn lock_schema_rejects_ambiguity_and_untrusted_integrity_fields() {
        let entry = "[[xirmo]]\nmagac='xisaab'\nnusxa='1.0.0'\njid='../xisaab'\n";
        for text in [
            format!("{entry}{entry}"),
            format!("{entry}sumad='../../outside'"),
            format!("{entry}sumad='{}'", "A".repeat(64)),
            format!("{entry}other='x'"),
            "xirmo='wrong'".into(),
            "[[xirmo]]\nmagac='xisaab'\nnusxa='bad'\njid='../xisaab'".into(),
        ] {
            assert_eq!(lock(&text).unwrap_err().code(), Some("E057"), "{text}");
        }
        let text = format!("# comment\n{entry}sumad='{}' # checked\n", "a".repeat(64));
        let entries = lock(&text).unwrap();
        assert_eq!(
            entries[0].sumad.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }

    #[test]
    fn lock_rendering_round_trips_escaped_paths() {
        let entry = LockEntry {
            name: "xisaab".into(),
            version: "1.0.0".into(),
            path: "../folder\\with\"quotes".into(),
            sumad: None,
            commit: None,
            selectors: Vec::new(),
        };
        let text = super::super::render_lock(&[entry.clone()]);
        assert_eq!(lock(&text).unwrap()[0], entry);
    }
}
