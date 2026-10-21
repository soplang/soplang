//! Source debugger state belongs to one execution context. Inspection never
//! evaluates Soplang expressions or retains values after their native frame.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use super::context::RuntimeLocal;
use super::{SoplangValue, Value};
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebugConfig {
    pub breakpoints: BTreeSet<(PathBuf, usize)>,
    pub continue_at_start: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub source: String,
    pub locals: Vec<(String, usize)>,
}

#[derive(Default)]
struct Frame {
    location: Option<Location>,
    initialized: BTreeSet<usize>,
}

#[derive(Default)]
pub(super) struct Debugger {
    enabled: bool,
    breakpoints: BTreeSet<(PathBuf, usize)>,
    // None continues; Some(n) pauses at the next statement at depth <= n.
    step_depth: Option<usize>,
    frames: Vec<Frame>,
}

const DEBUGGER: RuntimeLocal<RefCell<Debugger>> = RuntimeLocal::new(|state| &state.debugger);

pub(super) fn enter() {
    DEBUGGER.with(|debug| {
        let mut debug = debug.borrow_mut();
        if debug.enabled {
            debug.frames.push(Frame::default());
        }
    });
}
pub(super) fn leave() {
    DEBUGGER.with(|debug| {
        debug.borrow_mut().frames.pop();
    });
}

fn normalize(file: &std::path::Path) -> PathBuf {
    std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf())
}

pub(crate) fn begin(config: DebugConfig) {
    let root_count = RuntimeLocal::new(|state| &state.roots).with(|roots| roots.borrow().len());
    DEBUGGER.with(|debug| {
        *debug.borrow_mut() = Debugger {
            enabled: true,
            breakpoints: config
                .breakpoints
                .into_iter()
                .map(|(file, line)| (normalize(&file), line))
                .collect(),
            step_depth: (!config.continue_at_start).then_some(usize::MAX),
            frames: (0..root_count).map(|_| Frame::default()).collect(),
        };
    });
}

pub(crate) fn initialize(slot: usize) {
    DEBUGGER.with(|debug| {
        if let Some(frame) = debug.borrow_mut().frames.last_mut() {
            frame.initialized.insert(slot);
        }
    });
}

pub(crate) fn point(location: Location) -> Result<(), SoplangError> {
    let pause = DEBUGGER.with(|debug| {
        let mut debug = debug.borrow_mut();
        if !debug.enabled {
            return false;
        }
        let depth = debug.frames.len();
        let stop = debug.step_depth.is_some_and(|target| depth <= target)
            || debug
                .breakpoints
                .contains(&(location.file.clone(), location.line));
        if let Some(frame) = debug.frames.last_mut() {
            frame.location = Some(location);
        }
        stop
    });
    if pause {
        pause_console()?;
    }
    Ok(())
}

fn failure(message: &str) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}

/// An explicit jebi() remains available without compiler instrumentation.
pub(crate) fn explicit_break() -> Result<(), SoplangError> {
    pause_console()
}

fn pause_console() -> Result<(), SoplangError> {
    let input = std::io::stdin();
    let output = std::io::stderr();
    console(&mut input.lock(), &mut output.lock())
}

fn output_failure(error: std::io::Error) -> SoplangError {
    crate::error::io_error("Qalabka baarista lama akhriyi ama qori karin.", &error)
}

fn stack(output: &mut impl Write, selected: usize) -> std::io::Result<()> {
    let calls = super::abi::call_stack();
    DEBUGGER.with(|debug| {
        let debug = debug.borrow();
        let count = debug.frames.len();
        if count == 0 {
            for (index, call) in calls.iter().rev().enumerate() {
                writeln!(
                    output,
                    "{index}: {} {}:{}",
                    call.function, call.line, call.col
                )?;
            }
        } else {
            for (index, frame) in debug.frames.iter().rev().enumerate() {
                let name = if count - index - 1 == 0 {
                    "<main>"
                } else {
                    calls
                        .get(count - index - 2)
                        .map(|call| call.function.as_str())
                        .unwrap_or("<hawl>")
                };
                write!(
                    output,
                    "{} {index}: {name}",
                    if index == selected { ">" } else { " " }
                )?;
                if let Some(location) = &frame.location {
                    write!(
                        output,
                        " {}:{}:{}",
                        location.file.display(),
                        location.line,
                        location.col
                    )?;
                }
                writeln!(output)?;
            }
        }
        Ok(())
    })
}

// Cap nesting, collection entries and strings before rendering. This also
// avoids cycles and side effects from user-defined string conversions.
fn render(value: &Value, depth: usize) -> String {
    match value {
        Value::Str(text) => {
            let mut result: String = text.chars().take(256).collect();
            if text.chars().nth(256).is_some() {
                result.push('…');
            }
            format!("{result:?}")
        }
        Value::List(values) => {
            if depth == 0 {
                return "[…]".into();
            }
            let values = values.borrow();
            let mut parts: Vec<_> = values
                .iter()
                .take(16)
                .map(|v| render(v, depth - 1))
                .collect();
            if values.len() > 16 {
                parts.push("…".into());
            }
            format!("[{}]", parts.join(", "))
        }
        Value::Object(values) => {
            if depth == 0 {
                return "{…}".into();
            }
            let values = values.borrow();
            let mut keys: Vec<_> = values.keys().take(16).collect();
            keys.sort();
            let mut parts: Vec<_> = keys
                .iter()
                .take(16)
                .map(|key| {
                    format!(
                        "{}: {}",
                        render(&Value::Str((*key).clone()), 0),
                        render(&values[*key], depth - 1)
                    )
                })
                .collect();
            if values.len() > 16 {
                parts.push("…".into());
            }
            format!("{{{}}}", parts.join(", "))
        }
        Value::Tuple(_) => "<lammaane>".into(),
        Value::Set(_) => "<urur>".into(),
        Value::Bytes(bytes) => format!("<baytyo: {}>", bytes.borrow().len()),
        Value::Error(_) => "<khalad>".into(),
        _ => super::value_to_string(value),
    }
}

fn locals(output: &mut impl Write, selected: usize, name: Option<&str>) -> std::io::Result<()> {
    let (location, initialized, index) = DEBUGGER
        .with(|debug| {
            let debug = debug.borrow();
            let index = debug.frames.len().checked_sub(selected + 1);
            index.and_then(|index| {
                debug
                    .frames
                    .get(index)
                    .map(|frame| (frame.location.clone(), frame.initialized.clone(), index))
            })
        })
        .unwrap_or_default();
    let Some(location) = location else {
        return writeln!(
            output,
            "Xogta doorsoomayaasha lama diyaarin; adeegso soplang debug."
        );
    };
    let root =
        RuntimeLocal::new(|state| &state.roots).with(|roots| roots.borrow().get(index).copied());
    let Some(root) = root else {
        return Ok(());
    };
    let mut found = false;
    for (local, slot) in location.locals {
        if name.is_some_and(|name| name != local) {
            continue;
        }
        found = true;
        let rendered = if !initialized.contains(&slot) || slot >= root.len {
            "<lama qeexin>".into()
        } else {
            // The selected frame is still active and its registered shadow is
            // updated after every generated write. No evaluation occurs here.
            let value = unsafe {
                SoplangValue {
                    tag: *root.tags.add(slot * root.stride) as u8,
                    _pad: [0; 7],
                    payload: *root.payloads.add(slot * root.stride),
                }
            };
            let value = if index == 0 {
                RuntimeLocal::new(|state| &state.globals)
                    .with(|globals| globals.borrow().get(&local).copied())
                    .unwrap_or(value)
            } else {
                value
            };
            super::soplang_to_value(value)
                .map(|v| render(&v, 3))
                .unwrap_or_else(|_| "<qiime aan la heli karin>".into())
        };
        writeln!(output, "{local} = {rendered}")?;
    }
    if !found {
        writeln!(output, "Doorsoomaha lama helin.")?;
    }
    Ok(())
}

fn console(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), SoplangError> {
    let mut selected = 0;
    stack(output, selected).map_err(output_failure)?;
    DEBUGGER
        .with(|debug| {
            if let Some(location) = debug
                .borrow()
                .frames
                .last()
                .and_then(|f| f.location.as_ref())
            {
                writeln!(output, "{}", location.source.trim_end())?;
            }
            Ok::<_, std::io::Error>(())
        })
        .map_err(output_failure)?;
    loop {
        write!(output, "baar> ")
            .and_then(|_| output.flush())
            .map_err(output_failure)?;
        let mut bytes = Vec::new();
        let n = std::io::Read::take(&mut *input, 4097)
            .read_until(b'\n', &mut bytes)
            .map_err(output_failure)?;
        if n == 0 {
            if !DEBUGGER.with(|debug| debug.borrow().enabled) {
                return Ok(());
            }
            return Err(failure(
                "Gelintii baaristu way dhammaatay; barnaamijka waa la joojiyey.",
            ));
        }
        if bytes.len() > 4096 {
            return Err(failure("Amarka baaristu aad buu u dheer yahay."));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| failure("Amarku waa inuu noqdaa qoraal UTF-8 ah."))?;
        let parts: Vec<_> = text.split_whitespace().collect();
        match parts.as_slice() {
            ["soco"] => { DEBUGGER.with(|d|d.borrow_mut().step_depth=None); return Ok(()); }
            ["tallaabo"] => { DEBUGGER.with(|d|d.borrow_mut().step_depth=Some(usize::MAX)); return Ok(()); }
            ["xiga"] => { DEBUGGER.with(|d| { let mut d=d.borrow_mut(); d.step_depth=Some(d.frames.len()); }); return Ok(()); }
            ["bax"] => { DEBUGGER.with(|d| { let mut d=d.borrow_mut(); d.step_depth=Some(d.frames.len().saturating_sub(1)); }); return Ok(()); }
            ["jooji"] => return Err(failure("Joojinta waa la dalbaday.")),
            ["raad"] => stack(output, selected).map_err(output_failure)?,
            ["deegaan"] => locals(output,selected,None).map_err(output_failure)?,
            ["eeg",name] => locals(output,selected,Some(name)).map_err(output_failure)?,
            ["jir",index] => {
                if let Ok(index) = index.parse::<usize>() {
                    if DEBUGGER.with(|d| index < d.borrow().frames.len()) { selected=index; stack(output,selected).map_err(output_failure)?; continue; }
                }
                writeln!(output,"Lambarka jirku ma saxna.").map_err(output_failure)?;
            }
            ["jebi",target] | ["tir",target] => {
                let target = parse_breakpoint(target, None);
                match target {
                    Ok(target) => {
                        if parts[0] == "jebi" && DEBUGGER.with(|d|d.borrow().breakpoints.len() >= 128 && !d.borrow().breakpoints.contains(&target)) {
                            writeln!(output, "Waxaa la gaadhay xadka 128 barood oo joogsi ah.").map_err(output_failure)?;
                            continue;
                        }
                        DEBUGGER.with(|d| { let mut d=d.borrow_mut(); if parts[0]=="tir" {d.breakpoints.remove(&target);} else {d.breakpoints.insert(target);} });
                        writeln!(output,"Barta joogsiga waa la cusbooneysiiyey.").map_err(output_failure)?;
                    }
                    Err(error) => writeln!(output,"{}",error.message()).map_err(output_failure)?,
                }
            }
            _ => writeln!(output,"Amarro: soco, tallaabo, xiga, bax, raad, jir N, deegaan, eeg MAGAC, jebi FAYL:SAF, tir FAYL:SAF, jooji.").map_err(output_failure)?,
        }
    }
}

pub fn parse_breakpoint(
    text: &str,
    default: Option<&std::path::Path>,
) -> Result<(PathBuf, usize), SoplangError> {
    let (file, line) = if let Some((file, line)) = text.rsplit_once(':') {
        (PathBuf::from(file), line)
    } else {
        (
            default
                .ok_or_else(|| failure("Sheeg barta joogsiga sida FAYL:SAF."))?
                .to_path_buf(),
            text,
        )
    };
    let line = line
        .parse::<usize>()
        .ok()
        .filter(|line| *line > 0)
        .ok_or_else(|| failure("Safku waa inuu noqdaa tiro togan."))?;
    if file.as_os_str().is_empty() {
        return Err(failure("Jidka barta joogsiga waa madhan yahay."));
    }
    Ok((normalize(&file), line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{soplang_str, RuntimeContext};

    #[test]
    fn inspection_is_rooted_bounded_and_scoped_to_execution() {
        let context = RuntimeContext::new();
        context
            .execute(|| {
                let value = soplang_str("Amina".as_ptr(), 5);
                let tags = [value.tag as i64, 0];
                let payloads = [value.payload, 0];
                super::super::roots::enter(tags.as_ptr(), payloads.as_ptr(), 2, 1);
                begin(DebugConfig {
                    continue_at_start: true,
                    ..Default::default()
                });
                initialize(0);
                point(Location {
                    file: "inspect.sop".into(),
                    line: 1,
                    col: 1,
                    source: "".into(),
                    locals: vec![("name".into(), 0), ("future".into(), 1)],
                })
                .unwrap();
                context.collect();
                let mut output = Vec::new();
                console(&mut std::io::Cursor::new(b"deegaan\nsoco\n"), &mut output).unwrap();
                let output = String::from_utf8(output).unwrap();
                assert!(
                    output.contains("name = \"Amina\"")
                        && output.contains("future = <lama qeexin>"),
                    "{output}"
                );
                super::super::roots::leave();
            })
            .unwrap();
        context.with(|| {
            DEBUGGER.with(|debug| {
                let debug = debug.borrow();
                assert!(!debug.enabled);
                assert!(debug.frames.is_empty());
            })
        });
        let values = std::rc::Rc::new(RefCell::new(Vec::new()));
        let cycle = Value::List(values.clone());
        values.borrow_mut().push(cycle.clone());
        assert!(render(&cycle, 3).len() < 100);
        values.borrow_mut().clear();
        assert!(render(&Value::Str("x".repeat(100_000)), 3).len() < 300);
    }
}
