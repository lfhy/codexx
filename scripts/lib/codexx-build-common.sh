#!/usr/bin/env bash

codexx_repo_root() {
  cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd
}

codexx_cargo_root() {
  printf '%s/codex-rs\n' "$(codexx_repo_root)"
}

codexx_build_root() {
  printf '%s/build\n' "$(codexx_repo_root)"
}

codexx_toolchain() {
  sed -n 's/^channel = "\(.*\)"/\1/p' "$(codexx_cargo_root)/rust-toolchain.toml" | head -n 1
}

codexx_workspace_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' "$(codexx_cargo_root)/Cargo.toml" | head -n 1
}

codexx_git_commit() {
  git -C "$(codexx_repo_root)" rev-parse HEAD
}

codexx_git_short_commit() {
  git -C "$(codexx_repo_root)" rev-parse --short=7 HEAD
}

codexx_cargo() {
  local toolchain
  toolchain="$(codexx_toolchain)"
  rustup run "$toolchain" cargo "$@"
}

codexx_log() {
  printf '==> %s\n' "$*" >&2
}

codexx_warn() {
  printf 'warning: %s\n' "$*" >&2
}

codexx_die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

codexx_have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

codexx_sudo_prefix() {
  if [[ ${EUID:-$(id -u)} -eq 0 ]]; then
    return 0
  fi

  if codexx_have_cmd sudo; then
    printf 'sudo '
    return 0
  fi

  codexx_die "missing root privileges and sudo; install prerequisites manually"
}

codexx_detect_pkg_manager() {
  if codexx_have_cmd brew; then
    printf 'brew\n'
  elif codexx_have_cmd apt-get; then
    printf 'apt-get\n'
  elif codexx_have_cmd dnf; then
    printf 'dnf\n'
  elif codexx_have_cmd yum; then
    printf 'yum\n'
  elif codexx_have_cmd pacman; then
    printf 'pacman\n'
  elif codexx_have_cmd zypper; then
    printf 'zypper\n'
  else
    printf '\n'
  fi
}

codexx_append_git_config_env() {
  local key="$1"
  local value="$2"
  local index="${GIT_CONFIG_COUNT:-0}"
  local key_var="GIT_CONFIG_KEY_${index}"
  local value_var="GIT_CONFIG_VALUE_${index}"

  printf -v "$key_var" '%s' "$key"
  printf -v "$value_var" '%s' "$value"

  export GIT_CONFIG_COUNT="$((index + 1))"
  export "$key_var"
  export "$value_var"
}

codexx_probe_url() {
  curl --silent --fail --location --head --max-time 8 "$1" >/dev/null 2>&1 \
    || curl --silent --fail --location --max-time 8 "$1" >/dev/null 2>&1
}

codexx_git_mirror_probe_url() {
  case "$1" in
    https://gh-proxy.com/*)
      printf 'https://gh-proxy.com/\n'
      ;;
    https://ghfast.top/*)
      printf 'https://ghfast.top/\n'
      ;;
    https://gitclone.com/*)
      printf 'https://gitclone.com/\n'
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

codexx_select_git_mirror_prefix() {
  if [[ -n "${CODEXX_GIT_MIRROR_PREFIX:-}" ]]; then
    return 0
  fi

  export CODEXX_GIT_MIRROR_CANDIDATES="${CODEXX_GIT_MIRROR_CANDIDATES:-https://gh-proxy.com/https://github.com/ https://ghfast.top/https://github.com/ https://gitclone.com/github.com/}"

  local candidate
  local probe_url
  for candidate in $CODEXX_GIT_MIRROR_CANDIDATES; do
    probe_url="$(codexx_git_mirror_probe_url "$candidate")"
    if codexx_probe_url "$probe_url"; then
      export CODEXX_GIT_MIRROR_PREFIX="$candidate"
      return 0
    fi
  done

  codexx_warn "no GitHub mirror candidate responded; falling back to direct github.com"
  export CODEXX_GIT_MIRROR_PREFIX="https://github.com/"
}

codexx_export_mirror_env() {
  export RUSTUP_DIST_SERVER="${RUSTUP_DIST_SERVER:-https://rsproxy.cn}"
  export RUSTUP_UPDATE_ROOT="${RUSTUP_UPDATE_ROOT:-https://rsproxy.cn/rustup}"
  export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
  export CARGO_REGISTRIES_CRATES_IO_PROTOCOL="${CARGO_REGISTRIES_CRATES_IO_PROTOCOL:-sparse}"
  export CARGO_REGISTRIES_CRATES_IO_INDEX="${CARGO_REGISTRIES_CRATES_IO_INDEX:-sparse+https://rsproxy.cn/index/}"
  export PATH="${HOME}/.cargo/bin:${PATH}"
  codexx_select_git_mirror_prefix
  codexx_append_git_config_env "url.${CODEXX_GIT_MIRROR_PREFIX}.insteadof" "https://github.com/"
}

codexx_install_system_prereqs() {
  local pkg_manager
  pkg_manager="$(codexx_detect_pkg_manager)"

  if [[ -z "$pkg_manager" ]]; then
    codexx_warn "no supported package manager found; skipping system package installation"
    return 0
  fi

  case "$pkg_manager" in
    brew)
      if ! xcode-select -p >/dev/null 2>&1; then
        xcode-select --install || true
        codexx_die "Xcode Command Line Tools installation has been started; rerun after it finishes"
      fi

      codexx_log "Installing macOS build prerequisites with Homebrew"
      brew install git ripgrep cmake pkgconf llvm
      ;;
    apt-get)
      codexx_log "Installing Linux build prerequisites with apt-get"
      local sudo_prefix
      sudo_prefix="$(codexx_sudo_prefix)"
      ${sudo_prefix}apt-get update
      DEBIAN_FRONTEND=noninteractive ${sudo_prefix}apt-get install -y \
        build-essential \
        clang \
        cmake \
        curl \
        git \
        libcap-dev \
        libsqlite3-dev \
        libssl-dev \
        pkg-config \
        ripgrep
      ;;
    dnf)
      codexx_log "Installing Linux build prerequisites with dnf"
      local sudo_prefix
      sudo_prefix="$(codexx_sudo_prefix)"
      ${sudo_prefix}dnf install -y \
        clang \
        cmake \
        curl \
        gcc \
        gcc-c++ \
        git \
        libcap-devel \
        openssl-devel \
        pkgconf-pkg-config \
        ripgrep \
        sqlite-devel
      ;;
    yum)
      codexx_log "Installing Linux build prerequisites with yum"
      local sudo_prefix
      sudo_prefix="$(codexx_sudo_prefix)"
      ${sudo_prefix}yum install -y \
        clang \
        cmake \
        curl \
        gcc \
        gcc-c++ \
        git \
        libcap-devel \
        openssl-devel \
        pkgconfig \
        ripgrep \
        sqlite-devel
      ;;
    pacman)
      codexx_log "Installing Linux build prerequisites with pacman"
      local sudo_prefix
      sudo_prefix="$(codexx_sudo_prefix)"
      ${sudo_prefix}pacman -Sy --noconfirm \
        base-devel \
        clang \
        cmake \
        curl \
        git \
        libcap \
        openssl \
        pkgconf \
        ripgrep \
        sqlite
      ;;
    zypper)
      codexx_log "Installing Linux build prerequisites with zypper"
      local sudo_prefix
      sudo_prefix="$(codexx_sudo_prefix)"
      ${sudo_prefix}zypper --non-interactive install \
        clang \
        cmake \
        curl \
        gcc \
        gcc-c++ \
        git \
        libcap-devel \
        libopenssl-devel \
        pkg-config \
        ripgrep \
        sqlite3-devel
      ;;
  esac
}

codexx_ensure_curl() {
  if codexx_have_cmd curl; then
    return 0
  fi

  codexx_install_system_prereqs

  if ! codexx_have_cmd curl; then
    codexx_die "curl is required to install rustup"
  fi
}

codexx_ensure_rustup() {
  if codexx_have_cmd rustup && codexx_have_cmd cargo && codexx_have_cmd rustc; then
    return 0
  fi

  codexx_ensure_curl
  codexx_log "Installing rustup from RsProxy"
  curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh -s -- -y --profile minimal
  export PATH="${HOME}/.cargo/bin:${PATH}"

  if ! codexx_have_cmd rustup || ! codexx_have_cmd cargo || ! codexx_have_cmd rustc; then
    codexx_die "rustup installation completed but cargo/rustc is still not available in PATH"
  fi
}

codexx_ensure_rust_toolchain() {
  local toolchain
  toolchain="$(codexx_toolchain)"

  codexx_log "Ensuring Rust toolchain ${toolchain}"
  rustup toolchain install "$toolchain" --profile minimal
  rustup component add clippy rustfmt rust-src --toolchain "$toolchain"
}

codexx_ensure_cargo_binary() {
  local binary_name="$1"
  local crate_name="$2"

  if codexx_have_cmd "$binary_name"; then
    return 0
  fi

  codexx_log "Installing cargo package ${crate_name}"
  codexx_cargo install --locked "$crate_name"
}

codexx_print_environment_report() {
  local pkg_manager
  pkg_manager="$(codexx_detect_pkg_manager)"

  codexx_log "Build environment report"
  printf '  OS: %s\n' "$(uname -srvmo 2>/dev/null || uname -a)" >&2
  printf '  Repo: %s\n' "$(codexx_repo_root)" >&2
  printf '  Cargo workspace: %s\n' "$(codexx_cargo_root)" >&2
  printf '  Fork build root: %s\n' "$(codexx_build_root)" >&2
  printf '  Package manager: %s\n' "${pkg_manager:-not-found}" >&2
  printf '  RUSTUP_DIST_SERVER: %s\n' "$RUSTUP_DIST_SERVER" >&2
  printf '  RUSTUP_UPDATE_ROOT: %s\n' "$RUSTUP_UPDATE_ROOT" >&2
  printf '  CARGO_REGISTRIES_CRATES_IO_INDEX: %s\n' "$CARGO_REGISTRIES_CRATES_IO_INDEX" >&2
  printf '  CODEXX_GIT_MIRROR_CANDIDATES: %s\n' "$CODEXX_GIT_MIRROR_CANDIDATES" >&2
  printf '  CODEXX_GIT_MIRROR_PREFIX: %s\n' "$CODEXX_GIT_MIRROR_PREFIX" >&2
  printf '  rustup: %s\n' "$(rustup --version | head -n 1)" >&2
  printf '  active-toolchain: %s\n' "$(rustup show active-toolchain 2>/dev/null || printf 'unknown')" >&2
  printf '  cargo: %s\n' "$(cargo --version)" >&2
  printf '  rustc: %s\n' "$(rustc --version)" >&2
  printf '  just: %s\n' "$(just --version 2>/dev/null || printf 'not-installed')" >&2
  printf '  cargo-insta: %s\n' "$(cargo-insta --version 2>/dev/null || printf 'not-installed')" >&2
  printf '  cargo-nextest: %s\n' "$(cargo-nextest --version 2>/dev/null || printf 'not-installed')" >&2
  printf '  rg: %s\n' "$(rg --version 2>/dev/null | head -n 1 || printf 'not-installed')" >&2
  printf '  cmake: %s\n' "$(cmake --version 2>/dev/null | head -n 1 || printf 'not-installed')" >&2
  printf '  clang: %s\n' "$(clang --version 2>/dev/null | head -n 1 || printf 'not-installed')" >&2
  printf '  git: %s\n' "$(git --version 2>/dev/null || printf 'not-installed')" >&2
}

codexx_prepare_build_env() {
  codexx_export_mirror_env
  codexx_install_system_prereqs
  codexx_ensure_rustup
  codexx_ensure_rust_toolchain
  codexx_ensure_cargo_binary just just
  codexx_ensure_cargo_binary cargo-insta cargo-insta
  codexx_ensure_cargo_binary cargo-nextest cargo-nextest
  codexx_print_environment_report
}
