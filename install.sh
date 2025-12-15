#!/usr/bin/env zsh

# FireDBG installer (zsh)
# - Uses zsh semantics & conventions
# - Smarter rustc handling: for newer rustc versions (e.g. 1.91) it
#   falls back to the latest known compatible FireDBG release
#   instead of hard-failing.

set -u

# Shared return slot for helper functions
typeset -g RETVAL=""

function resolve_cache_dir
{   emulate -L zsh
    setopt localoptions

    typeset cache_dir xdg_cache os

    set +u
    cache_dir=${FIREDBG_CACHE_DIR:-}
    xdg_cache=${XDG_CACHE_HOME:-}
    set -u

    if [[ -n "$cache_dir" ]]; then
        RETVAL="$cache_dir"
        return 0
    fi

    if [[ -n "$xdg_cache" ]]; then
        RETVAL="${xdg_cache%/}/firedbg"
        return 0
    fi

    os=$(uname -s 2>/dev/null || true)
    if [[ "$os" == Darwin ]]; then
        RETVAL="${HOME}/Library/Caches/firedbg"
    else
        RETVAL="${HOME}/.cache/firedbg"
    fi
}

function clean_cache
{   emulate -L zsh
    setopt localoptions

    resolve_cache_dir || return 1
    typeset cache_dir="$RETVAL"

    if [[ -d "$cache_dir" ]]; then
        say "info: removing cache directory: ${cache_dir}"
        ensure rm -rf -- "$cache_dir"
    else
        say "info: cache directory does not exist: ${cache_dir}"
    fi
}

function get_firedbg_version
{   emulate -L zsh
    setopt localoptions

    typeset rustc_output version_str major minor short
    typeset -a parts
    typeset -A firedbg_versions

    rustc_output=$(rustc --version 2>/dev/null) || err "failed to run 'rustc --version'"

    # Expect: rustc 1.91.0 (....)
    version_str=${rustc_output#rustc }
    version_str=${version_str%% *}

    parts=(${(s:.:)version_str})
    if (( ${#parts} < 2 )); then
        err "unrecognized rustc version string: ${rustc_output}"
    fi

    major=${parts[1]}
    minor=${parts[2]}
    short="${major}.${minor}"

    # Known mappings (keep this in sync with FireDBG releases)
    firedbg_versions=(
        "1.74" "1.74.2"
        "1.75" "1.75.1"
        "1.76" "1.76.0"
        "1.77" "1.77.1"
        "1.78" "1.78.0"
        "1.79" "1.79.0"
        "1.80" "1.80.0"
        "1.81" "1.81.0"
    )

    if [[ -n ${firedbg_versions[$short]:-} ]]; then
        RETVAL="${firedbg_versions[$short]}"
        return 0
    fi

    # Fallback: choose the latest known <= current minor
    typeset best_key="" best_minor=-1 this_minor
    for k in ${(k)firedbg_versions}; do
        if [[ ${k%%.*} != "$major" ]]; then
            continue
        fi
        this_minor=${k#*.}
        if (( this_minor <= minor && this_minor > best_minor )); then
            best_minor=$this_minor
            best_key=$k
        fi
    done

    if [[ -z "$best_key" ]]; then
        err "no precompiled binaries available for ${rustc_output}"
    fi

    say "warning: no precompiled FireDBG binaries for rustc ${short}.*, using ${best_key} (FireDBG ${firedbg_versions[$best_key]}) instead"
    RETVAL="${firedbg_versions[$best_key]}"
}

function main
{   emulate -L zsh
    setopt localoptions

    typeset mode="source"
    typeset do_clean_cache=0

    while (( $# > 0 )); do
        case "$1" in
            --source)
                mode="source"
                ;;
            --prebuilt)
                mode="prebuilt"
                ;;
            --clean-cache)
                do_clean_cache=1
                ;;
            -h|--help)
                print -r -- "usage: install.sh [--source|--prebuilt] [--clean-cache]"
                return 0
                ;;
            *)
                err "unknown argument: $1"
                ;;
        esac
        shift
    done

    if (( do_clean_cache )); then
        clean_cache || return 1
    fi

    if [[ "$mode" == prebuilt ]]; then
        install_prebuilt
    else
        install_from_source
    fi
}

function run_self_test
{   emulate -L zsh
    setopt localoptions

    typeset _cargo_home="$1"
    typeset _self_test="${_cargo_home}/bin/firedbg-lib/debugger-self-test"

    if [ ! -d "${_self_test}" ]; then
        printf '%s\n' 'info: skipping FireDBG self tests (debugger-self-test assets not found)' 1>&2
        return 0
    fi

    printf '%s\n' 'info: performing FireDBG self tests' 1>&2

    cd "${_self_test}"
    "${_cargo_home}/bin/firedbg" run debugger_self_test --output "${_self_test}/output.firedbg.ss"
    cd - > /dev/null

    if [ $? != 0 ]; then
        say "fail to run FireDBG debugger"
        exit 1
    fi

    "${_cargo_home}/bin/firedbg-indexer" --input "${_self_test}/output.firedbg.ss" \
        validate --json "${_self_test}/expected_data.json"

    if [ $? != 0 ]; then
        say "fail to validate FireDBG debugger result"
        exit 1
    fi

    printf '%s\n' 'info: completed FireDBG self tests' 1>&2
}

function install_prebuilt
{   emulate -L zsh
    setopt localoptions

    downloader --check
    need_cmd uname
    need_cmd mktemp
    need_cmd mkdir
    need_cmd rm
    need_cmd tar
    need_cmd which

    get_architecture || return 1
    typeset _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    which rustup > /dev/null 2>&1
    need_ok "failed to find Rust installation, is rustup installed?"

    get_firedbg_version || return 1
    typeset _firedbg_version="$RETVAL"
    assert_nz "$_firedbg_version" "firedbg version"

    typeset _url="https://github.com/SeaQL/FireDBG.for.Rust/releases/download/$_firedbg_version/$_arch.tar.gz"
    typeset _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t FireDBG)"
    typeset _file="${_dir}/${_arch}.tar.gz"

    set +u
    typeset _cargo_home="$CARGO_HOME"
    if [ -z "$_cargo_home" ]; then
        _cargo_home="$HOME/.cargo";
    fi
    typeset _cargo_bin="$_cargo_home/bin"
    ensure mkdir -p "$_cargo_bin"
    set -u

    printf '%s `%s`\n' 'info: downloading FireDBG from' "$_url" 1>&2

    ensure mkdir -p "$_dir"
    downloader "$_url" "$_file"
    if [ $? != 0 ]; then
        say "failed to download $_url"
        say "this may be a standard network error, but it may also indicate"
        say "that FireDBG's release process is not working. When in doubt"
        say "please feel free to open an issue!"
        exit 1
    fi
    ensure tar xf "$_file" --strip-components 1 -C "$_dir"

    printf '%s `%s`\n' 'info: installing FireDBG binaries to' "$_cargo_bin" 1>&2

    ignore rm -rf "$_cargo_bin/firedbg*"
    ignore rm -rf "$_cargo_bin/firedbg-lib"

    ensure mv "$_dir/firedbg-lib"       "$_cargo_bin/firedbg-lib"
    ensure mv "$_dir/firedbg"           "$_cargo_bin/firedbg"
    ensure mv "$_dir/firedbg-indexer"   "$_cargo_bin/firedbg-indexer"
    ensure mv "$_dir/firedbg-debugger"  "$_cargo_bin/firedbg-debugger"

    run_self_test "$_cargo_home"
}

function install_from_source
{   emulate -L zsh
    setopt localoptions

    say "info: installing FireDBG from source"

    downloader --check
    need_cmd cargo
    need_cmd uname
    need_cmd mktemp
    need_cmd mkdir
    need_cmd rm
    need_cmd which
    need_cmd unzip

    set +u
    typeset _cargo_home="$CARGO_HOME"
    if [ -z "$_cargo_home" ]; then
        _cargo_home="$HOME/.cargo";
    fi
    typeset _cargo_bin="$_cargo_home/bin"
    ensure mkdir -p "$_cargo_bin"
    set -u

    if [[ ! -f "Cargo.toml" || ! -f "command/Cargo.toml" ]]; then
        err "source install requested but FireDBG source tree not found; clone the repository and run install.sh from its root"
    fi

    resolve_cache_dir || return 1
    typeset cache_dir="$RETVAL"
    ensure mkdir -p "$cache_dir"

    typeset codelldb_version="v1.10.0"
    typeset vsix_arch vsix_name cache_root cache_vsix cache_lldb tmpdir

    case "$(uname -m)" in
        x86_64|x86-64|x64|amd64)
            vsix_arch="x86_64-darwin"
            ;;
        arm64|aarch64)
            vsix_arch="aarch64-darwin"
            ;;
        *)
            err "unsupported CPU architecture for source install: $(uname -m)"
            ;;
    esac

    vsix_name="codelldb-${vsix_arch}.vsix"
    cache_root="${cache_dir}/codelldb/${codelldb_version}/${vsix_arch}"
    cache_vsix="${cache_root}/${vsix_name}"
    cache_lldb="${cache_root}/lldb"

    if [[ ! -d "${cache_lldb}/lib" ]]; then
        say "info: preparing codelldb bundle (${vsix_arch}) in cache"
        ensure mkdir -p "$cache_root"

        if [[ ! -f "$cache_vsix" ]]; then
            downloader "https://github.com/vadimcn/codelldb/releases/download/${codelldb_version}/${vsix_name}" "$cache_vsix"
        fi

        tmpdir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t FireDBG)"
        ensure unzip -q "$cache_vsix" -d "$tmpdir"

        if [[ ! -d "$tmpdir/extension/lldb" ]]; then
            err "unexpected codelldb bundle layout (missing extension/lldb)"
        fi

        ignore rm -rf -- "$cache_lldb"
        ensure mv "$tmpdir/extension/lldb" "$cache_lldb"
        ignore rm -rf -- "$tmpdir"
    fi

    say "info: building FireDBG from source (command, debugger, indexer)"
    ensure cargo build --manifest-path "command/Cargo.toml"
    ensure cargo build --manifest-path "debugger/Cargo.toml"
    ensure cargo build --manifest-path "indexer/Cargo.toml"

    say "info: installing FireDBG binaries from target/debug to '${_cargo_bin}'"
    ignore rm -f "${_cargo_bin}/firedbg" "${_cargo_bin}/firedbg-indexer" "${_cargo_bin}/firedbg-debugger"
    ignore rm -rf "${_cargo_bin}/firedbg-lib"

    ensure ln -sf "$PWD/target/debug/firedbg"           "${_cargo_bin}/firedbg"
    ensure ln -sf "$PWD/target/debug/firedbg-indexer"  "${_cargo_bin}/firedbg-indexer"
    ensure ln -sf "$PWD/target/debug/firedbg-debugger" "${_cargo_bin}/firedbg-debugger"
    ensure ln -sfn "$cache_lldb"                        "${_cargo_bin}/firedbg-lib"

    run_self_test "$_cargo_home"
}

function get_architecture
{   emulate -L zsh
    setopt localoptions
    typeset _ostype="$(uname -s)"
    typeset _cputype="$(uname -m)"

    set +u
    if [ -n "$TARGETOS" ]; then
        _ostype="$TARGETOS"
    fi

    if [ -n "$TARGETARCH" ]; then
        _cputype="$TARGETARCH"
    fi
    set -u

    if [ "$_ostype" = Darwin ] && [ "$_cputype" = i386 ]; then
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            _cputype=x86_64
        fi
    fi

    case "$_ostype" in
        Linux | linux)
            typeset _os_id="$(awk -F= '$1=="ID" { print $2 ;}' /etc/os-release | tr -d '"')"
            typeset _os_version_id="$(awk -F= '$1=="VERSION_ID" { print $2 ;}' /etc/os-release | tr -d '"')"
            _ostype="$_os_id$_os_version_id"
            case "$_ostype" in
                pop*)
                    _ostype="ubuntu$_os_version_id"
                    ;;
            esac
            typeset _os_id_like="$(awk -F= '$1=="ID_LIKE" { print $2 ;}' /etc/os-release | tr -d '"')"
            case "$_os_id" in
                linuxmint*)
                    case "$_os_id_like" in
                        ubuntu*)
                            case "$_os_version_id" in
                                24*) # Ubuntu Noble
                                    _ostype="ubuntu24.04"
                                    ;;
                                21*) # Ubuntu Jammy
                                    _ostype="ubuntu22.04"
                                    ;;
                                20*) # Ubuntu Focal
                                    _ostype="ubuntu20.04"
                                    ;;
                            esac
                            ;;
                        debian*) # Debian Bookworm
                            _ostype="debian12"
                            ;;
                    esac
            esac
            case "$_ostype" in
                ubuntu24*)
                    check_apt_install libc++abi1-18
                    _ostype="ubuntu22.04"
                    ;;
                ubuntu22*)
                    check_apt_install libc++abi1-15
                    ;;
                ubuntu20*)
                    check_apt_install libc++abi1-10
                    ;;
                debian12*)
                    check_apt_install libc++abi1-14
                    ;;
                debian10*)
                    check_apt_install libc++abi1-7
                    ;;
                debian*)
                    check_apt_install libc++abi1-16
                    _ostype="debian12"
                    ;;
                fedora39* | fedora40* | fedora41*)
                    check_dnf_install libcxxabi
                    _ostype="fedora39"
                    ;;
                centos9*)
                    check_yum_install_rpm libcxxabi https://kojipkgs.fedoraproject.org//packages/libcxx/17.0.4/1.fc39/x86_64/libcxxabi-17.0.4-1.fc39.x86_64.rpm
                    ;;
                arch* | manjaro* | endeavouros* | garuda*)
                    check_pacman_install libc++abi
                    _ostype="ubuntu20.04"
                    ;;
                *)
                    err "no precompiled binaries available for OS: $_ostype"
                    ;;
            esac
            ;;
        Darwin)
            _ostype=darwin
            ;;
        MINGW* | MSYS* | CYGWIN*)
            err "please run this installation script inside Windows Subsystem for Linux (WSL 2)"
            ;;
        *)
            err "no precompiled binaries available for OS: $_ostype"
            ;;
    esac

    case "$_cputype" in
        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;
        arm64 | aarch64)
            _cputype=aarch64
            ;;
        *)
            err "no precompiled binaries available for CPU architecture: $_cputype"
    esac

    if [ "$_cputype" = aarch64 ] && [ "$_ostype" = apple-darwin ]; then
        _cputype="x86_64"
    fi

    typeset _arch="$_cputype-$_ostype"

    RETVAL="$_arch"
}

function say
{   emulate -L zsh
    setopt localoptions
    echo "FireDBG: $1"
}

function err
{   emulate -L zsh
    setopt localoptions
    say "$1" >&2
    exit 1
}

function need_cmd
{   emulate -L zsh
    setopt localoptions
    if ! check_cmd "$1"
    then err "need '$1' (command not found)"
    fi
}

function check_cmd
{   emulate -L zsh
    setopt localoptions
    command -v "$1" > /dev/null 2>&1
    return $?
}

function need_ok
{   emulate -L zsh
    setopt localoptions
    if [ $? != 0 ]; then err "$1"; fi
}

function assert_nz
{   emulate -L zsh
    setopt localoptions
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
function ensure
{   emulate -L zsh
    setopt localoptions
    "$@"
    need_ok "command failed: $*"
}

function ignore
{   emulate -L zsh
    setopt localoptions
    "$@"
}

# This wraps curl or wget. Try curl first, if not installed,
# use wget instead.
function downloader
{   emulate -L zsh
    setopt localoptions
    if check_cmd curl
    then _dld=curl
    elif check_cmd wget
    then _dld=wget
    else _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]
    then need_cmd "$_dld"
    elif [ "$_dld" = curl ]
    then curl -sSfL "$1" -o "$2"
    elif [ "$_dld" = wget ]
    then wget "$1" -O "$2"
    else err "Unknown downloader"   # should not reach here
    fi
}

function check_apt_install
{   emulate -L zsh
    setopt localoptions
    if [ "$(dpkg-query -l | grep $1 | wc -l)" = 0 ]; then
        run_sudo apt install -y $1
    fi
}

function check_dnf_install
{   emulate -L zsh
    setopt localoptions
    if [ "$(dnf list installed | grep $1 | wc -l)" = 0 ]; then
        run_sudo dnf install -y $1
    fi
}

function check_yum_install_rpm
{   emulate -L zsh
    setopt localoptions
    if [ "$(dnf list installed | grep $1 | wc -l)" = 0 ]; then
        run_sudo yum install -y $2
    fi
}

function check_pacman_install
{   emulate -L zsh
    setopt localoptions
    if [ "$(pacman -Q | grep $1 | wc -l)" = 0 ]; then
        run_sudo pacman -S --noconfirm $1
    fi
}

function run_sudo
{   emulate -L zsh
    setopt localoptions
    if ! check_cmd "sudo"
    then $@
    else sudo $@
    fi
}

main "$@" || exit 1
