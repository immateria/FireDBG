# Upstream PR Notes
This fork has accumulated improvements that are generally useful but may be best maintained upstream.

This doc is a note to future-you on how to split changes into reviewable PRs.

## Candidate PR 1: CLI UX + Diagnostics
Scope:
- `firedbg doctor` subcommand
  - environment summary (rust/cargo versions, workspace root)
  - LLDB discovery output (lldb binary, python dir, debugserver)
  - installed binary presence checks
  - runtime spawn check for `firedbg-debugger --help`
  - actionable hints
- Replace `panic!` on build failures with `anyhow::bail!` messages

Files involved:
- `command/src/main.rs`
- `command/src/console.rs` (if changes are needed)

Suggested PR split:
- PR A: add `doctor` + hints
- PR B: error message improvements (replace panics)

## Candidate PR 2: Installer improvements
Scope:
- Make `install.sh` source-install friendly
- Keep downloaded CodeLLDB / `lldb` runtime assets out of the checkout
- Use cache dir selection and `--clean-cache`

Files involved:
- `install.sh`
- `INSTALL.md` / `BUILDING.md` (optional)

Note:
- Upstream might prefer keeping prebuilt-first; this fork prefers source-first.
  If upstream won’t accept that policy change, still consider proposing:
  - “don’t leave artifacts in checkout”
  - “cache directory support”

## Candidate PR 3: Parser determinism
Scope:
- Make `parse_workspace()` output deterministic
  - stable ordering of packages / deps / targets
  - prefer the primary binary over `src/bin/*` for display
- Make tests less toolchain-sensitive
  - avoid asserting on `end.column` where it drifts
  - skip workspace fixtures that are not vendored

Files involved:
- `parser/src/parsing/workspace.rs`
- `parser/tests/*`

Rationale:
- Deterministic target ordering improves UX in `firedbg list-target`.

## What I would NOT upstream (unless asked)
- Fork-specific policy docs (e.g. strongly recommending source-only builds) if it conflicts with upstream release strategy.

## Practical advice
- Keep PRs small and logically isolated.
- Avoid mixing docs-only changes with code changes, unless the doc is required to explain new behavior.
