# Somali CLI output audit and patch release plan

Status: Compiler implementation prepared and local checks passed. See the
[3.0.1 verification record](../testing/LOCALIZATION_3_0_1.md). Publication and
website deployment remain pending the release gates.

Audit date: 2026-10-18.

## Baseline and release scope

- Compiler: committed `main` at `e72c6668276fa801691ddb1c753ebbe2f73a417c`.
  GitHub main matched this revision during the audit.
- Website: `/home/sharafdin/Documents/Goobo-Labs/soplang-site`, main at
  `3bb54c3`.
- Latest published GitHub release: `v3.0.0`, published 2026-10-17. Checked
  with `gh release list --repo soplang/soplang` on the audit date.
- Suggested release: **v3.0.1**, provided the final changes remain compatible
  diagnostic and documentation fixes. Recheck available tags before release.
- Existing uncommitted compiler analysis changes are outside this patch:
  `src/frontend/mod.rs`, `src/frontend/parser.rs`, `src/lib.rs`,
  `src/analysis.rs`, and `src/analysis/`. Preserve them. Prepare the eventual
  patch from committed source in an isolated checkout if they remain unfinished.

The release stays in Soplang 3. Existing flags, keywords, aliases, APIs,
program results, exit statuses, runtime ABI, and compiler backends retain their
behavior. Documentation prose remains English. CLI help and diagnostics use
Somali. The localization work does not introduce file or networking APIs.

[Semantic Versioning](https://semver.org/spec/v2.0.0.html) assigns compatible
bug fixes to the patch component. Localization fixes fit 3.0.1; unrelated new
language capabilities need their own scope and version decision.

## Verified findings

The three reported categories are present in source and observable output.
The runtime checks below used the existing local `target/release/soplang`,
which reports `soplang 3.0.0`. They were not a rebuild of the dirty working
tree or a fresh verification of downloaded release packages.

| Area | Source | Audit result |
| --- | --- | --- |
| Help descriptions, about text, footer | [main.rs](../../src/main.rs) | All reported English strings confirmed; `--help` reproduced them. |
| Generated help labels and invalid arguments | `Cli::parse()` and locked Clap 4.5.59 | English `Usage`, `Arguments`, `Options`, placeholders, defaults, help/version descriptions, parser errors, hints, and integer-parser reasons reproduced. |
| HIR wrapper heading | [lib.rs](../../src/lib.rs) | `--- top_level ---` reproduced with `--hir`. IR instructions also use compiler notation. |
| LLVM tool and runtime discovery messages | [llvm_ir.rs](../../src/backend/llvm_ir.rs) | Every reported English message confirmed in source. Missing metadata, missing clang/rustc, toolchain mismatch, and failed-tool output reproduced. |
| JIT startup and compilation failures | [cranelift.rs](../../src/backend/cranelift.rs) | `main not compiled`, `main not finalized`, and raw error forwarding confirmed in source. These internal failures were not induced through normal programs. |
| Invalid runtime tag | [abi.rs](../../src/runtime/abi.rs) | `Invalid tag` confirmed in source. Not induced through valid Soplang source. |
| Script and REPL file reads | [main.rs](../../src/main.rs), [shell.rs](../../src/cli/shell.rs) | Missing-file English tails reproduced in both interfaces; directory and invalid UTF-8 errors reproduced through script execution. |
| REPL editor failures | [shell.rs](../../src/cli/shell.rs) | Both ordinary and continuation-input errors forward rustyline text. Source-confirmed; editor faults not injected. |
| `gelin` input and output failures | [stdlib.rs](../../src/runtime/stdlib.rs) | Raw I/O errors confirmed. Invalid UTF-8 on standard input reproduced with a retained source caret. |
| Build scratch files and output publication | [llvm_ir.rs](../../src/backend/llvm_ir.rs) | Directory, write, rename, copy, and executable-location errors forward raw errors. Source-confirmed; not every filesystem failure induced. |

### Reproduction coverage

Seventeen subprocess checks were run, including two controls (`--version`
and successful HIR emission). Temporary files and fake tool executables stayed
under `/tmp`; no repository source was executed or changed by these fixtures.

| Check | Observed status | Observation |
| --- | --- | --- |
| `--version` | 0 | `soplang 3.0.0` |
| `--help` | 0 | English help on stdout |
| Unknown option | 2 | English error and usage on stderr |
| Missing `--file` value | 2 | English missing-value explanation |
| `--opt-level abc` | 2 | English argument error and Rust integer-parse reason |
| Missing script | 1 | Somali prefix followed by `No such file or directory` |
| Directory as script | 1 | Somali prefix followed by `Is a directory` |
| Non-UTF-8 script | 1 | Somali prefix followed by invalid UTF-8 text |
| `--hir` with `qor(1)` | 0 | English top-level heading |
| Runtime directory without metadata | 1 | English toolchain-read message and OS reason |
| No clang in controlled `PATH` | 1 | English launch failure |
| REPL `/akhrifayl` missing file | 0 on REPL exit | English OS reason; session remains usable |
| Non-UTF-8 input to `gelin()` | 1 | English reason with line 1, column 1 and exact span |
| Deliberately mismatched toolchain metadata | 1 | English mismatch explanation and labels |
| No rustc with packaged-runtime override | 1 | English launch failure and install hint |
| Fixture clang exits unsuccessfully | 1 | English summary and unmodified fixture stderr |
| Fixture rustc exits unsuccessfully | 1 | English summary and unmodified fixture stderr |

The failed-tool checks used deliberately unsuccessful stubs. They verify
forwarding behavior, not a real clang or rustc compilation defect.

### Additional findings

- Build worker failure uses `expect("build thread panicked")`; REPL startup
  uses `expect("rustyline Editor")` and `expect("Cranelift JIT")`. Recoverable
  startup and worker failures should enter the Somali error path. Translating
  these strings alone would leave Rust's panic wrapper visible.
- `qor` and the `gelin` prompt use print macros that can panic on output
  failures. Review these I/O boundaries so a broken pipe does not expose a
  Rust panic. Keep successful output and catchable-error behavior consistent.
- Build errors without source locations currently display `safka 0, tiirka 0`.
  Omit nonexistent locations; retain real source spans and carets unchanged.
- Imports already use Somali, but some read failures are reported as a missing
  file regardless of cause. Reuse the contextual I/O explanation where suitable
  while preserving import error codes and source spans.
- Third-party panics and failures before Soplang starts are outside a promise
  that normal Soplang output is Somali. Document this boundary accurately.

## Language policy and proposed wording

Use short sentences describing the failed action and a useful next step.
Use `fuli` consistently for execution. Retain existing `Khalad` and
`Khalad fulin` headings. Do not introduce new language keywords to translate
CLI prose.

Names such as Soplang, Rust, Cranelift, LLVM, clang, rustc, UTF-8, AST, HIR,
flags, paths, environment variables, error codes, and user-provided text remain
literal. Technical identifiers and quoted program output are not English prose
to rewrite. AST/HIR instruction notation retains its compiler meaning.

Language evidence was reviewed against Somali Language Standard revision
`9bcab5e420d67406cb021c8fcf89df3aeb270675` and the existing
[Soplang terminology review](terminology.md). SLS-0002 orthography and
SLS-0003 grammar are Proposed; computing terminology SLS-0204 and SLS-0205 is
Planned. These proposals do not claim certification by a stable SLS standard.

The dictionary attests `barnaamij`, `xulasho`, and `summad`. This supports their
spelling, not automatic approval of a computing meaning. Its entry for `fayl`
does not define a computer file; keep the established Soplang usage without
claiming otherwise. Review sentence agreement, word boundaries, vowel length,
and doubled consonants rather than applying automatic spelling replacements.

### CLI help proposals

| Element | Proposed Somali |
| --- | --- |
| About | `Luuqadda barnaamijyada ee Af-Soomaaliga.` |
| Usage heading | `Adeegsiga:` |
| Positional section | `Faylka:` |
| Options heading | `Xulashooyinka:` |
| Usage pattern | `soplang [XULASHOOYINKA] [FAYLKA]` |
| `--build` | `Ka dhis faylka .sop barnaamij si madax bannaan u shaqeeya; sheeg faylka la dhisayo.` |
| `--output` | `Meesha lagu kaydinayo barnaamijka la dhisayo (--build).` |
| `--opt-level` | `Heerka hagaajinta koodhka: 0 ilaa 3 (--build; haddii aan la sheegin: 2).` |
| `--quiet` | `Ha muujin farriinta guusha dhismaha (--build).` |
| `--command` | `Fuli qoraalka la bixiyey, dabadeed bax.` |
| `--file` | `Fuli faylka la bixiyey, dabadeed bax.` |
| `--example` | `Fuli tusaalaha lambarkiisu yahay N ee examples/; tirintu waxay ka bilaabataa 1.` |
| `--interactive` | `Fur REPL marka faylka ama tusaalaha la fuliyo.` |
| Filename | `Faylka .sop ee la fulinayo ama la dhisayo; haddii aan la sheegin, fur REPL.` |
| `--ast` | `Muuji qaabdhismeedka AST adigoon fulin barnaamijka.` |
| `--hir` | `Muuji qaabka HIR adigoon fulin barnaamijka.` |
| `--strict` | `Daar hubinta adag ee noocyada xogta.` |
| `--no-color` | `Ha isticmaalin midab marka khalad la soo bandhigayo; waxay la mid tahay NO_COLOR=1.` |
| `--help` | `Muuji caawimada.` |
| `--version` | `Muuji nooca Soplang.` |
| No-argument footer | `Haddii aan waxba la raacin, waxaa furmaya REPL.` |
| Interactive footer | `Adeegso -i si aad REPL ugu sii shaqayso marka faylka ama tusaalaha la fuliyo.` |
| HIR wrapper | `--- qaybta guud ---` |

Use contextual placeholders such as `FAYLKA`, `QORAALKA`, `HEERKA`,
`LAMBARKA`, and `BARNAAMIJKA`. Preserve option spellings and their parsing.
Adjust the REPL banner and `/caawi` descriptions to the same wording style.

### Diagnostic proposals

The following are review candidates, not claims of completed linguistic review.

| Situation | Proposed Somali |
| --- | --- |
| Unknown flag | `Xulashada '--magac' lama aqoonsan. Adeegso --help si aad u aragto caawimada.` |
| Missing option value | `Xulashada '--file' waxay u baahan tahay qiime.` |
| Invalid integer | `Qiimaha 'abc' ee '--opt-level' ma aha tiro dhan oo sax ah.` |
| Missing script | `Faylka 'magac.sop' lama helin.` |
| Permission denied while reading | `Lama oggola in la akhriyo faylka 'magac.sop'.` |
| Directory supplied as file | `'magac' waa gal; waxaa loo baahan yahay fayl.` |
| Non-UTF-8 source file | `Faylka 'magac.sop' ma aha qoraal UTF-8 ah oo sax ah.` |
| Non-UTF-8 `gelin` input | `Xogta la geliyey ma aha qoraal UTF-8 ah oo sax ah.` |
| Other input failure | `Xogta la gelinayo lama akhrin karin.` |
| Output failure | `Waxsoosaarka lama qori karin.` |
| Temporary directory failure | `Galka ku meel gaarka ah lama abuuri karin.` |
| File write failure | `Faylka 'jidka' lama qori karin.` |
| Output publication failure | `Barnaamijka la dhisay laguma kaydin karin 'jidka'.` |
| Executable location failure | `Meesha uu ku jiro barnaamijka Soplang lama ogaan karin.` |
| Runtime metadata unreadable | `Xogta nooca Rust ee ku jirta 'jidka/rustc-version.txt' lama akhrin karin.` |
| clang launch failure | `clang lama bilaabi karin. Hubi in uu rakiban yahay oo laga heli karo PATH.` |
| clang compilation failure | `clang ma dhammaystiri karin ururinta LLVM IR.` |
| rustc launch failure | `rustc lama bilaabi karin. Rakib nooca Rust ee ku qoran 'jidka/rustc-version.txt'.` |
| rustc link failure | `rustc ma dhammaystiri karin isku xidhka barnaamijka.` |
| Toolchain mismatch | `Nooca Rust ee rakiban kuma habboona maktabadaha Soplang.` |
| Mismatch labels | `Nooca loo baahan yahay:` and `Nooca la helay:` |
| Mismatch remedy | `Rakib nooca Rust ee loo baahan yahay ama dib u dhis xirmadan Soplang.` |
| Incomplete runtime | `Qaybaha fulinta Soplang ee ku jira 'jidka' ma dhammaystirna.` |
| Missing runtime libraries | `Maktabadaha fulinta Soplang lama helin. Rakib xirmada oo dhammaystiran ama SOPLANG_RUNTIME_DIR u deji galkeeda lib/soplang.` |
| JIT startup failure | `Cranelift ma bilaabi karin ururinta barnaamijka.` |
| JIT compilation failure | `Cranelift ma dhammaystiri karin ururinta barnaamijka.` |
| main not compiled | `Hawsha 'main' weli lama ururin.` |
| main not finalized | `Hawsha 'main' weli diyaar uma aha fulin.` |
| Invalid tag | `Summadda qiimaha ma saxna: N.` |
| REPL editor failure | `Qoraalka amarka lama akhrin karin.` |

When the cause is known, add a corresponding Somali reason: permission,
missing file or tool, invalid data, full storage, read-only storage, and so on.
Use `io::ErrorKind` and operation context. Do not compare English `Display`
strings, assume a missing tool for every launch failure, or claim a file is
absent when permissions or encoding caused the failure. Unknown causes receive
a truthful general Somali explanation and the OS error number when available.

### External diagnostic details: proposed policy

Default explanations and recovery hints are Somali. Preserve external error
objects, tool exit status, and raw stderr separately as diagnostic details.
Do not translate arbitrary rustc, clang, Cranelift, or OS text using string
replacement. Do not delete the evidence needed to investigate a failure.

Proposed troubleshooting control: `SOPLANG_ERROR_DETAILS=1` explicitly includes
the original technical details under `Faahfaahin farsamo:` on stderr. These
details may be English. Without that setting, English tool prose is absent
from normal diagnostic explanations. This control is implemented with the reviewed localization changes.

Keep raw details out of the primary runtime message so caught errors have the
same Somali explanation in JIT and LLVM executables. Test that distinction.

## Implementation sequence after approval

### 1. Compiler localization and tests

1. Prepare a clean patch checkout and inspect the final committed baseline.
   Keep unfinished analysis work out of the release.
2. Add shared contextual I/O and backend-error helpers. Preserve structured
   codes, spans, hints, paths, and raw troubleshooting details.
3. Translate help and REPL prose. Configure Clap headings, placeholders,
   help/version descriptions, and default-value text explicitly.
4. Replace unconditional `Cli::parse()` error printing with structured
   handling of Clap error kinds and argument context. Cover all reachable
   parse failures and a Somali fallback. Preserve `--help`/`--version` stdout
   and status 0, and invalid-argument stderr and status 2.
5. Use a custom help template/usage and hide Clap's English default marker
   where its value is already explained in Somali. Localize error formatting
   without forwarding Clap's formatted English usage or parser reason.
   The locked Clap exposes `help_template`, `override_usage`, argument
   `help_heading`, `value_name`, `hide_default_value`, and `ErrorFormatter`.
   See the [Clap API](https://docs.rs/clap/latest/clap/struct.Command.html);
   verify behavior against locked 4.5.59 when implementing.
6. Update LLVM, JIT, runtime, file, and line-editor failure boundaries.
   Handle recoverable worker/editor initialization failures without panic.
   Keep native compilation and runtime behavior intact.
7. Localize the HIR wrapper, retain IR notation, and suppress nonexistent
   source locations for infrastructure errors.
8. Update [verify_package.py](../../scripts/verify_package.py): its runtime
   discovery and toolchain-mismatch checks currently require English phrases.
   Change those assertions in the same patch as the diagnostics.

### 2. Verification gates

Add meaningful coverage for help, argument errors, typed OS failures, REPL,
`gelin`, backend failures, and the technical-details policy. Include unknown
options, missing/repeated values, invalid and overflowing integers, invalid
argument encoding where supported, and filenames containing spaces or Unicode.

Use deterministic error-kind tests for platform-specific causes; use process
fixtures for the actual user-visible path. Do not rely on chmod permission
tests under a privileged account or alter global process environment in
parallel tests. Use portable tool fixtures on Windows as well as Unix.

Preserve color control, stdout/stderr separation, exact exit status, real
one-based line and column, Unicode/tab caret alignment, import locations,
and matching caught-error messages in JIT and LLVM. Explicitly test localized
input failures in generated executables as well as the JIT.

During development:

```sh
cargo test --test llvm_ir_tests
cargo test -- --skip test_aot
```

Before release, run the complete gates including AOT:

```sh
cargo test --locked
cargo test --release --locked
```

The existing native CI matrix must pass on Linux, Windows, and macOS, together
with the container job. Build/extract packages, check real installers where
supported, verify checksums, and run the installed-package verifier outside
the checkout. Add localization smoke checks to installed-package verification.
Local Linux results alone do not establish Windows or macOS success.

Format changed Rust files only. Review the final diff for unrelated changes.

### 3. Prepare and commit v3.0.1

1. Bump Cargo.toml and the Soplang package entry in Cargo.lock to 3.0.1.
   Keep dependencies and the Rust toolchain unchanged unless a concrete
   defect requires otherwise.
2. Add an accurate changelog entry and patch-specific release notes. Include
   compatible post-3.0.0 changes already on main. Do not repeat the whole
   3.0.0 feature announcement or label old benchmarks as 3.0.1 measurements.
3. Fix the release workflow's hardcoded `body_path: docs/RELEASE_NOTES_V3.md`
   so the draft uses reviewed 3.0.1 notes. Keep the 3.0.0 announcement and
   [verification record](../testing/RELEASE_VERIFICATION.md) historical.
4. Refresh current-version compiler installation/CLI/Docker documentation and
   platform READMEs. Keep pinned historical and benchmark references intact.
5. Record the exact release revision and green verification results.
   Suggested compiler commit subject: `Localize CLI help and system errors`.
   Provide exact staging and commit commands after the patch is green.

### 4. Build and publish the release

Follow the [release process](../RELEASE_PROCESS.md). A `release/v3.0.1` branch
must contain the reviewed version and notes. Its workflow runs native and
container verification before creating a draft tied to that tested revision.

Review actual archives/installers, their versions, combined `SHA256SUMS`,
runtime metadata, and release notes. Publish the verified draft as stable
`v3.0.1`. Do not replace or retag v3.0.0 assets.

Watch Docker publication, then pull and check the versioned image and stable
aliases. Verify the published image's version, localized help, script run,
and LLVM build. Keep architecture claims limited to verified images. AUR
publication depends on configured credentials; report its actual result.

### 5. Update soplang.org against the published assets

Prepare website edits alongside the patch; publish them after the assets and
referenced images are available. The site change is a separate repository
commit, suggested subject: `Update downloads and docs for Soplang 3.0.1`.

| Website file | Required update |
| --- | --- |
| `constants/latestVersion.ts` | Set the verified stable tag. |
| `lib/releases.ts` | Refresh fallback tag, date, real asset names and sizes from the published release. |
| `lib/github.ts` and `tests/releases.test.ts` | Verify latest stable selection, prerelease exclusion, fallback behavior, and asset mapping. |
| `app/(dashboard)/downloads/page.tsx` | Replace current-version checksum examples and package/toolchain statements using verified metadata. |
| `content/docs/00_installation.mdx` and `index.mdx` | Update package names, links, Docker tags, current version, and verified requirements. |
| `content/docs/19_cli-and-repl.mdx` | Update help and REPL examples to actual Somali output and describe technical-details opt-in in English. |
| `content/docs/17_error-handling.mdx` | Refresh affected diagnostic examples; keep explanatory prose English. |
| `app/llms.txt/route.ts` | Refresh current release/package information. |
| `scripts/check-docs.ts` | Replace the hardcoded 3.0.0 acceptance check with exact validation against the site's intended release. |

Search the remaining site for current-version references, preserving historical
3.0.0 announcements. Use published asset metadata rather than guessed package
sizes. Dynamic GitHub data and fallback data must describe the same release;
the current GitHub fetch has a one-hour revalidation interval.

Run the site's release tests, type checks, lint, production build, and docs
checker against the 3.0.1 compiler with its matching toolchain:

```sh
bun test
bun run types:check
bun run lint
bun run build
bun run docs:check /absolute/path/to/soplang-3.0.1
```

After deployment, check the live download page, every advertised asset URL,
checksum commands, version metadata, installation page, and CLI/error examples.
Confirm GitHub fallback and normal online paths do not mix package versions.
Record any deployment or registry failure instead of marking it complete.

## Approval checkpoints

1. Review the proposed Somali wording and external-details policy.
2. Implement and review the green compiler patch, release metadata, and
   staged website changes.
3. Review native/container gates and the concrete release draft before
   publication, then deploy and verify the website.

The compiler changes and regression tests follow this plan. Release-wide
platform verification, publication, and the website deployment must be recorded
separately when completed.
