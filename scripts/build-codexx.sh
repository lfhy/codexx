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
workspace_version="$(codexx_workspace_version)"
git_commit="$(codexx_git_commit)"
git_short_commit="$(codexx_git_short_commit)"

codexx_log "Building fork artifact codexx (${profile}, v${workspace_version}, ${git_short_commit})"
cd "$cargo_root"

export CODEXX_GIT_SHA="$git_commit"

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
cat > "$build_root/codexx.version.txt" <<EOF
name=CodexX
version=${workspace_version}
commit=${git_commit}
short_commit=${git_short_commit}
profile=${profile}
built_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
artifact=${fork_binary}
EOF
codexx_log "Fork binary ready: $fork_binary"
codexx_log "Build metadata ready: $build_root/codexx.version.txt"

if [[ "$run_after_build" == "1" ]]; then
  exec "$fork_binary" "${run_args[@]}"
fi
