mod arguments;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser as ClapParser;

use soplang::{build_source, cli::Shell, format_error_with_source, run_source, set_program_arguments};

#[derive(ClapParser)]
#[command(
    name = "soplang",
    about = "Luuqadda barnaamijyada ee Af-Soomaaliga.",
    disable_help_flag = true,
    disable_version_flag = true,
    override_usage = arguments::USAGE,
    help_template = "{about}\n\nAdeegsiga: {usage}\n\nFaylka:\n{positionals}\n\nXulashooyinka:\n{options}{after-help}\n",
    version,
    after_help = "Haddii aan waxba la raacin, waxaa furmaya REPL.\nAdeegso -i si aad REPL ugu sii shaqayso marka faylka ama tusaalaha la fuliyo."
)]
struct Cli {
    #[arg(short = 'h', long, action = clap::ArgAction::Help, help = "Muuji caawimada.")]
    help: Option<bool>,
    #[arg(short = 'V', long, action = clap::ArgAction::Version, help = "Muuji nooca Soplang.")]
    version: Option<bool>,
    #[arg(
        long,
        help = "Ka dhis faylka .sop barnaamij si madax bannaan u shaqeeya; sheeg faylka la dhisayo."
    )]
    build: bool,

    #[arg(
        short = 'o',
        long,
        help = "Meesha lagu kaydinayo barnaamijka la dhisayo (--build)."
    )]
    #[arg(value_name = "BARNAAMIJKA")]
    output: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = 2,
        hide_default_value = true,
        value_name = "HEERKA",
        help = "Heerka hagaajinta koodhka: 0 ilaa 3 (--build; haddii aan la sheegin: 2)."
    )]
    opt_level: u8,

    #[arg(short, long, help = "Ha muujin farriinta guusha dhismaha (--build).")]
    quiet: bool,

    #[arg(short, long, help = "Fuli qoraalka la bixiyey, dabadeed bax.")]
    #[arg(value_name = "QORAALKA")]
    command: Option<String>,

    #[arg(short, long, help = "Fuli faylka la bixiyey, dabadeed bax.")]
    #[arg(value_name = "FAYLKA")]
    file: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Fuli tusaalaha lambarkiisu yahay N ee examples/; tirintu waxay ka bilaabataa 1."
    )]
    #[arg(value_name = "LAMBARKA")]
    example: Option<usize>,

    #[arg(short, long, help = "Fur REPL marka faylka ama tusaalaha la fuliyo.")]
    interactive: bool,

    #[arg(help = "Faylka .sop ee la fulinayo ama la dhisayo; haddii aan la sheegin, fur REPL.")]
    #[arg(value_name = "FAYLKA")]
    filename: Option<PathBuf>,

    #[arg(long, help = "Muuji qaabdhismeedka AST adigoon fulin barnaamijka.")]
    ast: bool,

    #[arg(long, help = "Muuji qaabka HIR adigoon fulin barnaamijka.")]
    hir: bool,

    #[arg(long, help = "Daar hubinta adag ee noocyada xogta.")]
    strict: bool,

    #[arg(long, help = "Ha isticmaalin midab marka khalad la soo bandhigayo; waxay la mid tahay NO_COLOR=1.")]
    no_color: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    arguments: Vec<String>,
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code();
            if error.use_stderr() {
                eprint!("{}", arguments::format_argument_error(&error));
            } else {
                let _ = error.print();
            }
            process::exit(code);
        }
    };

    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }
    set_program_arguments(user_arguments(&cli));

    if cli.build {
        let path = match &cli.filename {
            Some(p) => p.clone(),
            None => {
                eprintln!("Khalad: --build waxa ay u baahan tahay magaca faylka: soplang --build <fayl.sop> [-o <barnaamij>] [-q]");
                process::exit(1);
            }
        };
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error_with_source(&soplang::error::file_read_error(&path, &e), None)
                );
                process::exit(1);
            }
        };
        let out = cli.output.clone().unwrap_or_else(|| {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("a.out")
                .to_string();
            PathBuf::from("barnaamij").join(stem)
        });
        let opt_level = cli.opt_level;
        let strict = cli.strict;

        if cli.quiet {
            match build_source(&source, Some(&path), &out, opt_level, strict) {
                Ok(()) => return,
                Err(e) => {
                    eprintln!("{}", format_error_with_source(&e, None));
                    process::exit(1);
                }
            }
        }

        let start = Instant::now();
        let out_build = out.clone();
        let path_for_build = path.clone();
        let build_handle = thread::spawn(move || {
            build_source(
                &source,
                Some(&path_for_build),
                &out_build,
                opt_level,
                strict,
            )
        });

        let spin = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴'];
        let mut i = 0;
        while !build_handle.is_finished() {
            print!("\rBarnaamijka waa la dhisayaa... {}  ", spin[i]);
            let _ = std::io::stdout().flush();
            i = (i + 1) % spin.len();
            thread::sleep(Duration::from_millis(80));
        }

        let result = build_handle.join().unwrap_or_else(|_| {
            Err(soplang::error::runtime_error(
                "Hawshii dhismaha ayaa si lama filaan ah u joogsatay.",
                0,
                0,
            ))
        });
        let elapsed = start.elapsed().as_secs_f64();

        match result {
            Ok(()) => {
                print!(
                    "\r{: <50}\rDhisiddu waa ay dhammaatay: {} ({:.1}s)\n",
                    "",
                    out.display(),
                    elapsed
                );
                let _ = std::io::stdout().flush();
                return;
            }
            Err(e) => {
                print!("\r{: <50}\r", "");
                let _ = std::io::stdout().flush();
                eprintln!("{}", format_error_with_source(&e, None));
                process::exit(1);
            }
        }
    }

    let run_then_maybe_shell = |path: PathBuf, source: String| match run_source(
        &source,
        Some(&path),
        cli.ast,
        cli.hir,
        cli.strict,
    ) {
        Ok(()) => {
            if cli.interactive {
                run_shell(cli.strict);
            }
        }
        Err(e) => {
            eprintln!("{}", format_error_with_source(&e, Some(&source)));
            process::exit(1);
        }
    };

    if let Some(code) = &cli.command {
        match run_source(code, None, false, false, cli.strict) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(code)));
                process::exit(1);
            }
        }
    }

    if let Some(ref path) = cli.file {
        match fs::read_to_string(path) {
            Ok(source) => run_then_maybe_shell(path.clone(), source),
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error_with_source(&soplang::error::file_read_error(&path, &e), None)
                );
                process::exit(1);
            }
        }
        return;
    }

    if let Some(n) = cli.example {
        let path = match example_path(n) {
            Some(p) => p,
            None => {
                eprintln!("Khalad: Tusaale {} ma jiro (faylka examples/ kuma jiro)", n);
                process::exit(1);
            }
        };
        match fs::read_to_string(&path) {
            Ok(source) => run_then_maybe_shell(path, source),
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error_with_source(&soplang::error::file_read_error(&path, &e), None)
                );
                process::exit(1);
            }
        }
        return;
    }

    if let Some(ref path) = cli.filename {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error_with_source(&soplang::error::file_read_error(&path, &e), None)
                );
                process::exit(1);
            }
        };
        match run_source(&source, Some(path.as_path()), cli.ast, cli.hir, cli.strict) {
            Ok(()) => {
                if cli.interactive {
                    run_shell(cli.strict);
                }
            }
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(&source)));
                process::exit(1);
            }
        }
        return;
    }

    run_shell(cli.strict);
}

fn run_shell(strict: bool) {
    match Shell::try_new(strict) {
        Ok(mut shell) => shell.run(),
        Err(error) => {
            eprintln!("{}", format_error_with_source(&error, None));
            process::exit(1);
        }
    }
}

fn user_arguments(cli: &Cli) -> Vec<String> {
    let positional_is_program = cli.command.is_none()
        && cli.file.is_none()
        && cli.example.is_none()
        && !cli.build;
    let mut args = Vec::new();
    if !positional_is_program {
        if let Some(name) = &cli.filename {
            args.push(name.display().to_string());
        }
    }
    args.extend(cli.arguments.iter().cloned());
    args
}

/// Resolve example N (1-based) to PathBuf. Sorts .sop files in examples/ by name.
fn example_path(n: usize) -> Option<PathBuf> {
    if n == 0 {
        return None;
    }
    let mut paths: Vec<PathBuf> = fs::read_dir("examples")
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "sop").unwrap_or(false))
        .filter(|p| {
            !p.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with('_')
        })
        .collect();
    paths.sort();
    paths.into_iter().nth(n - 1)
}
