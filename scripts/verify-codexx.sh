#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/scripts/lib/codexx-build-common.sh"

if [[ "${1:-}" == "--skip-bootstrap" ]]; then
  shift
  codexx_export_mirror_env
  codexx_export_build_cache_env
  codexx_start_sccache
else
  codexx_prepare_build_env
fi

cargo_root="$(codexx_cargo_root)"
toolchain="$(codexx_toolchain)"

cd "$cargo_root"
export CARGO_INCREMENTAL=0

codexx_log "Verifying fork build with cargo check"
rustup run "$toolchain" cargo check -p codex-cli --bin codex "$@"
