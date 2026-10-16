use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser as ClapParser;

use soplang::{build_source, cli::Shell, format_error_with_source, run_source};

#[derive(ClapParser)]
#[command(
    name = "soplang",
    about = "The Somali Programming Language",
    version,
    after_help = "With no arguments, starts the interactive REPL.\n  Use -i after a file/example to run it then continue in the REPL."
)]
struct Cli {
    #[arg(
        long,
        help = "Build .sop file into standalone executable (AOT); requires FILENAME"
    )]
    build: bool,

    #[arg(short = 'o', long, help = "Output binary path (used with --build)")]
    output: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = 2,
        help = "AOT optimization level: 0..3 (used with --build)"
    )]
    opt_level: u8,

    #[arg(
        short,
        long,
        help = "Quiet: no build success message (used with --build)"
    )]
    quiet: bool,

    #[arg(short, long, help = "Execute code snippet and exit")]
    command: Option<String>,

    #[arg(short, long, help = "Execute file and exit")]
    file: Option<PathBuf>,

    #[arg(short, long, help = "Run example N from examples/ (1-based)")]
    example: Option<usize>,

    #[arg(
        short,
        long,
        help = "After running file/example, open interactive REPL"
    )]
    interactive: bool,

    #[arg(help = "Path to .sop file (to run, or to build when using --build; omit to start REPL)")]
    filename: Option<PathBuf>,

    #[arg(long, help = "Print AST instead of executing")]
    ast: bool,

    #[arg(long, help = "Dump HIR (High-Level IR) instead of executing")]
    hir: bool,

    #[arg(long, help = "Enable strict static type mode")]
    strict: bool,

    #[arg(long, help = "Disable colored error output (same as NO_COLOR=1)")]
    no_color: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }

    if cli.build {
        let path = match &cli.filename {
            Some(p) => p.clone(),
            None => {
                eprintln!("Khalad: --build waxa ay u baahan tahay magaca faylka: soplang --build <fayl.sop> [-o <output>] [-q]");
                process::exit(1);
            }
        };
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Khalad: Ma suurtagelin in la akhriyo faylka '{}': {}",
                    path.display(),
                    e
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

        let result = build_handle.join().expect("build thread panicked");
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
                    "Khalad: Ma suurtagelin in la akhriyo faylka '{}': {}",
                    path.display(),
                    e
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
                eprintln!("Khalad: Ma suurtagelin in la akhriyo faylka: {}", e);
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
                    "Khalad: Ma suurtagelin in la akhriyo faylka '{}': {}",
                    path.display(),
                    e
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
    let mut sh = Shell::new(strict);
    sh.run();
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
