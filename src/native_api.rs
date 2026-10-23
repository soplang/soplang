//! Read-only native API declarations for editor services. The registration
//! table supplies global arities; these descriptions never invoke runtime code.
use std::collections::HashMap;
use std::sync::OnceLock;

pub const CURSOR: &str = "@native:cursor";
pub const SOURCE_PATH: &str = "<soplang>/native.sop";

#[derive(Clone, Debug)]
pub struct Signature {
    /// Square brackets mark optional positional arguments; * marks a rest.
    pub parameters: Vec<String>,
}
impl Signature {
    pub fn bounds(&self) -> (usize, usize) {
        let minimum = self
            .parameters
            .iter()
            .filter(|p| !p.starts_with('[') && !p.starts_with('*'))
            .count();
        let maximum = if self.parameters.iter().any(|p| p.starts_with('*')) {
            usize::MAX
        } else {
            self.parameters.len()
        };
        (minimum, maximum)
    }
}

#[derive(Clone, Debug)]
pub struct Api {
    pub id: String,
    pub name: String,
    pub receiver: Option<String>,
    pub signatures: Vec<Signature>,
    pub result: String,
    pub documentation: String,
    pub stride: usize,
    pub start: usize,
    pub end: usize,
}
impl Api {
    pub fn result_for(&self, receiver: &str, arguments: Option<usize>) -> String {
        let receiver = receiver.strip_suffix('?').unwrap_or(receiver);
        let element = || {
            if base(receiver) == Some("baytyo") {
                return "abn".to_string();
            }
            if base(receiver) == Some("qoraal") {
                return "qoraal".to_string();
            }
            if matches!(base(receiver), Some("teed" | "urur")) {
                return receiver
                    .split_once('[')
                    .and_then(|(_, tail)| tail.strip_suffix(']'))
                    .unwrap_or("door")
                    .into();
            }
            "door".into()
        };
        if self.name == "hel" && arguments == Some(1) && base(receiver) != Some("walax") {
            return optional(&element());
        }
        match self.result.as_str() {
            "$self" => receiver.into(),
            "$element" => element(),
            "$element?" => optional(&element()),
            result => result.into(),
        }
    }
    pub fn label(&self, signature: &Signature, receiver: Option<&str>) -> String {
        let ty = receiver.or(self.receiver.as_deref()).unwrap_or("");
        let owner = if self.receiver.is_some() {
            format!("{}.", display(ty))
        } else {
            String::new()
        };
        format!(
            "hawl {owner}{}({}): {}",
            self.name,
            signature.parameters.join(", "),
            display(&self.result_for(ty, None))
        )
    }
    pub fn hover(&self, receiver: Option<&str>) -> String {
        let signatures = self
            .signatures
            .iter()
            .map(|signature| self.label(signature, receiver))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{signatures}\n\n{}\nNative arguments are positional.{}",
            self.documentation,
            if self.stride == 2 {
                " Arguments are alternating string keys and values; the count must be even."
            } else {
                ""
            }
        )
    }
}
fn optional(ty: &str) -> String {
    if ty == "door" || ty.ends_with('?') {
        ty.into()
    } else {
        format!("{ty}?")
    }
}
pub fn display(ty: &str) -> String {
    ty.replace(CURSOR, "wareeg")
}

/// Only exact primitive/container shapes are native receivers. Opaque class
/// and module keys are kept separate, including classes named like methods.
pub fn base(receiver: &str) -> Option<&str> {
    let receiver = receiver.strip_suffix('?').unwrap_or(receiver);
    if receiver == CURSOR {
        return Some(CURSOR);
    }
    let root = receiver.split('[').next().unwrap_or(receiver);
    if !matches!(
        root,
        "teed" | "walax" | "qoraal" | "baytyo" | "xirmo" | "urur"
    ) {
        return None;
    }
    if root != receiver {
        let mut depth = 0usize;
        for (index, ch) in receiver.char_indices().skip(root.len()) {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index + 1 != receiver.len() {
                        return None;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 || !receiver.ends_with(']') {
            return None;
        }
    }
    Some(root)
}

struct Catalog {
    entries: Vec<Api>,
    source: String,
}
static CATALOG: OnceLock<Catalog> = OnceLock::new();
pub fn entries() -> &'static [Api] {
    &catalog().entries
}
pub fn source() -> &'static str {
    &catalog().source
}
fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(build)
}
fn build() -> Catalog {
    let arities: HashMap<_, _> = crate::runtime::stdlib::builtin_metadata().collect();
    let mut entries = Vec::new();
    for (text, method) in [
        (include_str!("native_api/globals.tsv"), false),
        (include_str!("native_api/methods.tsv"), true),
    ] {
        for line in text
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(
                fields.len(),
                if method { 5 } else { 4 },
                "invalid native API row: {line}"
            );
            let shift = usize::from(method);
            let name = fields[shift];
            let Some(stride) = (if method {
                Some(1)
            } else {
                arities.get(name).map(|arity| arity.stride)
            }) else {
                continue;
            };
            let receiver = method.then(|| fields[0].to_string());
            entries.push(Api {
                id: format!("@native:{}:{name}", receiver.as_deref().unwrap_or("global")),
                name: name.into(),
                receiver,
                signatures: fields[shift + 1]
                    .split('~')
                    .map(|params| Signature {
                        parameters: params
                            .split(';')
                            .filter(|p| !p.is_empty())
                            .map(str::to_string)
                            .collect(),
                    })
                    .collect(),
                result: fields[shift + 2].into(),
                documentation: fields[shift + 3].into(),
                stride,
                start: 0,
                end: 0,
            });
        }
    }
    entries.sort_by(|a, b| (&a.receiver, &a.name).cmp(&(&b.receiver, &b.name)));
    let mut source=String::from("// Native Soplang runtime API. Read-only generated reference.\n// [] marks optional positional arguments; * marks a rest argument.\n// Declarations below describe native operations and are not executable source.\n\n");
    for entry in &mut entries {
        source.push_str("// ");
        source.push_str(&entry.documentation);
        source.push('\n');
        for (index, signature) in entry.signatures.iter().enumerate() {
            let label = entry.label(signature, None);
            source.push_str("// ");
            if index == 0 {
                entry.start = source.len() + label.find(&format!("{}(", entry.name)).unwrap();
                entry.end = entry.start + entry.name.len();
            }
            source.push_str(&label);
            source.push('\n');
        }
        source.push('\n');
    }
    Catalog { entries, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declarations_cover_real_registered_globals_and_match_their_argument_ranges() {
        let native: HashMap<_, _> = crate::runtime::stdlib::builtin_metadata().collect();
        let globals: HashMap<_, _> = entries()
            .iter()
            .filter(|api| api.receiver.is_none())
            .map(|api| (api.name.as_str(), api))
            .collect();
        assert_eq!(globals.len(), native.len());
        for (name, arity) in native {
            let api = globals
                .get(name)
                .unwrap_or_else(|| panic!("missing native description: {name}"));
            let ranges: Vec<_> = api.signatures.iter().map(Signature::bounds).collect();
            assert_eq!(
                ranges.iter().map(|(min, _)| *min).min(),
                Some(arity.minimum),
                "{name}"
            );
            assert_eq!(
                ranges.iter().map(|(_, max)| *max).max(),
                Some(arity.maximum),
                "{name}"
            );
            assert_eq!(api.stride, arity.stride);
        }
        let ids: std::collections::HashSet<_> = entries().iter().map(|api| &api.id).collect();
        assert_eq!(ids.len(), entries().len());
        for api in entries() {
            assert_eq!(&source()[api.start..api.end], api.name);
        }
    }
    #[test]
    fn advertised_collection_methods_exist_and_accept_their_documented_argument_counts() {
        use crate::runtime::{self, abi, Value};
        use std::{cell::RefCell, rc::Rc};
        for api in entries().iter().filter(|api| api.receiver.is_some()) {
            for count in 0..=4 {
                let context = runtime::RuntimeContext::new();
                let result = context.execute(|| {
                    let value = match api.receiver.as_deref().unwrap() {
                        "teed" => Value::List(Rc::new(RefCell::new(Vec::new()))),
                        "walax" => Value::Object(Rc::new(RefCell::new(HashMap::new()))),
                        "qoraal" => Value::Str("abc".into()),
                        "baytyo" => Value::Bytes(Rc::new(RefCell::new(Vec::new()))),
                        "xirmo" => Value::Tuple(runtime::TupleData::new(Vec::new())),
                        "urur" => Value::Set(runtime::SetData::new(Vec::new()).unwrap()),
                        CURSOR => {
                            let callee = abi::soplang_get_builtin(b"wareeg".as_ptr(), 6);
                            let list = abi::value_to_soplang(&Value::List(Rc::new(RefCell::new(
                                Vec::new(),
                            ))));
                            let cursor = abi::soplang_call(callee, &list, 1);
                            abi::soplang_to_value(cursor).unwrap()
                        }
                        other => panic!("unknown native receiver: {other}"),
                    };
                    let value = abi::value_to_soplang(&value);
                    let args = vec![abi::SoplangValue::null(); count];
                    abi::soplang_call_method(
                        value,
                        api.name.as_ptr(),
                        api.name.len(),
                        args.as_ptr(),
                        count as i32,
                    )
                });
                let accepted = api.signatures.iter().any(|signature| {
                    let (min, max) = signature.bounds();
                    (min..=max).contains(&count)
                });
                if let Err(error) = &result {
                    let message = error.to_string();
                    assert!(!message.contains("ma laha hawl"), "{}: {message}", api.id);
                    let arity_error = message.contains("waxay qaadataa")
                        || message.contains("waxay u baahan tahay 1 dood oo hawl ah");
                    assert!(
                        !accepted || !arity_error,
                        "{} count {count}: {message}",
                        api.id
                    );
                } else {
                    assert!(accepted, "{} accepts undocumented count {count}", api.id);
                }
            }
        }
    }
}
