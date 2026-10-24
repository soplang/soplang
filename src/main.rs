mod arguments;

use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser as ClapParser};

use soplang::{cli::Shell, format_error_with_source, run_source, set_program_arguments};

#[derive(ClapParser)]
#[command(
    name = "soplang",
    about = "Luuqadda barnaamijyada ee Af-Soomaaliga.",
    disable_help_flag = true,
    disable_version_flag = true,
    override_usage = arguments::USAGE,
    help_template = "{about}\n\nAdeegsiga: {usage}\n\nFaylka:\n{positionals}\n\nXulashooyinka:\n{options}{after-help}\n",
    version,
    after_help = "Amarrada: init/abuur, run/fuli, build/dhis, check/hubi, fmt/habee, test/tijaabi, doc/duk, debug/baar, profile/cabbir, add/kudar, remove/saar, update/cusboonaysii, publish/daabac, data/xog, repl/isdhexgal, eval/qiimee, example/tusaale, inspect/eeg, help/caawimo, version/nooc, lsp.\nAmar kasta waxaa loo qori karaa -- horgale ahaan, tusaale --dhis.\nSoplang keligiis wuxuu terminalka ka furaa tilmaamaha amarada; soplang repl wuxuu furaa REPL."
)]
struct Cli {
    #[arg(short = 'h', long, alias = "caawimo", action = clap::ArgAction::Help, help = "Muuji caawimada.")]
    help: Option<bool>,
    #[arg(short = 'V', long, alias = "nooc", action = clap::ArgAction::Version, help = "Muuji nooca Soplang.")]
    version: Option<bool>,
    #[arg(
        long,
        alias = "dhis",
        help = "Dhis barnaamij madax bannaan oo ka yimaadda xirmada hadda ama faylasha .sop ee la bixiyey."
    )]
    build: bool,

    #[arg(
        short = 'o',
        long,
        alias = "natiijo",
        help = "Meesha lagu kaydinayo barnaamijka la dhisayo (--build)."
    )]
    #[arg(value_name = "BARNAAMIJKA")]
    output: Option<PathBuf>,

    #[arg(
        long,
        alias = "magac",
        value_name = "MAGAC",
        help = "Magaca barnaamijka la dhisayo."
    )]
    name: Option<String>,

    #[arg(
        long,
        alias = "heer-hagaajin",
        default_value_t = 2,
        hide_default_value = true,
        value_name = "HEERKA",
        help = "Heerka hagaajinta koodhka: 0 ilaa 3 (--build; haddii aan la sheegin: 2)."
    )]
    opt_level: u8,

    #[arg(
        short,
        long,
        alias = "aamus",
        help = "Ha muujin farriinta guusha dhismaha (--build)."
    )]
    quiet: bool,

    #[arg(
        long,
        value_name = "MAGAC",
        requires = "qeexid",
        help = "Ku dar ku-tiirsan xirmada. Waxay u baahan tahay --qeexid."
    )]
    ku_dar: Option<String>,

    #[arg(
        long,
        value_name = "QEEXID",
        requires = "ku_dar",
        help = "Jidka ama kaydka Git ee ku-tiirsiga, sida ../xisaab ama github.com/koox/xisaab@v1.0.0."
    )]
    qeexid: Option<String>,

    #[arg(
        long,
        help = "Cusbooneysii ku-tiirsanayaasha xirmada oo qor soplang.lock."
    )]
    cusbooneysii: bool,

    #[arg(long, value_name = "MAGAC", help = "Ka saar ku-tiirsan xirmada.")]
    ka_saar: Option<String>,

    #[arg(
        long,
        alias = "kayd-keliya",
        help = "Adeegso soplang.lock iyo kaydka xirmooyinka oo la hubiyey adigoon xirmooyin soo dejin; ma xannibo shabakadda barnaamijka."
    )]
    offline: bool,

    #[arg(long, help = "Daabac xirmadan kaydka Soplang.")]
    daabac: bool,

    #[arg(
        short,
        long,
        alias = "amar",
        help = "Fuli qoraalka la bixiyey, dabadeed bax."
    )]
    #[arg(value_name = "QORAALKA")]
    command: Option<String>,

    #[arg(
        short,
        long,
        alias = "fayl",
        help = "Fuli faylka la bixiyey, dabadeed bax."
    )]
    #[arg(value_name = "FAYLKA")]
    file: Option<PathBuf>,

    #[arg(
        short,
        long,
        alias = "tusaale",
        help = "Fuli tusaalaha lambarkiisu yahay N ee examples/; tirintu waxay ka bilaabataa 1."
    )]
    #[arg(value_name = "LAMBARKA")]
    example: Option<usize>,

    #[arg(
        short,
        long,
        alias = "isdhexgal",
        help = "Fur REPL marka faylka ama tusaalaha la fuliyo."
    )]
    interactive: bool,

    #[arg(
        help = "Faylka .sop ee la fulinayo ama la dhisayo; haddii aan wax dood ah la bixin, terminalku wuxuu furaa tilmaamaha amarada."
    )]
    #[arg(value_name = "FAYLKA")]
    filename: Option<PathBuf>,

    #[arg(long, help = "Muuji qaabdhismeedka AST adigoon fulin barnaamijka.")]
    ast: bool,

    #[arg(long, help = "Muuji qaabka HIR adigoon fulin barnaamijka.")]
    hir: bool,

    #[arg(long, alias = "adag", help = "Daar hubinta adag ee noocyada xogta.")]
    strict: bool,

    #[arg(
        long,
        alias = "midab-dami",
        help = "Ha isticmaalin midab marka khalad la soo bandhigayo; waxay la mid tahay NO_COLOR=1."
    )]
    no_color: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    arguments: Vec<String>,
}

fn canonical_action(argument: &str) -> Option<&'static str> {
    if matches!(
        argument,
        "--help" | "--caawimo" | "--version" | "--nooc" | "--example" | "--tusaale" | "--amar"
    ) {
        return None;
    }
    let name = argument.strip_prefix("--").unwrap_or(argument);
    if name == "cusbooneysii" {
        return Some("update");
    }
    if name == "amar" {
        return Some("eval");
    }
    soplang::cli::launcher::find_command(name).map(|spec| spec.action)
}

fn run_init() {
    let args = std::iter::once(std::ffi::OsString::from("soplang init"))
        .chain(std::env::args_os().skip(2));
    let options = match soplang::cli::project::InitOptions::try_parse_from(args) {
        Ok(options) => options,
        Err(error) => {
            if error.use_stderr() {
                eprint!(
                    "{}",
                    arguments::format_argument_error(&error)
                        .replace(arguments::USAGE, soplang::cli::project::INIT_USAGE)
                );
            } else {
                let _ = error.print();
            }
            process::exit(error.exit_code());
        }
    };
    match soplang::cli::project::initialize(&options) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{}", format_error_with_source(&error, None));
            process::exit(1);
        }
    }
}

fn show_action_help() {
    let args: Vec<_> = std::env::args_os().skip(2).collect();
    if args.is_empty() || args == ["--help"] || args == ["--caawimo"] || args == ["-h"] {
        let _ = Cli::command().print_help();
        println!();
        return;
    }
    if args.len() != 1 {
        eprintln!("Khalad: Adeegsiga: soplang help [AMAR]");
        process::exit(2);
    }
    let action = args[0]
        .to_str()
        .and_then(canonical_action)
        .unwrap_or_else(|| {
            eprintln!("Khalad: Amarka lama aqoonsan.");
            process::exit(2);
        });
    if action == "init" {
        let _ = soplang::cli::project::InitOptions::command().print_help();
        println!();
    } else if let Some(help) = soplang::cli::tools::command_help(action) {
        println!("{help}");
    } else {
        println!("{}", simple_action_help(action));
    }
}

fn simple_action_help(action: &str) -> &'static str {
    match action {
        "repl" => "Adeegsiga: soplang repl [--strict]\nFur fadhi isdhexgal ah.",
        "eval" => "Adeegsiga: soplang eval QORAAL [--strict]\nFuli qoraalka Soplang ee la bixiyey.",
        "example" => "Adeegsiga: soplang example LAMBARKA [--interactive]\nFuli tusaale ka mid ah examples/.",
        "inspect" => "Adeegsiga: soplang inspect FAYLKA (--ast | --hir)\nMuuji qaabdhismeedka barnaamijka adigoon fulin.",
        "publish" => "Adeegsiga: soplang publish\nDaabac xirmada ku jirta galka hadda.",
        "version" => "Adeegsiga: soplang version\nMuuji nooca Soplang.",
        "help" => "Adeegsiga: soplang help [AMAR]\nMuuji caawimada Soplang ama amar gaar ah.",
        _ => "Adeegsiga: soplang --help",
    }
}

fn run_simple_action(action: &str) {
    let args: Vec<String> = std::env::args_os()
        .skip(2)
        .map(|arg| {
            arg.into_string().unwrap_or_else(|_| {
                eprintln!("Khalad: Doodaha amarku ma aha qoraal UTF-8 ah oo sax ah.");
                process::exit(2);
            })
        })
        .collect();
    if args == ["--help"] || args == ["--caawimo"] || args == ["-h"] {
        println!("{}", simple_action_help(action));
        return;
    }
    let mut strict = false;
    let mut interactive = false;
    let mut ast = false;
    let mut hir = false;
    let mut positional = Vec::new();
    let mut after_separator = false;
    for arg in args {
        if arg == "--" && !after_separator {
            after_separator = true;
            continue;
        }
        if !after_separator {
            match arg.as_str() {
                "--strict" | "--adag" => {
                    strict = true;
                    continue;
                }
                "--interactive" | "--isdhexgal" | "-i" if action == "example" => {
                    interactive = true;
                    continue;
                }
                "--ast" if action == "inspect" => {
                    ast = true;
                    continue;
                }
                "--hir" if action == "inspect" => {
                    hir = true;
                    continue;
                }
                "--no-color" | "--midab-dami" => {
                    std::env::set_var("NO_COLOR", "1");
                    continue;
                }
                _ if arg.starts_with('-') => simple_action_error(action),
                _ => {}
            }
        }
        positional.push(arg);
    }
    match action {
        "repl" if positional.is_empty() => run_shell(strict),
        "eval" if positional.len() == 1 => {
            let code = &positional[0];
            if let Err(error) = run_source(code, None, false, false, strict) {
                eprintln!("{}", format_error_with_source(&error, Some(code)));
                process::exit(1);
            }
        }
        "example" if positional.len() == 1 => {
            let number: usize = positional[0]
                .parse()
                .unwrap_or_else(|_| simple_action_error(action));
            let path = example_path(number).unwrap_or_else(|| {
                eprintln!(
                    "Khalad: Tusaale {} ma jiro (faylka examples/ kuma jiro)",
                    number
                );
                process::exit(1);
            });
            run_action_file(&path, false, false, strict);
            if interactive {
                run_shell(strict);
            }
        }
        "inspect" if positional.len() == 1 && ast != hir => {
            run_action_file(&PathBuf::from(&positional[0]), ast, hir, strict);
        }
        _ => simple_action_error(action),
    }
}

fn simple_action_error(action: &str) -> ! {
    eprintln!("Khalad: {}", simple_action_help(action));
    process::exit(2);
}

fn run_action_file(path: &PathBuf, ast: bool, hir: bool, strict: bool) {
    let source = fs::read_to_string(path).unwrap_or_else(|failure| {
        eprintln!(
            "{}",
            format_error_with_source(&soplang::error::file_read_error(path, &failure), None)
        );
        process::exit(1);
    });
    if let Err(error) = run_source(&source, Some(path), ast, hir, strict) {
        eprintln!("{}", format_error_with_source(&error, Some(&source)));
        process::exit(1);
    }
}

fn main() {
    let first_argument = std::env::args_os()
        .nth(1)
        .and_then(|arg| arg.into_string().ok());
    let second_argument = std::env::args_os()
        .nth(2)
        .and_then(|arg| arg.into_string().ok());
    let action = match (first_argument.as_deref(), second_argument.as_deref()) {
        (Some("--example" | "--tusaale"), Some("--help" | "--caawimo" | "-h")) => Some("example"),
        (Some("--amar"), Some("--help" | "--caawimo" | "-h")) => Some("eval"),
        (Some(first), _) => canonical_action(first),
        _ => None,
    };
    if let Some(command) = action {
        match command {
            "init" => {
                run_init();
                return;
            }
            "help" => {
                show_action_help();
                return;
            }
            "version" => {
                let rest: Vec<_> = std::env::args_os().skip(2).collect();
                if rest.is_empty() {
                    println!("soplang {}", env!("CARGO_PKG_VERSION"));
                } else if rest == ["--help"] || rest == ["--caawimo"] || rest == ["-h"] {
                    println!("{}", simple_action_help("version"));
                } else {
                    simple_action_error("version");
                }
                return;
            }
            "repl" | "eval" | "example" | "inspect" => {
                run_simple_action(command);
                return;
            }
            "publish" => {
                let rest: Vec<_> = std::env::args_os().skip(2).collect();
                if rest == ["--help"] || rest == ["--caawimo"] || rest == ["-h"] {
                    println!("Adeegsiga: soplang publish\nDaabac xirmada ku jirta galka hadda.");
                    return;
                }
                if !rest.is_empty() {
                    eprintln!("Khalad: Adeegsiga: soplang publish");
                    process::exit(2);
                }
                let dir = std::env::current_dir().unwrap_or_else(|error| {
                    eprintln!("Khalad: {}", soplang::error::io_reason(&error));
                    process::exit(1);
                });
                match soplang::package::publish_package(&dir) {
                    Ok(message) => println!("{message}"),
                    Err(error) => {
                        eprintln!("{}", format_error_with_source(&error, None));
                        process::exit(1);
                    }
                }
                return;
            }
            _ => match soplang::cli::tools::dispatch(command) {
                Ok(code) => process::exit(code),
                Err(error) => {
                    eprintln!("{}", format_error_with_source(&error, None));
                    process::exit(1);
                }
            },
        }
    }
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

    if cli.offline {
        std::env::set_var("SOPLANG_OFFLINE", "1");
    }
    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }
    set_program_arguments(user_arguments(&cli));

    if let Some(result) = package_command(&cli) {
        match result {
            Ok(message) => {
                println!("{message}");
                return;
            }
            Err(error) => {
                eprintln!("{}", format_error_with_source(&error, None));
                process::exit(1);
            }
        }
    }

    if cli.build {
        if cli.command.is_some()
            || cli.file.is_some()
            || cli.example.is_some()
            || cli.ast
            || cli.hir
            || cli.interactive
        {
            eprintln!("Khalad: Dhisidda lama wada adeegsan karo fulinta ama muujinta AST/HIR.");
            process::exit(2);
        }
        let mut args = Vec::new();
        if let Some(path) = &cli.filename {
            args.push(path.display().to_string());
        }
        args.extend(cli.arguments.iter().cloned());
        if let Some(out) = &cli.output {
            args.extend(["--output".to_owned(), out.display().to_string()]);
        }
        if let Some(name) = &cli.name {
            args.extend(["--name".to_owned(), name.clone()]);
        }
        args.extend(["--opt-level".to_owned(), cli.opt_level.to_string()]);
        if cli.strict {
            args.push("--strict".to_owned());
        }
        if cli.quiet {
            args.push("--quiet".to_owned());
        }
        match soplang::cli::tools::dispatch_with_args("build", args) {
            Ok(_) => return,
            Err(error) => {
                eprintln!("{}", format_error_with_source(&error, None));
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

    if std::env::args_os().len() == 1 && soplang::cli::launcher::should_show() {
        if let Err(error) = soplang::cli::launcher::run() {
            eprintln!("Khalad: Tilmaamaha amarada lama furi karin: {error}");
            process::exit(1);
        }
    } else {
        run_shell(cli.strict);
    }
}

fn package_command(cli: &Cli) -> Option<Result<String, soplang::SoplangError>> {
    let selected = [
        cli.ku_dar.is_some(),
        cli.cusbooneysii,
        cli.ka_saar.is_some(),
        cli.daabac,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected == 0 {
        return None;
    }
    if selected > 1
        || cli.build
        || cli.command.is_some()
        || cli.file.is_some()
        || cli.example.is_some()
        || cli.filename.is_some()
        || cli.ast
        || cli.hir
    {
        return Some(Err(soplang::error::runtime_error(
            "Amarada xirmada waa inay keli kacaan.",
            0,
            0,
        )));
    }
    let dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(error) => {
            return Some(Err(soplang::error::runtime_error(
                format!(
                    "Galka hadda lama akhriyi karo. {}",
                    soplang::error::io_reason(&error)
                ),
                0,
                0,
            )));
        }
    };
    if let Some(name) = &cli.ku_dar {
        let spec = cli.qeexid.as_deref().unwrap_or("");
        return Some(soplang::package::add_dependency(&dir, name, spec));
    }
    if let Some(name) = &cli.ka_saar {
        return Some(soplang::package::remove_dependency(&dir, name));
    }
    if cli.cusbooneysii {
        return Some(soplang::package::update_dependencies(&dir));
    }
    Some(soplang::package::publish_package(&dir))
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
    let positional_is_program =
        cli.command.is_none() && cli.file.is_none() && cli.example.is_none() && !cli.build;
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
