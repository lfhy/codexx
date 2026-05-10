#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/scripts/lib/codexx-build-common.sh"

skip_bootstrap="0"
run_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-bootstrap)
      skip_bootstrap="1"
      shift
      ;;
    --)
      shift
      run_args=("$@")
      break
      ;;
    *)
      run_args+=("$1")
      shift
      ;;
  esac
done

if [[ "$skip_bootstrap" == "0" ]]; then
  codexx_export_mirror_env
  codexx_install_system_prereqs
  codexx_ensure_rustup
  codexx_ensure_rust_toolchain
else
  codexx_export_mirror_env
fi

cargo_root="$(codexx_cargo_root)"
toolchain="$(codexx_toolchain)"
workspace_version="$(codexx_workspace_version)"
git_commit="$(codexx_git_commit)"
git_short_commit="$(codexx_git_short_commit)"

cd "$cargo_root"

export CODEXX_GIT_SHA="$git_commit"

codexx_log "Building debug binary for local run (${workspace_version}, ${git_short_commit})"
rustup run "$toolchain" cargo build -p codex-cli --bin codex

debug_binary="$cargo_root/target/debug/codex"
if [[ ! -f "$debug_binary" ]]; then
  codexx_die "expected debug build output not found: $debug_binary"
fi

codexx_log "Starting debug binary without packaging: $debug_binary"
exec "$debug_binary" "${run_args[@]}"
