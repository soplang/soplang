//! Interactive command guide shown when Soplang starts on a terminal.

use std::io::{self, IsTerminal};
use std::process::Command;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{
    Cmd, CompletionType, Config, Context, Editor, Helper, KeyCode, KeyEvent, Modifiers,
};

pub struct OptionSpec {
    english: &'static str,
    somali: &'static str,
    value: &'static str,
    description: &'static str,
}

pub struct CommandSpec {
    pub english: &'static str,
    pub somali: &'static str,
    pub action: &'static str,
    description: &'static str,
    usage: &'static str,
    options: &'static [OptionSpec],
}

const INIT: &[OptionSpec] = &[
    OptionSpec {
        english: "--name",
        somali: "--magac",
        value: "MAGAC",
        description: "Magaca xirmada cusub.",
    },
    OptionSpec {
        english: "--lib",
        somali: "--maktabad",
        value: "",
        description: "Abuur maktabad leh lib.sop.",
    },
];
const RUN: &[OptionSpec] = &[
    OptionSpec { english: "--strict", somali: "--adag", value: "", description: "Daar hubinta adag ee noocyada xogta." },
    OptionSpec { english: "--offline", somali: "--kayd-keliya", value: "", description: "Ku-tiirsanaanta ka hel qufulka iyo kaydka la hubiyey; ma joojiso shabakadda barnaamijka." },
];
const BUILD: &[OptionSpec] = &[
    OptionSpec {
        english: "--name",
        somali: "--magac",
        value: "MAGAC",
        description: "U bixi magaca barnaamijka marka hal fayl ama xirmo la dhisayo.",
    },
    OptionSpec {
        english: "--opt-level",
        somali: "--heer-hagaajin",
        value: "0..3",
        description: "Heerka hagaajinta LLVM; haddii aan la sheegin waa 2.",
    },
    OptionSpec {
        english: "--quiet",
        somali: "--aamus",
        value: "",
        description: "Ha muujin farriinta guusha dhismaha.",
    },
    OptionSpec {
        english: "--output",
        somali: "--natiijo",
        value: "JID",
        description: "Jidka saxda ah ee natiijada hal dhismo; sida caadiga ah dhis/MAGAC.",
    },
    OptionSpec {
        english: "--strict",
        somali: "--adag",
        value: "",
        description: "Daar hubinta adag ee noocyada xogta.",
    },
    OptionSpec {
        english: "--offline",
        somali: "--kayd-keliya",
        value: "",
        description: "Ha soo dejin xirmooyin; adeegso qufulka iyo kaydka la hubiyey.",
    },
];
const CHECK: &[OptionSpec] = &[
    OptionSpec {
        english: "--strict",
        somali: "--adag",
        value: "",
        description: "Daar hubinta adag ee noocyada xogta.",
    },
    OptionSpec {
        english: "--offline",
        somali: "--kayd-keliya",
        value: "",
        description: "Ha soo dejin xirmooyin; adeegso qufulka iyo kaydka la hubiyey.",
    },
];
const FORMAT: &[OptionSpec] = &[OptionSpec {
    english: "--write",
    somali: "--qor",
    value: "",
    description: "Faylasha ku qor qaabka la habeeyey; dhowr fayl ayaa la siin karaa.",
}];
const TEST: &[OptionSpec] = &[
    OptionSpec {
        english: "--filter",
        somali: "--shaandhe",
        value: "MAGAC",
        description: "Dooro tijaabooyinka magacooda ku habboon.",
    },
    OptionSpec {
        english: "--index",
        somali: "--lambarka",
        value: "N",
        description: "Fuli hal tijaabo oo lambarkeeda laga tiriyo eber.",
    },
    OptionSpec {
        english: "--parallel",
        somali: "--barbar",
        value: "",
        description:
            "Tijaabooyinka ku fuli shaqaale isbarbar socda oo tiradooda si toos ah loo doorto.",
    },
    OptionSpec {
        english: "--workers",
        somali: "--shaqaale",
        value: "1..32",
        description: "Tirada shaqaalaha isbarbar socda.",
    },
    OptionSpec {
        english: "--coverage",
        somali: "--dabool",
        value: "",
        description: "Soo saar xogta daboolidda koodhka.",
    },
    OptionSpec {
        english: "--report",
        somali: "--warbixin",
        value: "FAYL",
        description: "Warbixinta tijaabada ku qor faylka.",
    },
];
const DOC: &[OptionSpec] = &[
    OptionSpec {
        english: "--output",
        somali: "--natiijo",
        value: "FAYL",
        description: "Dukumentiga ku qor faylka halkii laga daabici lahaa.",
    },
    OptionSpec {
        english: "--private",
        somali: "--gaar",
        value: "",
        description: "Ku dar magacyada aan la soo saarin.",
    },
];
const DEBUG: &[OptionSpec] = &[
    OptionSpec {
        english: "--break",
        somali: "--hakad",
        value: "[FAYL:]SAF",
        description: "Dhig meel hakad oo debugger-ku istaago.",
    },
    OptionSpec {
        english: "--continue",
        somali: "--soco",
        value: "",
        description: "Sii wad ilaa meesha hakadka halkii laga joogsan lahaa weedha koowaad.",
    },
    OptionSpec {
        english: "--strict",
        somali: "--adag",
        value: "",
        description: "Daar hubinta adag ee noocyada xogta.",
    },
    OptionSpec {
        english: "--build",
        somali: "--dhis",
        value: "JID",
        description: "Dhis barnaamij LLVM ah oo leh baarista debugger-ka.",
    },
];
const OFFLINE: &[OptionSpec] = &[OptionSpec {
    english: "--offline",
    somali: "--kayd-keliya",
    value: "",
    description: "Ku-tiirsanaanta ka hel qufulka iyo kaydka la hubiyey oo keliya.",
}];
const DATA: &[OptionSpec] = &[
    OptionSpec {
        english: "--output",
        somali: "--natiijo",
        value: "FAYL",
        description: "Macmiilka xogta la dhalinayo ku qor faylka.",
    },
    OptionSpec {
        english: "--check",
        somali: "--hubi",
        value: "FAYL",
        description: "Hubi in macmiilka hore uu la mid yahay midka schema-du dhalinayso.",
    },
];
const EVAL: &[OptionSpec] = &[OptionSpec {
    english: "--strict",
    somali: "--adag",
    value: "",
    description: "Daar hubinta adag ee noocyada xogta.",
}];
const EXAMPLE: &[OptionSpec] = &[OptionSpec {
    english: "--interactive",
    somali: "--isdhexgal",
    value: "",
    description: "Fur REPL marka tusaaluhu dhammaado.",
}];
const INSPECT: &[OptionSpec] = &[
    OptionSpec {
        english: "--ast",
        somali: "--ast",
        value: "",
        description: "Muuji qaabdhismeedka AST adigoon fulin barnaamijka.",
    },
    OptionSpec {
        english: "--hir",
        somali: "--hir",
        value: "",
        description: "Muuji qaabdhismeedka HIR adigoon fulin barnaamijka.",
    },
];
const LSP: &[OptionSpec] = &[OptionSpec {
    english: "--stdio",
    somali: "--stdio",
    value: "",
    description: "La xiriir tafatiraha adigoo adeegsanaya gelinta iyo soo-saarka caadiga ah.",
}];
const HELP_ONLY: &[OptionSpec] = &[OptionSpec {
    english: "--help",
    somali: "--caawimo",
    value: "",
    description: "Muuji caawimada amarkan.",
}];
const GLOBAL: &[OptionSpec] = &[
    OptionSpec {
        english: "--help",
        somali: "--caawimo",
        value: "",
        description: "Muuji caawimada amarkan.",
    },
    OptionSpec {
        english: "--no-color",
        somali: "--midab-dami",
        value: "",
        description: "Dami midabka farriimaha khaladka.",
    },
];

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        english: "init",
        somali: "abuur",
        action: "init",
        description: "Abuur xirmo Soplang cusub.",
        usage: "[GALKA]",
        options: INIT,
    },
    CommandSpec {
        english: "run",
        somali: "fuli",
        action: "run",
        description: "Fuli faylka .sop ama xirmada hadda.",
        usage: "[FAYLKA.sop|GALKA] [-- DOODO]",
        options: RUN,
    },
    CommandSpec {
        english: "build",
        somali: "dhis",
        action: "build",
        description: "Dhis hal ama dhowr barnaamij oo madax bannaan.",
        usage: "[FAYLKA.sop ...|GALKA]",
        options: BUILD,
    },
    CommandSpec {
        english: "check",
        somali: "hubi",
        action: "check",
        description: "Hubi koodhka adigoon fulin.",
        usage: "[FAYLKA.sop ...]",
        options: CHECK,
    },
    CommandSpec {
        english: "fmt",
        somali: "habee",
        action: "fmt",
        description: "Habee qaabka koodhka.",
        usage: "FAYLKA.sop [...]",
        options: FORMAT,
    },
    CommandSpec {
        english: "test",
        somali: "tijaabi",
        action: "test",
        description: "Fuli tijaabooyinka barnaamijka.",
        usage: "[FAYLKA|GALKA]",
        options: TEST,
    },
    CommandSpec {
        english: "doc",
        somali: "duk",
        action: "doc",
        description: "Soo saar dukumenti API oo Markdown ah.",
        usage: "FAYLKA.sop",
        options: DOC,
    },
    CommandSpec {
        english: "debug",
        somali: "baar",
        action: "debug",
        description: "Ku baar barnaamijka meelaha hakadka.",
        usage: "FAYLKA.sop",
        options: DEBUG,
    },
    CommandSpec {
        english: "profile",
        somali: "cabbir",
        action: "profile",
        description: "Cabbir wakhtiga iyo wicitaannada hawlaha.",
        usage: "FAYLKA.sop",
        options: &[],
    },
    CommandSpec {
        english: "add",
        somali: "kudar",
        action: "add",
        description: "Ku dar ku-tiirsi xirmada.",
        usage: "MAGAC IL [GALKA]",
        options: OFFLINE,
    },
    CommandSpec {
        english: "remove",
        somali: "saar",
        action: "remove",
        description: "Ka saar ku-tiirsi xirmada.",
        usage: "MAGAC [GALKA]",
        options: OFFLINE,
    },
    CommandSpec {
        english: "update",
        somali: "cusboonaysii",
        action: "update",
        description: "Cusboonaysii ku-tiirsiyada qufulka.",
        usage: "[GALKA]",
        options: OFFLINE,
    },
    CommandSpec {
        english: "publish",
        somali: "daabac",
        action: "publish",
        description: "Daabac xirmada ku jirta galka hadda.",
        usage: "",
        options: &[],
    },
    CommandSpec {
        english: "data generate",
        somali: "xog abuur",
        action: "xog",
        description: "Soo saar ama hubi macmiilka schema.sop.",
        usage: "[SCHEMA]",
        options: DATA,
    },
    CommandSpec {
        english: "repl",
        somali: "isdhexgal",
        action: "repl",
        description: "Fur fadhi koodh isdhexgal ah.",
        usage: "",
        options: EVAL,
    },
    CommandSpec {
        english: "eval",
        somali: "qiimee",
        action: "eval",
        description: "Fuli qoraal Soplang oo toos loo bixiyey.",
        usage: "QORAAL",
        options: EVAL,
    },
    CommandSpec {
        english: "example",
        somali: "tusaale",
        action: "example",
        description: "Fuli tusaale lambarkiisa la doortay.",
        usage: "LAMBARKA",
        options: EXAMPLE,
    },
    CommandSpec {
        english: "inspect",
        somali: "eeg",
        action: "inspect",
        description: "Muuji AST ama HIR adigoon fulin.",
        usage: "FAYLKA.sop",
        options: INSPECT,
    },
    CommandSpec {
        english: "help",
        somali: "caawimo",
        action: "help",
        description: "Muuji caawimada amar gaar ah.",
        usage: "[AMAR]",
        options: &[],
    },
    CommandSpec {
        english: "version",
        somali: "nooc",
        action: "version",
        description: "Muuji nooca Soplang.",
        usage: "",
        options: &[],
    },
    CommandSpec {
        english: "lsp",
        somali: "lsp",
        action: "lsp",
        description: "Bilow adeegaha hab-maamuuska tafatiraha.",
        usage: "[--stdio]",
        options: LSP,
    },
];

fn common_options(spec: &CommandSpec) -> &'static [OptionSpec] {
    match spec.action {
        "init" | "publish" | "help" | "version" => HELP_ONLY,
        _ => GLOBAL,
    }
}

pub fn find_command(name: &str) -> Option<&'static CommandSpec> {
    let name = name.strip_prefix("--").unwrap_or(name);
    COMMANDS.iter().find(|spec| {
        [spec.english, spec.somali]
            .iter()
            .any(|alias| *alias == name || alias.split_whitespace().next() == Some(name))
    })
}

#[derive(Clone, Copy)]
enum Mode {
    Commands,
    Options(&'static CommandSpec),
}

struct DescriptionHint(String);

impl Hint for DescriptionHint {
    fn display(&self) -> &str {
        &self.0
    }
    fn completion(&self) -> Option<&str> {
        None
    }
}

struct BrowserHelper {
    mode: Mode,
    files: FilenameCompleter,
}

impl BrowserHelper {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            files: FilenameCompleter::new(),
        }
    }
}

impl Helper for BrowserHelper {}
impl Highlighter for BrowserHelper {}
impl Validator for BrowserHelper {}

impl Completer for BrowserHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let before = &line[..pos];
        match self.mode {
            Mode::Commands => {
                let prefixed = before.starts_with("--");
                let mut candidates = Vec::new();
                for spec in COMMANDS {
                    for (index, alias) in [spec.english, spec.somali].into_iter().enumerate() {
                        if index == 1 && spec.english == spec.somali {
                            continue;
                        }
                        let replacement = if prefixed {
                            format!("--{alias}")
                        } else {
                            alias.to_owned()
                        };
                        if replacement.starts_with(before) {
                            candidates.push(Pair {
                                display: format!("{replacement}  —  {}", spec.description),
                                replacement,
                            });
                        }
                    }
                }
                Ok((0, candidates))
            }
            Mode::Options(spec) => {
                let start = before
                    .rfind(char::is_whitespace)
                    .map_or(0, |index| index + 1);
                let partial = &before[start..];
                if partial.is_empty() || partial.starts_with('-') {
                    let mut candidates = Vec::new();
                    for option in spec.options.iter().chain(common_options(spec)) {
                        for (index, alias) in
                            [option.english, option.somali].into_iter().enumerate()
                        {
                            if index == 1 && option.english == option.somali {
                                continue;
                            }
                            if alias.starts_with(partial) {
                                candidates.push(Pair {
                                    display: format!(
                                        "{alias} {}  —  {}",
                                        option.value, option.description
                                    ),
                                    replacement: alias.to_owned(),
                                });
                            }
                        }
                    }
                    Ok((start, candidates))
                } else {
                    self.files.complete(line, pos, ctx)
                }
            }
        }
    }
}

impl Hinter for BrowserHelper {
    type Hint = DescriptionHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos != line.len() {
            return None;
        }
        match self.mode {
            Mode::Commands => find_command(line.trim())
                .map(|spec| DescriptionHint(format!("  — {}", spec.description))),
            Mode::Options(spec) => {
                let word = line.split_whitespace().last()?;
                spec.options
                    .iter()
                    .chain(common_options(spec))
                    .find(|option| option.english == word || option.somali == word)
                    .map(|option| DescriptionHint(format!("  — {}", option.description)))
            }
        }
    }
}

pub fn should_show() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

pub fn run() -> Result<(), String> {
    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .build();
    let mut editor = Editor::<BrowserHelper, DefaultHistory>::with_config(config)
        .map_err(|error| error.to_string())?;
    editor.bind_sequence(KeyEvent(KeyCode::Down, Modifiers::NONE), Cmd::Complete);
    editor.bind_sequence(
        KeyEvent(KeyCode::Up, Modifiers::NONE),
        Cmd::CompleteBackward,
    );
    println!(
        "Soplang {} — dooro amar. ↑↓ baadh, Tab dhammaystir, Enter dooro, Ctrl+D bax.",
        env!("CARGO_PKG_VERSION")
    );
    print_commands();
    loop {
        editor.set_helper(Some(BrowserHelper::new(Mode::Commands)));
        let selected = match editor.readline("soplang> ") {
            Ok(line) => line,
            Err(ReadlineError::Eof | ReadlineError::Interrupted) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let selected = selected.trim();
        if selected == "?" {
            print_commands();
            continue;
        }
        if selected.is_empty() {
            continue;
        }
        let Some(parts) = shlex::split(selected) else {
            eprintln!("Khalad: Calaamadaha xigashada amarku ma dhammaystirna.");
            continue;
        };
        let Some(first) = parts.first() else {
            continue;
        };
        let Some(spec) = find_command(first) else {
            eprintln!(
                "Khalad: Amarka '{}' lama aqoonsan. Geli ? si aad u aragto amarada.",
                first
            );
            continue;
        };
        let command = if first.starts_with("--") {
            format!("--{}", spec.action)
        } else if first == spec.somali.split_whitespace().next().unwrap_or("") {
            spec.somali
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned()
        } else {
            spec.english
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned()
        };
        let mut action_args = Vec::new();
        if spec.action == "xog" {
            action_args.push(if command == "xog" || command == "--xog" {
                "abuur".to_owned()
            } else {
                "generate".to_owned()
            });
        }
        let consumed = if spec.action == "xog"
            && parts
                .get(1)
                .is_some_and(|word| word == "generate" || word == "abuur")
        {
            2
        } else {
            1
        };
        let prefill = parts
            .iter()
            .skip(consumed)
            .map(|part| {
                shlex::try_quote(part)
                    .map(|word| word.to_string())
                    .unwrap_or_else(|_| part.to_owned())
            })
            .collect::<Vec<_>>()
            .join(" ");
        show_options(spec);
        editor.set_helper(Some(BrowserHelper::new(Mode::Options(spec))));
        let prompt = format!("soplang {command}> ");
        let line = match editor.readline_with_initial(&prompt, (&prefill, "")) {
            Ok(line) => line,
            Err(ReadlineError::Eof | ReadlineError::Interrupted) => continue,
            Err(error) => return Err(error.to_string()),
        };
        let Some(mut options) = shlex::split(&line) else {
            eprintln!("Khalad: Calaamadaha xigashada dooduhu ma dhammaystirna.");
            continue;
        };
        let mut command_line = vec![command];
        command_line.append(&mut action_args);
        command_line.append(&mut options);
        let status = Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
            .args(&command_line)
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            eprintln!("Amarku wuxuu ku dhammaaday xaalad: {status}");
        }
        println!();
    }
}

fn print_commands() {
    for spec in COMMANDS {
        println!(
            "  {:<26} {}",
            format!("{} / {}", spec.english, spec.somali),
            spec.description
        );
    }
    println!("  ?                          Mar kale muuji liiska amarada.\n");
}

fn show_options(spec: &CommandSpec) {
    println!(
        "\n{} / {} — {}",
        spec.english, spec.somali, spec.description
    );
    println!("Adeegsiga: soplang {} {}", spec.english, spec.usage);
    for option in spec.options.iter().chain(common_options(spec)) {
        println!(
            "  {:<36} {}",
            format!("{} / {} {}", option.english, option.somali, option.value),
            option.description
        );
    }
    println!("  ↑↓/Tab dooro doorasho; geli doodaha, dabadeed Enter. Ctrl+C ku noqo amarada.\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::history::DefaultHistory;

    #[test]
    fn every_command_has_two_spellings_and_completes_options() {
        let history = DefaultHistory::new();
        let context = Context::new(&history);
        let helper = BrowserHelper::new(Mode::Commands);
        for spec in COMMANDS {
            assert_eq!(find_command(spec.english).unwrap().action, spec.action);
            assert_eq!(find_command(spec.somali).unwrap().action, spec.action);
            let prefix = &spec.somali[..1];
            let (_, matches) = helper.complete(prefix, prefix.len(), &context).unwrap();
            assert!(
                matches
                    .iter()
                    .any(|candidate| candidate.replacement == spec.somali),
                "{}",
                spec.somali
            );
        }
        let build = find_command("dhis").unwrap();
        let helper = BrowserHelper::new(Mode::Options(build));
        let (_, options) = helper.complete("--hak", 5, &context).unwrap();
        assert!(options.is_empty());
        let (_, options) = helper.complete("--heer", 6, &context).unwrap();
        assert!(options
            .iter()
            .any(|candidate| candidate.replacement == "--heer-hagaajin"));
        let hint = helper
            .hint("--heer-hagaajin", "--heer-hagaajin".len(), &context)
            .unwrap();
        assert!(hint.display().contains("LLVM"));
        assert!(hint.completion().is_none());

        let init = BrowserHelper::new(Mode::Options(find_command("abuur").unwrap()));
        let (_, options) = init.complete("--midab", 7, &context).unwrap();
        assert!(options.is_empty());
        let lsp = BrowserHelper::new(Mode::Options(find_command("lsp").unwrap()));
        let (_, options) = lsp.complete("--std", 5, &context).unwrap();
        assert!(options
            .iter()
            .any(|candidate| candidate.replacement == "--stdio"));
    }
}
