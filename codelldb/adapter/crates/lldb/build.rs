use std::{env, fs, path::Path};

pub type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let weak_linkage = env::var("CARGO_FEATURE_WEAK_LINKAGE").is_ok();

    // Rebuild if any source files change
    rerun_if_changed_in(Path::new("src"))?;

    let mut build_config = cpp_build::Config::new();

    if weak_linkage {
        build_config.cpp_set_stdlib(None);
    } else {
        // This branch is used when building test runners.
        //
        // On macOS, `cargo test --workspace` will also try to build and link the `lldb` crate's
        // test harness. When FireDBG is installed in "source" mode we have a known-good LLDB
        // runtime in the user cache (`~/.cargo/bin/firedbg-lib`). Auto-detect it here so tests
        // work out-of-the-box without requiring CODELLDB_LIB_PATH to be set.
        let lib_paths = resolve_liblldb_search_paths(&target_os);
        set_rustc_link_search(&lib_paths);
        set_rpath(&target_os, &lib_paths);
        set_dylib_search_path(&target_os, &lib_paths);

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

            let liblldb = detect_liblldb_library(&target_os, &lib_paths)
                .unwrap_or_else(|| "lldb".to_string());
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

fn resolve_liblldb_search_paths(target_os: &str) -> Vec<String> {
    // 1) CODELLDB_LIB_PATH is the upstream knob.
    if let Ok(value) = env::var("CODELLDB_LIB_PATH") {
        let dirs: Vec<String> = value
            .split_terminator(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !dirs.is_empty() {
            return dirs;
        }
    }

    // 2) FireDBG uses a stable "installed-mode" cache location.
    //    - FIREDBG_LLDB_DIR (preferred override)
    //    - ~/.cargo/bin/firedbg-lib (install.sh creates this as a symlink to the cache)
    let mut dirs = Vec::new();

    if let Ok(v) = env::var("FIREDBG_LLDB_DIR") {
        if !v.is_empty() {
            let p = Path::new(&v);
            if p.join("lib").is_dir() {
                dirs.push(p.join("lib").display().to_string());
            } else if p.is_dir() {
                dirs.push(p.display().to_string());
            }
        }
    }

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
        let candidate = Path::new(&cargo_bin).join("firedbg-lib");
        if candidate.join("lib").is_dir() {
            dirs.push(candidate.join("lib").display().to_string());
        }
    }

    // 3) Best-effort system fallbacks (kept minimal and only when relevant).
    if target_os == "macos" {
        dirs.extend([
            "/Library/Developer/CommandLineTools/usr/lib".to_string(),
            "/Applications/Xcode.app/Contents/Developer/usr/lib".to_string(),
        ]);
    } else if target_os == "linux" {
        dirs.extend([
            "/usr/lib/llvm-20/lib".to_string(),
            "/usr/lib/llvm-19/lib".to_string(),
            "/usr/lib/llvm-18/lib".to_string(),
            "/usr/lib/llvm-17/lib".to_string(),
            "/usr/lib/llvm-16/lib".to_string(),
            "/usr/lib/x86_64-linux-gnu".to_string(),
            "/usr/lib".to_string(),
        ]);
    }

    dirs
}

fn set_rustc_link_search(paths: &[String]) {
    for path in paths {
        println!("cargo:rustc-link-search=native={}", path);
    }
}

fn set_rpath(target_os: &str, lib_paths: &[String]) {
    if lib_paths.is_empty() {
        return;
    }

    // Ensure binaries/tests can locate `@rpath/liblldb.*` at runtime.
    // This is particularly important for `cargo test --workspace` where the execution environment
    // is not wrapped by our installer scripts.
    if target_os == "macos" || target_os == "linux" {
        for p in lib_paths {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", p);
        }
    }
}

fn detect_liblldb_library(target_os: &str, search_dirs: &[String]) -> Option<String> {
    for dir in search_dirs {
        let path = Path::new(dir);
        if !path.exists() {
            continue;
        }

        if target_os == "macos" {
            // CodeLLDB bundles `liblldb.dylib`.
            if path.join("liblldb.dylib").exists() {
                return Some("lldb".to_string());
            }
            continue;
        }

        if target_os != "linux" {
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

fn set_dylib_search_path(target_os: &str, lib_paths: &[String]) {
    if lib_paths.is_empty() {
        return;
    }

    let joined = lib_paths.join(":");

    if target_os == "linux" {
        let prev = env::var("LD_LIBRARY_PATH").unwrap_or_default();
        if prev.is_empty() {
            println!("cargo:rustc-env=LD_LIBRARY_PATH={}", joined);
        } else {
            println!("cargo:rustc-env=LD_LIBRARY_PATH={}:{}", prev, joined);
        }
    } else if target_os == "macos" {
        println!("cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH={}", joined);
    } else if target_os == "windows" {
        println!(
            "cargo:rustc-env=PATH={};{}",
            env::var("PATH").unwrap_or_default(),
            lib_paths.join(";")
        );
    }
}
