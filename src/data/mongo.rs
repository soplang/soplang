//! Schema-bound MongoDB rows and integer key allocation.
use super::*;

pub(super) fn projection(table: &schema::Table) -> Document {
    let mut fields: Document = table
        .columns
        .iter()
        .map(|column| (column.name.clone(), Bson::Int32(1)))
        .collect();
    if !fields.contains_key("_id") {
        fields.insert("_id", 0);
    }
    fields
}

pub(super) fn decode_row(
    mut document: Document,
    table: &schema::Table,
) -> Result<Value, SoplangError> {
    let mut fields = HashMap::new();
    for column in &table.columns {
        let raw = match document.remove(&column.name) {
            Some(value) => value,
            None if column.optional => Bson::Null,
            None => {
                return Err(data_error(format!(
                    "Tiirka '{}.{}' waa ka maqan yahay xogta.",
                    table.name, column.name
                )))
            }
        };
        let value = match raw {
            Bson::Null => SqlValue::Null,
            Bson::Int32(n) => SqlValue::Int(i64::from(n)),
            Bson::Int64(n) => SqlValue::Int(n),
            Bson::Double(n) if n.is_finite() => SqlValue::Float(n),
            Bson::String(s) => SqlValue::Text(s),
            Bson::Boolean(b) => SqlValue::Bool(b),
            _ => {
                return Err(data_error(format!(
                    "Qiimaha tiirka '{}.{}' ma waafaqsana noociisa.",
                    table.name, column.name
                )))
            }
        };
        fields.insert(column.name.clone(), value);
    }
    validate_fields(table, &mut fields, false)?;
    Ok(object_from(
        fields
            .iter()
            .map(|(name, value)| (name.clone(), runtime_value(value)))
            .collect(),
    ))
}

/// Reconcile the counter with the indexed maximum before reserving an ID.
/// $max never lowers a concurrently advanced counter; $inc reserves uniquely.
pub(super) fn baseline_counter(
    mongo: &mut MongoHandle,
    table: &schema::Table,
) -> Result<(), SoplangError> {
    let key = table.columns.iter().find(|column| column.key).unwrap();
    let collection = mongo.database.collection::<Document>(&table.name);
    let mut fields = doc! { &key.name: 1 };
    if key.name != "_id" {
        fields.insert("_id", 0);
    }
    let find = collection
        .find_one(doc! {})
        .sort(doc! { &key.name: -1 })
        .projection(fields);
    let last = if let Some(session) = mongo.session.as_mut() {
        find.session(session).run()
    } else {
        find.run()
    }
    .map_err(driver_error)?;
    let largest = match last.as_ref().and_then(|row| row.get(&key.name)) {
        Some(Bson::Int32(n)) => i64::from(*n),
        Some(Bson::Int64(n)) => *n,
        None if last.is_none() => 0,
        _ => return Err(data_error("Furaha ku jira xogta ma aha abn sax ah.")),
    }
    .max(0);
    let counters = mongo.database.collection::<Document>("soplang_counters");
    let action = counters
        .update_one(
            doc! { "_id": &table.name },
            doc! { "$max": { "value": largest } },
        )
        .upsert(true);
    if let Some(session) = mongo.session.as_mut() {
        action.session(session).run()
    } else {
        action.run()
    }
    .map_err(driver_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table() -> schema::Table {
        parse_schema("jadwal t {\n _id abn furaha\n amount jajab\n note qoraal?\n}\n")
            .unwrap()
            .tables
            .remove(0)
    }
    #[test]
    fn rows_preserve_declared_ids_project_fields_and_normalize_optional_numbers() {
        let value = decode_row(
            doc! {"_id": 7_i32, "amount": 2_i32, "extra": [1, 2]},
            &table(),
        )
        .unwrap();
        let Value::Object(fields) = value else {
            panic!("object")
        };
        let fields = fields.borrow();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields.get("_id"), Some(&Value::Int(7)));
        assert_eq!(fields.get("amount"), Some(&Value::Float(2.0)));
        assert_eq!(fields.get("note"), Some(&Value::Null));
        assert_eq!(projection(&table()).get_i32("_id").unwrap(), 1);
        let mut ordinary = table();
        ordinary.columns[0].name = "id".into();
        assert_eq!(projection(&ordinary).get_i32("_id").unwrap(), 0);
        let result = decode_row(
            doc! {"_id": mongodb::bson::oid::ObjectId::new(), "id": 1_i64, "amount": 1.0},
            &ordinary,
        )
        .unwrap();
        let Value::Object(fields) = result else {
            panic!("object")
        };
        assert!(!fields.borrow().contains_key("_id"));
    }
    #[test]
    fn reads_reject_missing_required_fields_wrong_types_and_nonfinite_numbers() {
        for document in [
            doc! {"_id": 7_i64},
            doc! {"_id": 7_i64, "amount": Bson::Null},
            doc! {"_id": 7_i64, "amount": "bad"},
            doc! {"_id": 7_i64, "amount": f64::NAN},
            doc! {"_id": 7_i64, "amount": f64::INFINITY},
            doc! {"_id": 7_i64, "amount": 2.0, "note": {"nested": 1}},
        ] {
            assert!(decode_row(document, &table()).is_err());
        }
    }
}
