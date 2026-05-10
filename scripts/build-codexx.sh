#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/scripts/lib/codexx-build-common.sh"

profile="debug"
skip_bootstrap="0"
run_after_build="0"
run_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      profile="release"
      shift
      ;;
    --skip-bootstrap)
      skip_bootstrap="1"
      shift
      ;;
    --run)
      run_after_build="1"
      shift
      ;;
    --)
      shift
      run_args=("$@")
      break
      ;;
    *)
      codexx_die "unknown argument: $1"
      ;;
  esac
done

if [[ "$skip_bootstrap" == "0" ]]; then
  codexx_prepare_build_env
else
  codexx_export_mirror_env
fi

cargo_root="$(codexx_cargo_root)"
build_root="$(codexx_build_root)"
toolchain="$(codexx_toolchain)"

codexx_log "Building fork artifact codexx (${profile})"
cd "$cargo_root"

if [[ "$profile" == "release" ]]; then
  rustup run "$toolchain" cargo build -p codex-cli --bin codex --release
else
  rustup run "$toolchain" cargo build -p codex-cli --bin codex
fi

artifact_dir="$cargo_root/target/$profile"
source_binary="$artifact_dir/codex"
fork_binary="$build_root/codexx"

if [[ ! -f "$source_binary" ]]; then
  codexx_die "expected build output not found: $source_binary"
fi

mkdir -p "$build_root"
install -m 755 "$source_binary" "$fork_binary"
codexx_log "Fork binary ready: $fork_binary"

if [[ "$run_after_build" == "1" ]]; then
  exec "$fork_binary" "${run_args[@]}"
fi
