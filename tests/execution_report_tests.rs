//! Test-body boundaries and instrumentation must not confuse independent runs.

use soplang::{frontend::parse_source, run_statements_report, ExecutionOptions};

#[test]
fn reports_are_owned_bounded_and_do_not_leak_between_runs() {
    let source = "hawl inner() { celi 1 }\nhawl outer() { celi inner() }\ndoor n = 0\nintay (n < 500) {\n n = n + outer()\n}\nhaddii (been) {\n qor(99)\n}\n";
    let path = std::path::Path::new("report.sop");
    let stmts = parse_source(source, Some(path)).unwrap();
    let first = run_statements_report(
        &stmts,
        false,
        ExecutionOptions {
            coverage: true,
            profile: true,
        },
    )
    .unwrap();
    assert!(first.error.is_none(), "{:?}", first.error);
    let report = first.report;
    let executable = &report.coverage.executable[path];
    let hit = &report.coverage.hit[path];
    assert!(executable.contains(&8));
    assert!(!hit.contains(&8));
    assert!(hit.contains(&5));
    assert!(
        hit.len() <= source.lines().count(),
        "loop hits must not grow the report"
    );
    let outer = &report.profile["outer"];
    let inner = &report.profile["inner"];
    assert_eq!(outer.calls, 500);
    assert_eq!(inner.calls, 500);
    assert!(outer.inclusive > outer.exclusive);
    assert!(inner.inclusive <= outer.inclusive);
    assert!(report.execution_time >= outer.inclusive);
    assert!(!report.compile_time.is_zero());

    let other = parse_source("door value = 1", Some(std::path::Path::new("other.sop"))).unwrap();
    let second = run_statements_report(&other, false, ExecutionOptions::default()).unwrap();
    assert!(second.report.profile.is_empty());
    assert!(second.report.coverage.executable.is_empty());
    assert!(second.report.coverage.hit.is_empty());
    assert_eq!(report.profile["outer"].calls, 500);
}

#[test]
fn report_configuration_is_context_local() {
    use soplang::runtime::{self, RuntimeContext};
    let first = RuntimeContext::new();
    let second = RuntimeContext::new();
    first.configure_reporting(Default::default(), true);
    first
        .execute(|| {
            runtime::soplang_enter_function(b"outer".as_ptr(), 5);
            second.configure_reporting(Default::default(), true);
            second
                .execute(|| {
                    runtime::soplang_enter_function(b"inner".as_ptr(), 5);
                    runtime::soplang_leave_function();
                })
                .unwrap();
            runtime::soplang_leave_function();
        })
        .unwrap();
    assert_eq!(first.execution_report().profile["outer"].calls, 1);
    assert!(!first.execution_report().profile.contains_key("inner"));
    assert_eq!(second.execution_report().profile["inner"].calls, 1);
    assert!(!second.execution_report().profile.contains_key("outer"));
}
