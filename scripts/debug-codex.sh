#!/bin/bash

# Set "chatgpt.cliExecutable": "/Users/<USERNAME>/code/codex/scripts/debug-codex.sh" in VSCode settings to always get the 
# latest codex-rs binary when debugging Codex Extension.


set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
exec "$repo_root/scripts/run-codexx-debug.sh" --skip-bootstrap -- "$@"
