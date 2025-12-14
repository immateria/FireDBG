use std::{env, fs, path::Path};

pub type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let weak_linkage = match env::var("CARGO_FEATURE_WEAK_LINKAGE") {
        Ok(_) => true,
        Err(_) => false,
    };

    // Rebuild if any source files change
    rerun_if_changed_in(Path::new("src"))?;

    let mut build_config = cpp_build::Config::new();

    if weak_linkage {
        build_config.cpp_set_stdlib(None);
    } else {
        // This branch is used when building test runners
        set_rustc_link_search();
        set_dylib_search_path();
        if target_os == "windows" {
            println!("cargo:rustc-link-lib=dylib=liblldb");
        } else {
            // gcc does not recognize the "-stdlib=libc++" flag used by clang. Explicitly
            // request libc++ only on platforms where clang is expected (e.g. macOS) and let
            // Linux use the toolchain default to remain compatible with newer Rust images
            // that default to gcc. This keeps the build working without requiring a clang
            // toolchain to be installed.
            if target_os != "linux" {
                build_config.cpp_set_stdlib(Some("c++"));
            }

            let liblldb = detect_liblldb_library().unwrap_or_else(|| "lldb".to_string());
            println!("cargo:rustc-link-lib=dylib={}", liblldb);

            if target_os == "linux" {
                // Require all symbols to be defined in test runners
                println!("cargo:rustc-link-arg=--no-undefined");
            }
        }
    }

    // Generate C++ bindings
    build_config.include("include");
    build_config.build("src/lib.rs");

    Ok(())
}

fn rerun_if_changed_in(dir: &Path) -> Result<(), Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        } else {
            rerun_if_changed_in(&entry.path())?;
        }
    }
    Ok(())
}

fn set_rustc_link_search() {
    if let Ok(value) = env::var("CODELLDB_LIB_PATH") {
        for path in value.split_terminator(';') {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }
}

fn detect_liblldb_library() -> Option<String> {
    // Prefer user-provided locations first
    let mut search_dirs: Vec<String> = env::var("CODELLDB_LIB_PATH")
        .unwrap_or_default()
        .split_terminator(';')
        .map(|s| s.to_string())
        .collect();

    search_dirs.extend([
        "/usr/lib/llvm-20/lib".to_string(),
        "/usr/lib/llvm-19/lib".to_string(),
        "/usr/lib/llvm-18/lib".to_string(),
        "/usr/lib/llvm-17/lib".to_string(),
        "/usr/lib/llvm-16/lib".to_string(),
        "/usr/lib/x86_64-linux-gnu".to_string(),
        "/usr/lib".to_string(),
    ]);

    for dir in search_dirs {
        let path = Path::new(&dir);
        if !path.exists() {
            continue;
        }

        let versioned = ["20", "19", "18", "17", "16", "15", "14"];
        let candidates = std::iter::once("liblldb.so".to_string()).chain(
            versioned
                .iter()
                .flat_map(|v| [format!("liblldb-{v}.so"), format!("liblldb-{v}.so.1")]),
        );

        for candidate in candidates {
            if path.join(&candidate).exists() {
                println!("cargo:rustc-link-search=native={}", path.display());

                if let Some(version) = candidate
                    .strip_prefix("liblldb-")
                    .and_then(|rest| rest.strip_prefix("lldb-").or(Some(rest)))
                {
                    if let Some(version) = version.split_once('.') {
                        return Some(format!("lldb-{}", version.0.trim_start_matches('v')));
                    }
                }

                if let Some(version) = candidate
                    .strip_prefix("liblldb-")
                    .and_then(|rest| rest.strip_suffix(".so"))
                {
                    return Some(version.to_string());
                }

                return Some("lldb".to_string());
            }
        }
    }

    None
}

fn set_dylib_search_path() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if let Ok(value) = env::var("CODELLDB_LIB_PATH") {
        if target_os == "linux" {
            let prev = env::var("LD_LIBRARY_PATH").unwrap_or_default();
            println!(
                "cargo:rustc-env=LD_LIBRARY_PATH={}:{}",
                prev,
                value.replace(";", ":")
            );
        } else if target_os == "macos" {
            println!(
                "cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH={}",
                value.replace(";", ":")
            );
        } else if target_os == "windows" {
            println!(
                "cargo:rustc-env=PATH={};{}",
                env::var("PATH").unwrap(),
                value
            );
        }
    }
}
