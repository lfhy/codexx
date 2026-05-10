#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_TAG="${1:-${TARGET_TAG:-}}"
OUTPUT_FILE="${2:-${OUTPUT_FILE:-}}"
TEMPLATE_FILE="${3:-${TEMPLATE_FILE:-$ROOT_DIR/.github/release-template.md}}"

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing command: $1"
}

previous_tag() {
  local tag="$1"
  local candidate
  candidate="$(git -C "$ROOT_DIR" describe --tags --match 'v*' --abbrev=0 "${tag}^" 2>/dev/null || true)"
  printf '%s' "$candidate"
}

strip_prefix() {
  local subject="$1"
  case "$subject" in
    feat:*) printf '%s' "${subject#feat: }" ;;
    fix:*) printf '%s' "${subject#fix: }" ;;
    docs:*) printf '%s' "${subject#docs: }" ;;
    chore:*) printf '%s' "${subject#chore: }" ;;
    build:*) printf '%s' "${subject#build: }" ;;
    ci:*) printf '%s' "${subject#ci: }" ;;
    test:*) printf '%s' "${subject#test: }" ;;
    refactor:*) printf '%s' "${subject#refactor: }" ;;
    perf:*) printf '%s' "${subject#perf: }" ;;
    style:*) printf '%s' "${subject#style: }" ;;
    *) printf '%s' "$subject" ;;
  esac
}

collect_commits() {
  local range="$1"
  git -C "$ROOT_DIR" log --no-merges --format='%s%x09%h' "$range"
}

main() {
  [[ -n "$TARGET_TAG" ]] || fail "TARGET_TAG is required"
  [[ -n "$OUTPUT_FILE" ]] || fail "OUTPUT_FILE is required"
  [[ -f "$TEMPLATE_FILE" ]] || fail "Template not found: $TEMPLATE_FILE"
  require_cmd git
  require_cmd cat

  if [[ ! "$TARGET_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    fail "Unsupported tag format: $TARGET_TAG"
  fi

  local version="${TARGET_TAG#v}"
  local prev_tag=""
  prev_tag="$(previous_tag "$TARGET_TAG")"

  local range
  if [[ -n "$prev_tag" ]]; then
    range="$prev_tag..$TARGET_TAG"
  else
    range="$(git -C "$ROOT_DIR" rev-list --max-parents=0 "$TARGET_TAG")..$TARGET_TAG"
  fi

  local all_updates=""
  local highlights=""
  local fixes=""

  while IFS=$'\t' read -r subject hash; do
    [[ -n "${subject:-}" ]] || continue
    local normalized
    normalized="$(strip_prefix "$subject")"
    local entry="- \`$hash\` $normalized"
    all_updates+="${entry}"$'\n'
    case "$subject" in
      feat:*) highlights+="${entry}"$'\n' ;;
      fix:*) fixes+="${entry}"$'\n' ;;
    esac
  done < <(collect_commits "$range")

  if [[ -z "$all_updates" ]]; then
    all_updates='- 暂无新增提交。'
  else
    all_updates="${all_updates%$'\n'}"
  fi

  if [[ -z "$highlights" ]]; then
    highlights='- 暂无重点变更。'
  else
    highlights="${highlights%$'\n'}"
  fi

  if [[ -z "$fixes" ]]; then
    fixes='- 暂无修复项。'
  else
    fixes="${fixes%$'\n'}"
  fi

  {
    while IFS= read -r line || [[ -n "$line" ]]; do
      case "$line" in
        "{{ALL_UPDATES}}")
          printf '%s\n' "$all_updates"
          ;;
        "{{HIGHLIGHTS}}")
          printf '%s\n' "$highlights"
          ;;
        "{{FIXES}}")
          printf '%s\n' "$fixes"
          ;;
        *)
          printf '%s\n' "${line//\{\{VERSION\}\}/$version}"
          ;;
      esac
    done < "$TEMPLATE_FILE"
  } > "$OUTPUT_FILE"
}

main "$@"
