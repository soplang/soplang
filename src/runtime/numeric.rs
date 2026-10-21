//! Numeric functions and context-local reproducible random streams.
use std::cell::Cell;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::context::RuntimeLocal;
use super::Value;
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};
const RANDOM: RuntimeLocal<Cell<Option<u64>>> = RuntimeLocal::new(|state| &state.random);
const STEP: u64 = 0x9e3779b97f4a7c15;
static STREAMS: AtomicU64 = AtomicU64::new(0);
fn error(message: impl Into<String>) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E044_NUMERIC),
    )
}
fn number(value: &Value, name: &str) -> Result<f64, SoplangError> {
    match value {
        Value::Int(value) => Ok(*value as f64),
        Value::Float(value) if !value.is_nan() => Ok(*value),
        _ => Err(error(format!(
            "{}() waxay u baahan tahay abn ama jajab aan NaN ahayn.",
            name
        ))),
    }
}
fn compare(a: &Value, b: &Value) -> Ordering {
    super::list_ops::numeric_compare(a, b).expect("validated numeric values")
}
pub(super) fn extreme(args: &[Value], name: &str, minimum: bool) -> Result<Value, SoplangError> {
    let mut best = &args[0];
    number(best, name)?;
    let mut all_int = matches!(best, Value::Int(_));
    for value in &args[1..] {
        number(value, name)?;
        all_int &= matches!(value, Value::Int(_));
        let order = compare(value, best);
        if (minimum && order == Ordering::Less) || (!minimum && order == Ordering::Greater) {
            best = value;
        }
    }
    if all_int {
        Ok(best.clone())
    } else {
        Ok(Value::Float(number(best, name)?))
    }
}
pub(super) fn logarithm(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(&args, "loog", 1, 2)?;
    let value = number(&args[0], "loog")?;
    if value <= 0.0 {
        return Err(error("loog() qiimuhu waa inuu ka weyn yahay eber."));
    }
    let result = match args.get(1) {
        None => value.ln(),
        Some(base) => {
            let base = number(base, "loog")?;
            if !base.is_finite() || base <= 0.0 || base == 1.0 {
                return Err(error(
                    "loog() saldhiggu waa inuu noqdaa tiro togan oo dhammaad leh, kana duwan 1.",
                ));
            }
            value.log(base)
        }
    };
    Ok(Value::Float(result))
}
fn trig(
    args: &[Value],
    name: &str,
    function: fn(f64) -> f64,
    inverse: fn(f64) -> f64,
    bounded: bool,
) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, name, 1, 2)?;
    let value = number(&args[0], name)?;
    let reverse = match args.get(1) {
        None => false,
        Some(Value::Str(mode)) if mode == "rogan" => true,
        _ => {
            return Err(error(format!(
                "{}() doodda labaad waa inay noqotaa 'rogan'.",
                name
            )))
        }
    };
    if (!reverse && !value.is_finite()) || (reverse && bounded && !(-1.0..=1.0).contains(&value)) {
        return Err(error(format!(
            "{}() qiimuhu wuxuu ka baxsan yahay xadka hawsha.",
            name
        )));
    }
    Ok(Value::Float(if reverse {
        inverse(value)
    } else {
        function(value)
    }))
}
pub(super) fn sine(args: Vec<Value>) -> Result<Value, SoplangError> {
    trig(&args, "sin", f64::sin, f64::asin, true)
}
pub(super) fn cosine(args: Vec<Value>) -> Result<Value, SoplangError> {
    trig(&args, "cos", f64::cos, f64::acos, true)
}
pub(super) fn tangent(args: Vec<Value>) -> Result<Value, SoplangError> {
    trig(&args, "tan", f64::tan, f64::atan, false)
}
pub(super) fn angle(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "xagal", 2)?;
    let y = number(&args[0], "xagal")?;
    let x = number(&args[1], "xagal")?;
    if !x.is_finite() || !y.is_finite() {
        return Err(error(
            "xagal() labada qiime waa inay noqdaan tirooyin dhammaad leh.",
        ));
    }
    Ok(Value::Float(y.atan2(x)))
}
pub(super) fn constant(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "joogto", 1)?;
    let Value::Str(name) = &args[0] else {
        return Err(error("joogto() magacu waa inuu noqdaa qoraal."));
    };
    let value = match name.as_str() {
        "pi" => std::f64::consts::PI,
        "e" => std::f64::consts::E,
        "tau" => std::f64::consts::TAU,
        "inf" => f64::INFINITY,
        "nan" => f64::NAN,
        _ => return Err(error(format!("joogto() ma aqoonsana magaca '{}'.", name))),
    };
    Ok(Value::Float(value))
}
pub(super) fn clamp(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "xadid", 3)?;
    for value in &args {
        number(value, "xadid")?;
    }
    if compare(&args[1], &args[2]) == Ordering::Greater {
        return Err(error("xadid() xadka hoose kama weynaan karo xadka sare."));
    }
    Ok(if compare(&args[0], &args[1]) == Ordering::Less {
        args[1].clone()
    } else if compare(&args[0], &args[2]) == Ordering::Greater {
        args[2].clone()
    } else {
        args[0].clone()
    })
}
pub(super) fn seed(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "abuur", 1)?;
    let Value::Int(seed) = args[0] else {
        return Err(error("abuur() abuurku waa inuu noqdaa abn."));
    };
    RANDOM.with(|state| state.set(Some(seed as u64)));
    Ok(Value::Null)
}
// SplitMix64, following Vigna's public-domain reference implementation:
// https://prng.di.unimi.it/splitmix64.c
fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}
fn next() -> u64 {
    RANDOM.with(|state| {
        let initial = state.get().unwrap_or_else(|| {
            let clock = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            mix(clock ^ STREAMS.fetch_add(STEP, AtomicOrdering::Relaxed))
        });
        let value = initial.wrapping_add(STEP);
        state.set(Some(value));
        mix(value)
    })
}
fn fraction() -> f64 {
    (next() >> 11) as f64 * (1.0 / 9007199254740992.0)
}
fn index(bound: u64) -> usize {
    let threshold = bound.wrapping_neg() % bound;
    loop {
        let value = next();
        if value >= threshold {
            return (value % bound) as usize;
        }
    }
}
pub(super) fn random(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(&args, "xul", 0, 2)?;
    match args.as_slice() {
        [] => Ok(Value::Float(fraction())),
        [Value::List(values)] => {
            let values = values
                .try_borrow()
                .map_err(|_| error("Teedka xul() hadda lama akhriyi karo."))?;
            if values.is_empty() {
                return Err(error("teedku waa madhan yahay"));
            }
            Ok(values[index(values.len() as u64)].clone())
        }
        [_] => Err(error("Qiimaha ma ahan teed")),
        [a, b] => {
            let a = number(a, "xul")?;
            let b = number(b, "xul")?;
            if !a.is_finite() || !b.is_finite() {
                return Err(error(
                    "xul() xuduuduhu waa inay noqdaan tirooyin dhammaad leh.",
                ));
            }
            if a > b {
                return Err(error(
                    "Qiimaha koowaad waa in uu ka yar yahay ama la mid yahay qiimaha labaad",
                ));
            }
            if a == b {
                return Ok(Value::Float(a));
            }
            let unit = fraction();
            let value = if a <= 0.0 && b >= 0.0 {
                a * (1.0 - unit) + b * unit
            } else {
                a + (b - a) * unit
            };
            Ok(Value::Float(value.min(b.next_down()).max(a)))
        }
        _ => unreachable!("validated arity"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::RuntimeContext;
    use super::*;

    #[test]
    fn splitmix_matches_reference_words_and_choice_bounds() {
        let context = RuntimeContext::new();
        context.with(|| {
            seed(vec![Value::Int(0)]).unwrap();
            assert_eq!(
                [next(), next(), next()],
                [0xe220a8397b1dcdaf, 0x6e789e6aa1b965f4, 0x06c45d188009454f]
            );
            for bound in [1, 3, 17, (1u64 << 63) + 1, u64::MAX] {
                for _ in 0..1000 {
                    assert!((index(bound) as u64) < bound);
                }
            }
            for _ in 0..1000 {
                assert!((0.0..1.0).contains(&fraction()));
            }
        });
    }

    #[test]
    fn random_ranges_stay_finite_and_exclude_the_upper_endpoint() {
        let context = RuntimeContext::new();
        context.with(|| {
            seed(vec![Value::Int(0)]).unwrap();
            let ranges = [
                (-f64::MAX, f64::MAX),
                (-f64::MAX, -1.0),
                (1.0, f64::MAX),
                (-1.0, 0.0),
                (0.0, f64::from_bits(1)),
                (1.0, 1.0f64.next_up()),
                (-1.0, (-1.0f64).next_up()),
            ];
            for (low, high) in ranges {
                for _ in 0..1000 {
                    let Value::Float(value) =
                        random(vec![Value::Float(low), Value::Float(high)]).unwrap()
                    else {
                        panic!("float")
                    };
                    assert!(
                        value.is_finite() && value >= low && value < high,
                        "{low}..{high}: {value}"
                    );
                }
            }
        });
    }
}
