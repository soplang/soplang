//! UTC timestamps, calendar parts, and durations measured in seconds.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const DAY: i64 = 86_400;

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E050_TIME),
    )
}

pub fn now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(earlier) => -(earlier.duration().as_secs() as i64),
    }
}

pub fn parse(text: &str) -> Result<i64, SoplangError> {
    let (date, time) = match text.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (text, None),
    };
    let (year, month, day) = parse_date(date)?;
    let (hour, minute, second) = match time {
        None => (0, 0, 0),
        Some(time) => parse_time(time)?,
    };
    from_fields(year, month, day, hour, minute, second)
}

pub fn format_utc(seconds: i64) -> Result<String, SoplangError> {
    let parts = fields(seconds)?;
    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        parts.year, parts.month, parts.day, parts.hour, parts.minute, parts.second
    ))
}

pub fn parts(seconds: i64) -> Result<Value, SoplangError> {
    let parts = fields(seconds)?;
    Ok(object(&[
        ("sanad", parts.year as i64),
        ("bil", parts.month as i64),
        ("maalin", parts.day as i64),
        ("saac", parts.hour as i64),
        ("daqiiqad", parts.minute as i64),
        ("ilbiriqsi", parts.second as i64),
    ]))
}

pub fn from_parts(value: &Value) -> Result<i64, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error("waqti_ka_qayb() wuxuu u baahan yahay walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    from_fields(
        field(&object, "sanad")?,
        field(&object, "bil")?,
        field(&object, "maalin")?,
        field(&object, "saac")?,
        field(&object, "daqiiqad")?,
        field(&object, "ilbiriqsi")?,
    )
}

pub fn add(seconds: i64, delta: i64) -> Result<i64, SoplangError> {
    let sum = seconds
        .checked_add(delta)
        .ok_or_else(|| error("Wadarta waqtiga way buuxsantay."))?;
    fields(sum)?;
    Ok(sum)
}

struct Fields {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
}

fn from_fields(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> Result<i64, SoplangError> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return Err(error("Taariikhdu ma saxna."));
    }
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
        return Err(error("Waqtigu ma saxna."));
    }
    let days = days_from_civil(year, month, day);
    days.checked_mul(DAY)
        .and_then(|day_seconds| day_seconds.checked_add(i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second)))
        .ok_or_else(|| error("Waqtigu waa ka baxsan yahay xadka."))
}

fn fields(seconds: i64) -> Result<Fields, SoplangError> {
    let (days, time) = floor_div(seconds, DAY);
    let days = i32::try_from(days).map_err(|_| error("Taariikhdu waa ka baxsan tahay xadka."))?;
    let (year, month, day) = civil_from_days(days);
    if !(1..=9999).contains(&year) {
        return Err(error("Taariikhdu waa ka baxsan tahay xadka."));
    }
    Ok(Fields {
        year,
        month,
        day,
        hour: (time / 3600) as i32,
        minute: ((time % 3600) / 60) as i32,
        second: (time % 60) as i32,
    })
}

fn parse_date(text: &str) -> Result<(i32, i32, i32), SoplangError> {
    let mut parts = text.split('-');
    let year = number(parts.next(), 4)?;
    let month = number(parts.next(), 2)?;
    let day = number(parts.next(), 2)?;
    if parts.next().is_some() {
        return Err(error("Qaabka taariikhdu ma saxna."));
    }
    Ok((year, month, day))
}

fn parse_time(text: &str) -> Result<(i32, i32, i32), SoplangError> {
    let Some(text) = text.strip_suffix('Z') else {
        return Err(error("Waqtigu waa inuu ku dhammaadaa Z."));
    };
    let mut parts = text.split(':');
    let hour = number(parts.next(), 2)?;
    let minute = number(parts.next(), 2)?;
    let second = number(parts.next(), 2)?;
    if parts.next().is_some() {
        return Err(error("Qaabka waqtigu ma saxna."));
    }
    Ok((hour, minute, second))
}

fn number(text: Option<&str>, width: usize) -> Result<i32, SoplangError> {
    let Some(text) = text else {
        return Err(error("Qaabka taariikhdu ma saxna."));
    };
    if text.len() != width || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error("Qaabka taariikhdu ma saxna."));
    }
    text.parse().map_err(|_| error("Qaabka taariikhdu ma saxna."))
}

fn field(object: &HashMap<String, Value>, name: &str) -> Result<i32, SoplangError> {
    match object.get(name) {
        Some(Value::Int(value)) => i32::try_from(*value).map_err(|_| error("Qaybta waqtigu ma saxna.")),
        _ => Err(error(format!("Qaybta '{name}' waa inay noqotaa abn."))),
    }
}

fn object(fields: &[(&str, i64)]) -> Value {
    let mut map = HashMap::new();
    for (name, value) in fields {
        map.insert((*name).to_string(), Value::Int(*value));
    }
    Value::Object(Rc::new(RefCell::new(map)))
}

fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn floor_div(value: i64, unit: i64) -> (i64, i64) {
    let mut quotient = value / unit;
    let mut remainder = value % unit;
    if remainder < 0 {
        quotient -= 1;
        remainder += unit;
    }
    (quotient, remainder)
}

/// Days since 1970-01-01. Howard Hinnant's civil-date algorithm.
fn days_from_civil(mut year: i32, month: i32, day: i32) -> i64 {
    year -= i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year as u32;
    i64::from(era) * 146097 + i64::from(day_of_era) - 719468
}

fn civil_from_days(days: i32) -> (i32, i32, i32) {
    let shifted = days as i64 + 719468;
    let era = if shifted >= 0 { shifted } else { shifted - 146096 } / 146097;
    let day_of_era = (shifted - era * 146097) as u32;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let year = year_of_era as i32 + era as i32 * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = if month_part < 10 { month_part + 3 } else { month_part - 9 };
    (year + i32::from(month <= 2), month as i32, day as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_and_known_utc_instant_round_trip() {
        assert_eq!(format_utc(0).unwrap(), "1970-01-01T00:00:00Z");
        assert_eq!(parse("1970-01-01").unwrap(), 0);
        assert_eq!(parse("2026-10-21T08:12:00Z").unwrap(), 1_792_570_320);
        assert_eq!(format_utc(1_792_570_320).unwrap(), "2026-10-21T08:12:00Z");
        assert_eq!(format_utc(parse("2024-02-29").unwrap()).unwrap(), "2024-02-29T00:00:00Z");
        assert!(parse("2023-02-29").is_err());
        assert_eq!(add(0, DAY).unwrap(), DAY);
    }
}
