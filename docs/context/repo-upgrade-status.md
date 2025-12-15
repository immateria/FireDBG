# Repo upgrade status (immateria fork)
This document is a persistent, repo-local log of the ongoing upgrade work and current blockers, intended to survive local reboots and lost chat context.

Last updated: 2025-12-15

## High-level goal
Keep the fork “build-it-yourself” while improving UX/diagnostics/docs and reducing support burden.

## What’s done (recent)
- Installer redesign: source-first install, CodeLLDB assets cached under user cache dir, no repo-root artifacts.
- Added `firedbg doctor` with actionable diagnostics + JSON output.
- Added docs under `docs/context/` (architecture notes, smoke-test workflow, upstream PR splitting notes).
- Added `examples/smoke-workspace/` minimal workspace for smoke testing.
- Parser determinism/test robustness fixes.
- Feature-gated flaky LLDB integration tests behind `lldb-tests`.

## What’s in progress / recently changed locally (not yet pushed)
- Improved LLDB debugserver discovery and selection.
- `firedbg run`/`index` now fail fast when debugger/indexer exit non-zero.
- New workspace-local status logs written to `<workspace>/firedbg/status/*.json`:
  - `*-doctor.json`
  - `*-run-pre.json` (exact command + key env)
  - `*-run-post.json` (exit status)
- `firedbg doctor` now reports macOS Developer Mode status (via `DevToolsSecurity -status`) and provides a hint when disabled.

## Current blocker (end-to-end run)
End-to-end `firedbg run` on macOS still fails to launch the debugger.

Observed failures (examples)
- “the platform is not currently connected”
- “process exited with status -1 (debugserver is x86_64 binary running in translation, attach failed.)”

Interpretation
- This points to a host configuration / architecture / entitlement mismatch (debugserver running under translation / Developer Mode disabled), not just a missing-path issue.

## Environment checks that matter
- macOS Developer Mode (host permission gate):
  - Symptoms when it’s off can include LLDB failing to launch/attach (e.g. “platform is not currently connected”).
  - Check status:
    - `DevToolsSecurity -status`
  - Enable (CLI):
    - `sudo /usr/sbin/DevToolsSecurity -enable`
  - Enable (UI):
    - System Settings → Privacy & Security → Developer Mode → enable (often requires reboot)
  - Verify again after enabling:
    - `DevToolsSecurity -status`
- Architecture alignment:
  - Ensure `firedbg-debugger`, the target binary, liblldb, and debugserver are compatible and not unintentionally running under translation.

## Next steps
1. Enable Developer Mode:
   - System Settings → Privacy & Security → Developer Mode → enable
   - and/or run: `sudo /usr/sbin/DevToolsSecurity -enable`
   - reboot if macOS prompts you to.
2. Re-run diagnostics:
   - `firedbg doctor --deep`
   - confirm `macos_developer_mode_enabled: true`.
3. Re-run the smoke test:
   - `firedbg run` on `examples/smoke-workspace`
   - inspect the newest `firedbg/status/*-run-pre.json` for:
     - `LLDB_DEBUGSERVER_PATH`
     - `DYLD_FALLBACK_LIBRARY_PATH`
     - `PYTHONPATH`
     - detected LLDB python/debugserver paths
4. If the error still references “running in translation”, focus on ensuring the full stack is native for the machine (Rust toolchain arch, built binaries, and debugserver slice).

## Notes
- Status logs are intentionally ignored by git via `.gitignore` (`**/firedbg/status/`).
- This doc should be updated whenever we change installer/doctor/debugserver behavior, or when a new blocker is identified.
