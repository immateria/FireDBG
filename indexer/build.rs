use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-env-changed=LLDB_REL_DIR");
    println!("cargo:rerun-if-env-changed=FIREDBG_LLDB_DIR");
    println!("cargo:rerun-if-env-changed=FIREDBG_HOME");
    println!("cargo:rerun-if-env-changed=CARGO_HOME");
    println!("cargo:rerun-if-env-changed=HOME");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "macos" {
        return;
    }

    let lldb_dir = resolve_lldb_dir();
    let runtime_lib_dir = if Path::new(&lldb_dir).is_absolute() {
        format!("{lldb_dir}/lib")
    } else {
        // Preserve the historical "../lldb" behavior for repo-relative dirs.
        format!("../{lldb_dir}/lib")
    };

    // `liblldb.dylib` uses an `@rpath/...` install name. Cargo does not propagate runtime
    // environment variables from dependencies to dependents, so crates that indirectly link
    // against LLDB (via firedbg-rust-debugger) need to set a fallback search path themselves
    // for `cargo test --workspace` to work.
    println!("cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH={runtime_lib_dir}");
}

fn resolve_lldb_dir() -> String {
    // Explicit override (legacy)
    if let Ok(v) = env::var("LLDB_REL_DIR") {
        if !v.is_empty() {
            return v;
        }
    }

    // Explicit override (preferred)
    if let Ok(v) = env::var("FIREDBG_LLDB_DIR") {
        if !v.is_empty() {
            return v;
        }
    }

    // Try installed-mode `firedbg-lib`.
    // Prefer FIREDBG_HOME if set, else CARGO_HOME, else ~/.cargo/bin.
    let cargo_bin = if let Ok(home) = env::var("FIREDBG_HOME") {
        home
    } else if let Ok(ch) = env::var("CARGO_HOME") {
        format!("{ch}/bin")
    } else if let Ok(home) = env::var("HOME") {
        format!("{home}/.cargo/bin")
    } else {
        String::new()
    };

    if !cargo_bin.is_empty() {
        let candidate = format!("{cargo_bin}/firedbg-lib");
        if Path::new(&candidate).join("lib").is_dir() {
            return candidate;
        }
    }

    // Fallback to the historical repo-local folder.
    "lldb".to_owned()
}
