#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_FILE="$SCRIPT_DIR/CRITERION_RESULTS.md"

cd "$PROJECT_DIR"

echo "Building release binary..."
cargo build --release --locked 2>&1

echo ""
echo "Running criterion benchmarks..."
BENCH_LOG=$(mktemp)
trap 'rm -f "$BENCH_LOG"' EXIT
cargo bench --locked 2>&1 | tee "$BENCH_LOG"

echo ""
echo "Generating formatted $RESULTS_FILE ..."

TIMESTAMP=$(date -u "+%Y-%m-%d %H:%M:%S UTC")
RUSTC_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version)
UNAME_INFO=$(uname -srm)

# Parse criterion output from the log file (avoids ARG_MAX when output is huge).
# For each "time: [ low mean high unit ]" capture benchmark name and values.
# Skip "comparison/soplang/*" so we keep one row per benchmark.
parse_criterion() {
  awk '
    BEGIN { last_bench = "" }
    /Benchmarking [^:]+: Analyzing/ {
      if (match($0, /Benchmarking [^:]+: Analyzing/)) {
        # name is between "Benchmarking " and ": Analyzing"
        s = substr($0, RSTART + 13, RLENGTH - 24)
        if (s != "") last_bench = s
      }
    }
    /time:[[:space:]]*\[/ {
      name = last_bench
      if (match($0, /[a-z0-9_]+\/[a-z0-9_x]+/)) name = substr($0, RSTART, RLENGTH)
      if (name ~ /^comparison\//) next
      # Criterion format: [ 45.791 ms 47.145 ms 48.808 ms ] or [45.791 ms ...] (low unit mean unit high unit)
      if (match($0, /\[[[:space:]]*[0-9.]+[[:space:]]+[^[:space:]]+[[:space:]]+[0-9.]+[[:space:]]+[^[:space:]]+[[:space:]]+[0-9.]+[[:space:]]+[^]]+\]/)) {
        s = substr($0, RSTART, RLENGTH)
        gsub(/^\[[[:space:]]*|[[:space:]]*\]$/, "", s)
        n = split(s, a, /[[:space:]]+/)
        if (n >= 6) {
          low = a[1]; mean = a[3]; high = a[5]; unit = a[2]
          print name "|" mean "|" low "|" high "|" unit
        }
      }
    }
  ' "$BENCH_LOG"
}

# Build Criterion results table (main 6 benchmarks)
criterion_table=""
criterion_table="$criterion_table\n| Benchmark | Mean | Low | High |"
criterion_table="$criterion_table\n|-----------|------|-----|------|"

while IFS='|' read -r name mean low high unit; do
  [ -z "$name" ] && continue
  # Format numbers: drop trailing zeros, keep 2 decimal places when needed
  case "$unit" in
    *ms*) criterion_table="$criterion_table\n| $name | ${mean} ms | ${low} ms | ${high} ms |" ;;
    *µs*|*us*) criterion_table="$criterion_table\n| $name | ${mean} µs | ${low} µs | ${high} µs |" ;;
    *) criterion_table="$criterion_table\n| $name | ${mean} $unit | ${low} $unit | ${high} $unit |" ;;
  esac
done < <(parse_criterion | grep -E '^(fibonacci|loops|strings|lists|objects)/' | head -6)

# Pipeline stage rows (lex_only, parse_only)
pipeline_rows=""
while IFS='|' read -r name mean low high unit; do
  [ -z "$name" ] && continue
  case "$unit" in
    *ms*) pipeline_rows="${pipeline_rows}\n| $name | ${mean} ms | ${low} ms | ${high} ms |" ;;
    *µs*|*us*) pipeline_rows="${pipeline_rows}\n| $name | ${mean} µs | ${low} µs | ${high} µs |" ;;
    *) pipeline_rows="${pipeline_rows}\n| $name | ${mean} $unit | ${low} $unit | ${high} $unit |" ;;
  esac
done < <(parse_criterion | grep 'pipeline_stages/')

# If parsing produced nothing, show a placeholder
if [ -z "$criterion_table" ] || [ "$(echo -e "$criterion_table" | wc -l)" -le 2 ]; then
  criterion_table="| Benchmark | Mean | Low | High |
|-----------|------|-----|------|
| (run \`cargo bench\` to populate) | — | — | — |"
fi

# Wall-clock timings
WALL_TABLE="| Benchmark | Time |
|-----------|------|"
BINARY="$PROJECT_DIR/target/release/soplang"
for bench_file in "$SCRIPT_DIR"/*.sop; do
  [ -f "$bench_file" ] || continue
  name=$(basename "$bench_file" .sop)
  elapsed=$( { time "$BINARY" "$bench_file" > /dev/null 2>&1; } 2>&1)
  real_time=$(echo "$elapsed" | grep -E 'real|user' | head -1 | awk '{print $2}')
  [ -z "$real_time" ] && real_time="—"
  WALL_TABLE="$WALL_TABLE
| $name | $real_time |"
done

{
  cat <<HEADER
# Soplang Criterion Benchmarks

For the Python/v2/v3 comparison, see [RESULTS.md](RESULTS.md).

> Auto-generated on **$TIMESTAMP**
> System: \`$UNAME_INFO\` | \`$RUSTC_VERSION\` | \`$CARGO_VERSION\`

## Benchmark Programs

| Benchmark | Description | Workload |
|-----------|-------------|----------|
| fib_recursive | Recursive Fibonacci | fib(25) = 75025 |
| loop_sum | Tight for-loop | Sum 1..100,001 inclusive |
| nested_loops | Nested loop dispatch | 201 x 201 iterations |
| string_concat | String concatenation | 1,001 appends |
| list_ops | List push + traversal | 5,001 elements |
| object_create | Object allocation | 2,002 objects |

## Criterion Results (compiler — Cranelift JIT)

In-process: Lex → Parse → HIR → Cranelift JIT → execute (same path as \`soplang file.sop\`).

HEADER
  echo -e "$criterion_table"
  if [ -n "$pipeline_rows" ]; then
    echo ""
    echo "### Pipeline stage breakdown"
    echo ""
    echo "| Stage | Mean | Low | High |"
    echo "|-------|------|-----|------|"
    echo -e "$pipeline_rows"
  fi
  cat <<WALL

## Quick timings (wall-clock)

$WALL_TABLE

## How to run

\`\`\`bash
# Full criterion benchmarks (HTML reports in target/criterion/)
cargo bench

# This script: benchmarks + formatted results
bash benchmarks/run_benchmarks.sh

# Single group
cargo bench -- fibonacci
cargo bench -- loops
\`\`\`

## Notes

- **What we benchmark:** The compiler (Cranelift JIT). Same path as \`./target/release/soplang file.sop\`.
- **Criterion** = in-process, statistical (mean, confidence intervals).
- **HTML reports:** \`target/criterion/report/index.html\`.
WALL
} > "$RESULTS_FILE"

echo ""
echo "Done! Formatted results written to $RESULTS_FILE"
