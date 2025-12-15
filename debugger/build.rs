// Statically import `rustc_version` method into scope
include!("src/version.rs");

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // FireDBG needs CodeLLDB's LLDB runtime assets (liblldb + friends) at link time.
    // Historically this was provided as a repo-local `./lldb/` folder.
    //
    // We now prefer a user-local cache (installed via install.sh) and allow overriding the
    // location via env vars.
    let lldb_dir = resolve_lldb_dir();

    // At compile/link time, we need the LLDB lib dir.
    println!(r"cargo:rustc-link-search={lldb_dir}/lib");

    // At runtime, set a best-effort fallback search path.
    // If `lldb_dir` is absolute, use it directly.
    // If it's relative, preserve the historical `../` behavior.
    let runtime_lib_dir = if std::path::Path::new(&lldb_dir).is_absolute() {
        format!("{lldb_dir}/lib")
    } else {
        format!("../{lldb_dir}/lib")
    };

    if target_os == "linux" {
        println!(r"cargo:rustc-env=LD_LIBRARY_PATH={runtime_lib_dir}");
    } else if target_os == "macos" {
        println!(r"cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH={runtime_lib_dir}");
    }

    println!(r"cargo:rustc-env=RUSTC_VERSION={}", rustc_version());
}

fn resolve_lldb_dir() -> String {
    // Explicit override (legacy)
    if let Ok(v) = std::env::var("LLDB_REL_DIR") {
        if !v.is_empty() {
            return v;
        }
    }

    // Explicit override (preferred)
    if let Ok(v) = std::env::var("FIREDBG_LLDB_DIR") {
        if !v.is_empty() {
            return v;
        }
    }

    // Try installed-mode `firedbg-lib`.
    // Prefer FIREDBG_HOME if set, else CARGO_HOME, else ~/.cargo/bin.
    let cargo_bin = if let Ok(home) = std::env::var("FIREDBG_HOME") {
        home
    } else if let Ok(ch) = std::env::var("CARGO_HOME") {
        format!("{ch}/bin")
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.cargo/bin")
    } else {
        String::new()
    };

    if !cargo_bin.is_empty() {
        let candidate = format!("{cargo_bin}/firedbg-lib");
        if std::path::Path::new(&candidate).join("lib").is_dir() {
            return candidate;
        }
    }

    // Fallback to the historical repo-local folder.
    "lldb".to_owned()
}
