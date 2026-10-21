//! Execution reports contain owned diagnostic data, never runtime handles.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::context::RuntimeLocal;
use crate::error::SoplangError;

pub type CoverageMap = BTreeMap<PathBuf, BTreeSet<usize>>;

#[derive(Clone, Debug, Default)]
pub struct CoverageReport {
    pub executable: CoverageMap,
    pub hit: CoverageMap,
}

impl CoverageReport {
    pub fn merge(&mut self, other: &Self) {
        for (target, source) in [
            (&mut self.executable, &other.executable),
            (&mut self.hit, &other.hit),
        ] {
            for (file, lines) in source {
                target.entry(file.clone()).or_default().extend(lines);
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FunctionProfile {
    pub calls: u64,
    /// Includes time spent in callees.
    pub inclusive: Duration,
    /// Excludes time spent in other compiled functions.
    pub exclusive: Duration,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionReport {
    pub body_error: Option<SoplangError>,
    pub coverage: CoverageReport,
    pub profile: BTreeMap<String, FunctionProfile>,
    pub compile_time: Duration,
    pub execution_time: Duration,
}

#[derive(Default)]
pub(super) struct Reporting {
    pub report: ExecutionReport,
    pub profiling: bool,
    pub frames: Vec<ProfileFrame>,
}

pub(super) struct ProfileFrame {
    name: String,
    start: Instant,
    children: Duration,
}

static REPORTING: RuntimeLocal<std::cell::RefCell<Reporting>> =
    RuntimeLocal::new(|state| &state.reporting);

pub(super) fn enter(name: &str) {
    REPORTING.with(|state| {
        let mut state = state.borrow_mut();
        if state.profiling {
            state.frames.push(ProfileFrame {
                name: name.into(),
                start: Instant::now(),
                children: Duration::ZERO,
            });
        }
    });
}

pub(super) fn leave() {
    REPORTING.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(frame) = state.frames.pop() {
            let elapsed = frame.start.elapsed();
            if let Some(parent) = state.frames.last_mut() {
                parent.children += elapsed;
            }
            let profile = state.report.profile.entry(frame.name).or_default();
            profile.calls += 1;
            profile.inclusive += elapsed;
            profile.exclusive += elapsed.saturating_sub(frame.children);
        }
    });
}

pub fn set_profile(enabled: bool) {
    REPORTING.with(|state| {
        let mut state = state.borrow_mut();
        state.profiling = enabled;
        state.frames.clear();
        state.report.profile.clear();
    });
}

pub fn profile_counts() -> Vec<(String, u64)> {
    REPORTING.with(|state| {
        let mut rows: Vec<_> = state
            .borrow()
            .report
            .profile
            .iter()
            .map(|(name, profile)| (name.clone(), profile.calls))
            .collect();
        rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        rows
    })
}

pub(crate) fn record_test_error(error: SoplangError) {
    REPORTING.with(|state| state.borrow_mut().report.body_error = Some(error));
}

pub(crate) fn record_hit(file: PathBuf, line: usize) {
    REPORTING.with(|state| {
        let mut state = state.borrow_mut();
        // Only declared executable lines may enter the report. Repeated hits
        // consume no additional memory, including inside long-running loops.
        if state
            .report
            .coverage
            .executable
            .get(&file)
            .is_some_and(|lines| lines.contains(&line))
        {
            state
                .report
                .coverage
                .hit
                .entry(file)
                .or_default()
                .insert(line);
        }
    });
}

/// Compatibility with the original instrumentation helper. New compiler output
/// always supplies the source identity explicitly.
pub fn record_coverage(line: usize) {
    let file = RuntimeLocal::new(|state| &state.current_source)
        .with(|source| source.borrow().as_ref().and_then(|span| span.file.clone()));
    if let Some(file) = file {
        record_hit(file, line);
    }
}

pub fn coverage_lines() -> Vec<usize> {
    REPORTING.with(|state| {
        state
            .borrow()
            .report
            .coverage
            .hit
            .values()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    })
}

pub fn clear_coverage() {
    REPORTING.with(|state| state.borrow_mut().report.coverage.hit.clear());
}
