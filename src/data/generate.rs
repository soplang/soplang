//! Deterministic, typed Soplang clients generated from the database schema.

use std::fmt::Write;

use super::schema::{Column, ColumnType, Schema, Table};
use crate::frontend::{Lexer, TokenType};

pub const GENERATED_HEADER: &str =
    "// Generated Soplang data client. Edit schema.sop and regenerate.";

fn title(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|ch| format!("{}{}", ch.to_ascii_uppercase(), chars.as_str()))
        .unwrap_or_default()
}

fn annotation(column: &Column) -> String {
    let ty = match column.ty {
        ColumnType::Int => "abn",
        ColumnType::Float => "jajab",
        ColumnType::Text => "qoraal",
        ColumnType::Bool => "bool",
    };
    format!("{ty}{}", if column.optional { "?" } else { "" })
}

fn identifier(name: &str) -> bool {
    Lexer::new(name)
        .tokenize()
        .ok()
        .is_some_and(|tokens| tokens.len() == 2 && tokens[0].kind == TokenType::Identifier)
}

fn argument(column: &Column) -> String {
    if identifier(&column.name) {
        column.name.clone()
    } else {
        format!("qiime{}", title(&column.name))
    }
}

fn literal(text: &str) -> String {
    let mut value = String::from("e\"");
    for ch in text.chars() {
        match ch {
            '"' => value.push_str("\\\""),
            '\\' => value.push_str("\\\\"),
            '\n' => value.push_str("\\n"),
            '\r' => value.push_str("\\r"),
            '\t' => value.push_str("\\t"),
            ch if ch.is_control() => write!(value, "\\u{{{:x}}}", ch as u32).unwrap(),
            ch => value.push(ch),
        }
    }
    value.push('"');
    value
}

fn canonical(schema: &Schema) -> String {
    let mut text = String::new();
    for table in &schema.tables {
        writeln!(text, "jadwal {} {{", table.name).unwrap();
        for column in &table.columns {
            writeln!(
                text,
                "  {} {}{}",
                column.name,
                annotation(column),
                if column.key { " furaha" } else { "" }
            )
            .unwrap();
        }
        text.push_str("}\n");
    }
    text
}

fn parameters(table: &Table, insertion: bool) -> String {
    let mut columns: Vec<_> = table.columns.iter().collect();
    columns.sort_by_key(|column| column.optional || insertion && column.key);
    columns
        .into_iter()
        .map(|column| {
            let optional = column.optional || insertion && column.key;
            let ty = if insertion && column.key {
                "abn?".into()
            } else {
                annotation(column)
            };
            format!(
                "{ty} {}{}",
                argument(column),
                if optional { " = maran" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The generated file has no dependency on a registry, downloads, or runtime
/// source files. Every backend executes the same typed client and native driver.
pub fn generate_client(schema: &Schema) -> Result<String, String> {
    super::parse_schema(&canonical(schema))?;
    if schema.tables.len() > 128
        || schema
            .tables
            .iter()
            .map(|table| table.columns.len())
            .sum::<usize>()
            > 2048
    {
        return Err(
            "Schema aad buu ugu weyn yahay hal macmiil; u qaybi macmiillo kala duwan.".into(),
        );
    }
    let mut out = format!(
        "{GENERATED_HEADER}\n// sha256: {}\n\n",
        schema.fingerprint()
    );
    let mut public = vec!["MacmiilXog".to_string()];
    for table in &schema.tables {
        let row = title(&table.name);
        public.extend([
            row.clone(),
            format!("{row}Gelin"),
            format!("{row}Shaandhe"),
            format!("{row}Beddel"),
            format!("{row}Jadwal"),
        ]);
    }
    writeln!(out, "soosaar {}\n", public.join(", ")).unwrap();
    writeln!(
        out,
        "madoor qoraal _soplangSchema = {}\n",
        literal(&canonical(schema))
    )
    .unwrap();
    for table in &schema.tables {
        emit_row(&mut out, table);
        emit_input(&mut out, table);
        emit_builder(&mut out, table, false);
        emit_builder(&mut out, table, true);
        emit_model(&mut out, table);
        if out.len() > crate::analysis::Limits::default().max_file_bytes {
            return Err("Macmiilka la dhalinayo wuxuu dhaafay xadka hal fayl; u qaybi schema macmiillo kala duwan.".into());
        }
    }
    emit_client(&mut out, schema);
    if out.len() > crate::analysis::Limits::default().max_file_bytes {
        return Err("Macmiilka la dhalinayo aad buu u weyn yahay.".into());
    }
    // Fail generation before writing if a language/schema change makes this
    // source invalid, including in strict mode. No user code is executed.
    let stmts = crate::frontend::parse_source(&out, None)
        .map_err(|error| format!("Macmiilka xogta lama dhalin karin: {}", error.message()))?;
    crate::semantic::analyze_with_options(
        &stmts,
        crate::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .map_err(|error| {
        format!(
            "Noocyada macmiilka xogta ma waafaqsana: {}",
            error.message()
        )
    })?;
    Ok(out)
}

fn emit_row(out: &mut String, table: &Table) {
    let row = title(&table.name);
    writeln!(
        out,
        "qaab {row} {{\n    hawl dhis({}): maran {{",
        parameters(table, false)
    )
    .unwrap();
    let fields: Vec<_> = table
        .columns
        .iter()
        .map(|column| format!("{}: {}", literal(&column.name), argument(column)))
        .collect();
    writeln!(
        out,
        "        nafta._xog = {{ {} }}\n    }}",
        fields.join(", ")
    )
    .unwrap();
    for column in &table.columns {
        writeln!(
            out,
            "    hawl hel{}(): {} {{ celi nafta._xog[{}] }}",
            title(&column.name),
            annotation(column),
            literal(&column.name)
        )
        .unwrap();
    }
    out.push_str("    hawl xog(): walax { celi nafta._xog.nuqul() }\n}\n\n");
    let args: Vec<_> = table
        .columns
        .iter()
        .map(|column| format!("{}: xog[{}]", argument(column), literal(&column.name)))
        .collect();
    writeln!(
        out,
        "hawl _soplangRow{row}(walax xog): {row} {{\n    celi cusub {row}({})\n}}\n",
        args.join(", ")
    )
    .unwrap();
}

fn emit_input(out: &mut String, table: &Table) {
    let row = title(&table.name);
    writeln!(
        out,
        "qaab {row}Gelin {{\n    hawl dhis({}): maran {{",
        parameters(table, true)
    )
    .unwrap();
    let fields: Vec<_> = table
        .columns
        .iter()
        .filter(|column| !column.key)
        .map(|column| format!("{}: {}", literal(&column.name), argument(column)))
        .collect();
    writeln!(out, "        nafta._xog = {{ {} }}", fields.join(", ")).unwrap();
    let key = table.columns.iter().find(|column| column.key).unwrap();
    writeln!(
        out,
        "        haddii ({} != maran) {{ nafta._xog[{}] = {} }}\n    }}\n}}\n",
        argument(key),
        literal(&key.name),
        argument(key)
    )
    .unwrap();
}

fn emit_builder(out: &mut String, table: &Table, update: bool) {
    let row = title(&table.name);
    let class = format!("{row}{}", if update { "Beddel" } else { "Shaandhe" });
    writeln!(
        out,
        "qaab {class} {{\n    hawl dhis(): maran {{ nafta._xog = {{}} }}"
    )
    .unwrap();
    for column in table.columns.iter().filter(|column| !update || !column.key) {
        writeln!(out, "    hawl {}{}({} qiime): {class} {{\n        nafta._xog[{}] = qiime\n        celi nafta\n    }}", if update { "dhig" } else { "halka" }, title(&column.name), annotation(column), literal(&column.name)).unwrap();
    }
    out.push_str("}\n\n");
}

fn emit_model(out: &mut String, table: &Table) {
    let row = title(&table.name);
    let name = literal(&table.name);
    writeln!(
        out,
        "qaab {row}Jadwal {{\n    hawl dhis(walax db): maran {{ nafta._db = db }}"
    )
    .unwrap();
    writeln!(
        out,
        "    hawl geli({row}Gelin xog): abn {{ celi xog_geli(nafta._db, {name}, xog._xog) }}"
    )
    .unwrap();
    writeln!(out, "    hawl raadi({row}Shaandhe? shaandhe = maran): teed[{row}] {{\n        walax shuruud = {{}}\n        haddii (shaandhe != maran) {{ shuruud = shaandhe._xog }}\n        teed[walax] xog = xog_raadi(nafta._db, {name}, shuruud)\n        teed[{row}] safaf = []\n        kuceli (saf ku xog) {{ safaf.geli(_soplangRow{row}(saf)) }}\n        celi safaf\n    }}").unwrap();
    writeln!(out, "    hawl mid({row}Shaandhe shaandhe): {row}? {{\n        teed[{row}] safaf = nafta.raadi(shaandhe)\n        haddii (dherer(safaf) == 0) {{ celi maran }}\n        celi safaf[0]\n    }}").unwrap();
    for update in [true, false] {
        let method = if update { "beddel" } else { "tirtir" };
        let input = if update {
            format!(", {row}Beddel xog")
        } else {
            String::new()
        };
        let data = if update { ", xog._xog" } else { "" };
        writeln!(out, "    hawl {method}({row}Shaandhe shaandhe{input}): abn {{\n        haddii (dherer(shaandhe._xog) == 0) {{ tuur khalad(\"Shaandhe madhan lama oggola; adeegso {method}Dhammaan.\", \"E059\") }}\n        celi xog_{method}(nafta._db, {name}, shaandhe._xog{data})\n    }}").unwrap();
        let input = if update {
            format!("{row}Beddel xog")
        } else {
            String::new()
        };
        writeln!(out, "    hawl {method}Dhammaan({input}): abn {{ celi xog_{method}(nafta._db, {name}, {{}}{data}) }}").unwrap();
    }
    out.push_str("}\n\n");
}

fn emit_client(out: &mut String, schema: &Schema) {
    out.push_str("qaab MacmiilXog {\n    hawl dhis(qoraal nooc, qoraal jid, abn wadarta = 1): maran {\n        nafta._db = xog_fur(nooc, jid, { wadarta: wadarta })\n        nafta._xidhan = been\n        tijaabi {\n            xog_schema(nafta._db, _soplangSchema)\n");
    for table in &schema.tables {
        let row = title(&table.name);
        writeln!(
            out,
            "            nafta._jadwal{row} = cusub {row}Jadwal(nafta._db)"
        )
        .unwrap();
        if identifier(&table.name)
            && !matches!(
                table.name.as_str(),
                "guuri" | "xidh" | "macaamil" | "xidhiidh" | "sha256"
            )
        {
            writeln!(out, "            nafta.{} = nafta._jadwal{row}", table.name).unwrap();
        }
    }
    out.push_str("        } qabo (fariin, xog) {\n            xog_xidh(nafta._db)\n            nafta._xidhan = run\n            tuur xog\n        }\n    }\n");
    for table in &schema.tables {
        let row = title(&table.name);
        writeln!(
            out,
            "    hawl jadwal{row}(): {row}Jadwal {{ celi nafta._jadwal{row} }}"
        )
        .unwrap();
    }
    out.push_str("    hawl guuri(): maran { xog_guuri(nafta._db) }\n    hawl xidhiidh(): walax { celi nafta._db }\n    hawl macaamil(hawl shaqo): maran { xog_macaamil(nafta._db, shaqo) }\n    hawl xidh(): maran {\n        haddii (!nafta._xidhan) { xog_xidh(nafta._db) nafta._xidhan = run }\n    }\n");
    writeln!(
        out,
        "    hawl sha256(): qoraal {{ celi {} }}\n}}",
        literal(&schema.fingerprint())
    )
    .unwrap();
}
