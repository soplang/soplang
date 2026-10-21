//! Callable annotations preserve function values and enforce shared contracts.

use soplang::{analyze_with_options, frontend::parse_source, AnalyzeOptions};

#[test]
fn strict_callable_values_accept_named_native_and_captured_functions() {
    soplang::run_source(
        include_str!("fixtures/callable_types.sop"),
        None,
        false,
        false,
        true,
    )
    .unwrap();
}

#[test]
fn callable_literals_and_invalid_type_arguments_are_checked_before_execution() {
    for source in [
        "hawl value = 7",
        "hawl consume(hawl cb): maran {}\nconsume(7)",
        "urur[hawl] values = urur()",
    ] {
        let parsed = parse_source(source, None).unwrap();
        assert!(
            analyze_with_options(
                &parsed,
                AnalyzeOptions {
                    strict: false,
                    repl: false
                }
            )
            .is_err(),
            "{source}"
        );
    }
    for source in [
        "hawl f(): maran {}",
        "hawl() : maran {}",
        "hawl? f = maran",
        "hawl | maran f = maran",
        "madoor hawl f = qor",
    ] {
        let parsed = parse_source(source, None).unwrap();
        analyze_with_options(
            &parsed,
            AnalyzeOptions {
                strict: true,
                repl: false,
            },
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
}
