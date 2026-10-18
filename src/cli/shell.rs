//! Interactive REPL (Phase 6).

use std::fs;
use std::path::Path;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::error::{external_error, file_read_error, input_error, SoplangError};
use crate::{format_error_with_source, run_source};

use super::ReplSession;

const PROMPT: &str = "soplang> ";
const CONTINUATION_PROMPT: &str = "    ... ";

/// History file under home or temp; ignored if we can't determine path.
fn history_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .map(|p| p.join(".soplang_history"))
}

fn welcome_banner() -> String {
    format!(
        "Soplang {}: Luuqadda barnaamijyada ee Af-Soomaaliga.\n  /caawi  = caawimo   /bixi = bixi   /akhrifayl <fayl> = akhri oo fuli\n  Ctrl+D ama /bixi = bixi",
        env!("CARGO_PKG_VERSION")
    )
}

const HELP_TEXT: &str = r#"Caawimo (amarada):
  /caawi, /help     Muuji boggan
  /bixi, /exit      Ka bixi REPL
  /akhrifayl <fayl> Akhri faylka .sop oo fuli
  /ast              Muuji AST (qaabka weedha)
  /hir              Muuji qaabka HIR

Haddii aad geliso weedh kaliya (tusaale: 1+2), natiijada waxaa laguu soo bandhigayaa."#;

pub struct Shell {
    editor: DefaultEditor,
    strict: bool,
    session: ReplSession,
}

impl Shell {
    pub fn new(strict: bool) -> Self {
        Self::try_new(strict).expect("REPL lama bilaabi karin")
    }

    /// Fallible startup used by the CLI, so editor/JIT errors are diagnostics.
    pub fn try_new(strict: bool) -> Result<Self, SoplangError> {
        let mut editor = DefaultEditor::new().map_err(|e| editor_error(&e))?;
        if let Some(p) = history_path() {
            let _ = editor.load_history(&p);
        }
        let session = ReplSession::new(strict)?;
        Ok(Self {
            editor,
            strict,
            session,
        })
    }

    pub fn run(&mut self) {
        println!("{}\n", welcome_banner());

        loop {
            let mut input = match self.read_input(PROMPT) {
                Some(s) => s,
                None => break,
            };

            // Multi-line: keep reading while line ends with \ or has unclosed { ( [
            loop {
                let trimmed = input.trim_end();
                let needs_more = trimmed.ends_with('\\')
                    || trimmed.matches('{').count() != trimmed.matches('}').count()
                    || trimmed.matches('(').count() != trimmed.matches(')').count()
                    || trimmed.matches('[').count() != trimmed.matches(']').count();
                if !needs_more {
                    break;
                }
                match self.editor.readline(CONTINUATION_PROMPT) {
                    Ok(line) => {
                        input.push('\n');
                        input.push_str(&line);
                    }
                    Err(ReadlineError::Interrupted) => {
                        input.clear();
                        break;
                    }
                    Err(ReadlineError::Eof) => break,
                    Err(e) => {
                        eprintln!("{}", format_error_with_source(&editor_error(&e), None));
                        break;
                    }
                }
            }

            let source = input.trim().trim_end_matches('\\').trim();
            if source.is_empty() {
                continue;
            }

            let _ = self.editor.add_history_entry(source);

            // Special commands
            if source.starts_with('/') {
                if self.handle_command(source) {
                    break;
                }
                continue;
            }

            self.execute(source);
        }

        if let Some(p) = history_path() {
            let _ = self.editor.save_history(&p);
        }
    }

    fn read_input(&mut self, prompt: &str) -> Option<String> {
        match self.editor.readline(prompt) {
            Ok(line) => Some(line),
            Err(ReadlineError::Interrupted) => None,
            Err(ReadlineError::Eof) => None,
            Err(e) => {
                eprintln!("{}", format_error_with_source(&editor_error(&e), None));
                None
            }
        }
    }

    /// Returns true if REPL should exit.
    fn handle_command(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let cmd = parts.get(0).copied().unwrap_or("");
        match cmd {
            "/caawi" | "/help" => {
                println!("{}", HELP_TEXT);
            }
            "/bixi" | "/exit" => return true,
            "/akhrifayl" | "/load" => {
                let path = parts.get(1).copied().unwrap_or("").trim();
                if path.is_empty() {
                    eprintln!("Khalad: Faylka waa in la sheegaa: /akhrifayl <fayl>");
                    return false;
                }
                match fs::read_to_string(path) {
                    Ok(source) => self.execute_at(&source, Some(Path::new(path))),
                    Err(e) => eprintln!(
                        "{}",
                        format_error_with_source(
                            &file_read_error(std::path::Path::new(path), &e),
                            None
                        )
                    ),
                }
            }
            "/ast" => {
                let rest = line.strip_prefix(cmd).unwrap_or("").trim();
                if rest.is_empty() {
                    eprintln!("Khalad: Geli weedh: /ast <weedh>");
                    return false;
                }
                if let Err(e) = run_source(rest, None, true, false, self.strict) {
                    eprintln!("{}", format_error_with_source(&e, Some(rest)));
                }
            }
            "/hir" => {
                let rest = line.strip_prefix(cmd).unwrap_or("").trim();
                if rest.is_empty() {
                    eprintln!("Khalad: Geli weedh: /hir <weedh>");
                    return false;
                }
                if let Err(e) = run_source(rest, None, false, true, self.strict) {
                    eprintln!("{}", format_error_with_source(&e, Some(rest)));
                }
            }
            _ => eprintln!("Khalad: Amar aan la garanayn '{}'. Geli /caawi.", cmd),
        }
        false
    }

    fn execute(&mut self, source: &str) {
        self.execute_at(source, None);
    }

    fn execute_at(&mut self, source: &str, path: Option<&Path>) {
        if let Err(e) = self.session.eval(source, path) {
            eprintln!("{}", format_error_with_source(&e, Some(source)));
        }
    }
}

fn editor_error(error: &ReadlineError) -> SoplangError {
    match error {
        ReadlineError::Io(error) => input_error(error),
        _ => external_error("Qoraalka amarka lama akhrin karin.", error.to_string()),
    }
}
