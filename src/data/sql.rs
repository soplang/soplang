//! SQL text contains identifiers and placeholders; application values are bound.

use super::{schema::Table, ColumnType, DriverKind, SqlValue};
use std::collections::HashMap;

pub(super) struct Statement {
    pub text: String,
    pub params: Vec<Param>,
}

pub(super) struct Param {
    ty: ColumnType,
    value: SqlValue,
}

impl Param {
    pub(super) fn text(value: String) -> Self {
        Self {
            ty: ColumnType::Text,
            value: SqlValue::Text(value),
        }
    }

    pub fn sqlite(&self) -> rusqlite::types::Value {
        use rusqlite::types::Value as V;
        match &self.value {
            SqlValue::Null => V::Null,
            SqlValue::Int(v) => V::Integer(*v),
            SqlValue::Float(v) => V::Real(*v),
            SqlValue::Text(v) => V::Text(v.clone()),
            SqlValue::Bool(v) => V::Integer(i64::from(*v)),
        }
    }

    pub fn mysql(&self) -> mysql::Value {
        use mysql::Value as V;
        match &self.value {
            SqlValue::Null => V::NULL,
            SqlValue::Int(v) => V::Int(*v),
            SqlValue::Float(v) => V::Double(*v),
            SqlValue::Text(v) => V::Bytes(v.as_bytes().to_vec()),
            SqlValue::Bool(v) => V::Int(i64::from(*v)),
        }
    }

    pub fn postgres(&self) -> Box<dyn postgres::types::ToSql + Sync + Send> {
        match &self.value {
            SqlValue::Int(v) => Box::new(*v),
            SqlValue::Float(v) => Box::new(*v),
            SqlValue::Text(v) => Box::new(v.clone()),
            SqlValue::Bool(v) => Box::new(*v),
            SqlValue::Null => match self.ty {
                ColumnType::Int => Box::new(Option::<i64>::None),
                ColumnType::Float => Box::new(Option::<f64>::None),
                ColumnType::Text => Box::new(Option::<String>::None),
                ColumnType::Bool => Box::new(Option::<bool>::None),
            },
        }
    }

    pub fn sqlserver(&self) -> Box<dyn tiberius::ToSql> {
        match &self.value {
            SqlValue::Int(v) => Box::new(*v),
            SqlValue::Float(v) => Box::new(*v),
            SqlValue::Text(v) => Box::new(v.clone()),
            SqlValue::Bool(v) => Box::new(*v),
            SqlValue::Null => match self.ty {
                ColumnType::Int => Box::new(Option::<i64>::None),
                ColumnType::Float => Box::new(Option::<f64>::None),
                ColumnType::Text => Box::new(Option::<String>::None),
                ColumnType::Bool => Box::new(Option::<bool>::None),
            },
        }
    }
}

pub(super) fn identifier(kind: DriverKind, name: &str) -> String {
    match kind {
        DriverKind::Mysql | DriverKind::Mariadb => format!("`{}`", name.replace('`', "``")),
        DriverKind::SqlServer => format!("[{}]", name.replace(']', "]]")),
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

fn parameter(
    kind: DriverKind,
    column: &super::schema::Column,
    value: &SqlValue,
    params: &mut Vec<Param>,
) -> String {
    params.push(Param {
        ty: column.ty,
        value: value.clone(),
    });
    match kind {
        DriverKind::Postgres | DriverKind::Cockroach => format!("${}", params.len()),
        DriverKind::SqlServer => format!("@P{}", params.len()),
        _ => "?".into(),
    }
}

fn predicate(
    kind: DriverKind,
    table: &Table,
    filter: &HashMap<String, SqlValue>,
    params: &mut Vec<Param>,
) -> String {
    let fields = table
        .columns
        .iter()
        .filter_map(|column| {
            filter.get(&column.name).map(|value| {
                let name = identifier(kind, &column.name);
                if matches!(value, SqlValue::Null) {
                    format!("{name} IS NULL")
                } else {
                    format!("{name} = {}", parameter(kind, column, value, params))
                }
            })
        })
        .collect::<Vec<_>>();
    if fields.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", fields.join(" AND "))
    }
}

pub(super) fn insert(
    kind: DriverKind,
    table: &Table,
    fields: &HashMap<String, SqlValue>,
) -> Statement {
    let mut params = Vec::new();
    let mut names = Vec::new();
    let mut values = Vec::new();
    for column in &table.columns {
        if let Some(value) = fields.get(&column.name) {
            names.push(identifier(kind, &column.name));
            values.push(parameter(kind, column, value, &mut params));
        }
    }
    let table = identifier(kind, &table.name);
    let text = if names.is_empty() {
        if matches!(kind, DriverKind::Mysql | DriverKind::Mariadb) {
            format!("INSERT INTO {table} () VALUES ()")
        } else {
            format!("INSERT INTO {table} DEFAULT VALUES")
        }
    } else {
        format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            names.join(", "),
            values.join(", ")
        )
    };
    Statement { text, params }
}

pub(super) fn select(
    kind: DriverKind,
    table: &Table,
    filter: &HashMap<String, SqlValue>,
) -> Statement {
    let mut params = Vec::new();
    let clause = predicate(kind, table, filter, &mut params);
    let columns = table
        .columns
        .iter()
        .map(|column| identifier(kind, &column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let table = identifier(kind, &table.name);
    let text = if matches!(kind, DriverKind::SqlServer) {
        format!("SELECT TOP 1000 {columns} FROM {table}{clause}")
    } else {
        format!("SELECT {columns} FROM {table}{clause} LIMIT 1000")
    };
    Statement { text, params }
}

pub(super) fn update(
    kind: DriverKind,
    table: &Table,
    filter: &HashMap<String, SqlValue>,
    changes: &HashMap<String, SqlValue>,
) -> Statement {
    let mut params = Vec::new();
    let values = table
        .columns
        .iter()
        .filter_map(|column| {
            changes.get(&column.name).map(|value| {
                format!(
                    "{} = {}",
                    identifier(kind, &column.name),
                    parameter(kind, column, value, &mut params)
                )
            })
        })
        .collect::<Vec<_>>();
    let clause = predicate(kind, table, filter, &mut params);
    Statement {
        text: format!(
            "UPDATE {} SET {}{clause}",
            identifier(kind, &table.name),
            values.join(", ")
        ),
        params,
    }
}

pub(super) fn delete(
    kind: DriverKind,
    table: &Table,
    filter: &HashMap<String, SqlValue>,
) -> Statement {
    let mut params = Vec::new();
    let clause = predicate(kind, table, filter, &mut params);
    Statement {
        text: format!("DELETE FROM {}{clause}", identifier(kind, &table.name)),
        params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_bound_and_null_filters_use_is_null() {
        let table =
            super::super::parse_schema("jadwal select {\n id abn furaha\n name qoraal?\n}\n")
                .unwrap()
                .tables
                .remove(0);
        let hostile = "x\\' OR 1=1 -- ";
        let fields = HashMap::from([("name".into(), SqlValue::Text(hostile.into()))]);
        for kind in [
            DriverKind::Sqlite,
            DriverKind::Postgres,
            DriverKind::Cockroach,
            DriverKind::Mysql,
            DriverKind::Mariadb,
            DriverKind::SqlServer,
        ] {
            let statement = insert(kind, &table, &fields);
            assert!(!statement.text.contains(hostile));
            assert_eq!(statement.params.len(), 1);
            assert!(
                matches!(&statement.params[0].value, SqlValue::Text(value) if value == hostile)
            );
            let statement = select(
                kind,
                &table,
                &HashMap::from([("name".into(), SqlValue::Null)]),
            );
            assert!(statement.text.contains("IS NULL"));
            assert!(statement.params.is_empty());
        }
    }
}
