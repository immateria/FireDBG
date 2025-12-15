## Installing FireDBG

This repository supports a **source-first** installation flow.

- Recommended: build and install from source using `install.sh`.
- Optional: use prebuilt binaries (opt-in; typically from the upstream SeaQL releases).

If anything fails, run `firedbg doctor` for diagnostics and fix hints.

### Source installation (recommended)

1. Clone this repository and run the installer from the repo root:

```shell
./install.sh
```

This will:
- build `firedbg`, `firedbg-debugger`, and `firedbg-indexer` with `cargo build`
- install (symlink) binaries into your cargo bin directory (usually `~/.cargo/bin`)
- download CodeLLDB’s `lldb` runtime bundle **into a user cache directory** (not into your repo checkout)
- point `~/.cargo/bin/firedbg-lib` at that cached `lldb` directory

You can force modes / maintenance actions:

```shell
# Source build (explicit)
./install.sh --source

# Remove cached CodeLLDB assets
./install.sh --clean-cache

# Opt-in: prebuilt binaries (from upstream releases)
./install.sh --prebuilt
```

#### Cache directory locations

The installer chooses the cache directory in this order:
- `$FIREDBG_CACHE_DIR` (if set)
- `$XDG_CACHE_HOME/firedbg` (if set)
- macOS fallback: `~/Library/Caches/firedbg`
- other fallback: `~/.cache/firedbg`

### Verify installation

A debugger self-test is run automatically by the installer. If successful, expect to see:

```shell
info: completed FireDBG self tests
```

If the self-test fails, consult `Troubleshooting.md` and run:

```shell
firedbg doctor
```

### Prebuilt binaries (optional)

If you prefer prebuilt binaries, you can opt in with:

```shell
./install.sh --prebuilt
```

Or manually download from upstream releases:

1. Go to the upstream releases page
2. Download a compatible `*.tar.gz`
3. Extract and copy into `~/.cargo/bin`

(See upstream project docs for OS/arch naming.)
