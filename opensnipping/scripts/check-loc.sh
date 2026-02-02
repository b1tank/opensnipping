#!/usr/bin/env bash
set -euo pipefail

LIMIT_DEFAULT=500

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"

if [[ -z "$REPO_ROOT" ]]; then
  echo "error: not in a git repo (needed for git ls-files)" >&2
  exit 2
fi

ALLOWLIST_FILE="$SCRIPT_DIR/loc.allowlist.tsv"

if [[ ! -f "$ALLOWLIST_FILE" ]]; then
  echo "error: missing allowlist at $ALLOWLIST_FILE" >&2
  exit 2
fi

# Allowlist format (TSV):
# path<TAB>max_lines<TAB>expires_yyyy-mm-dd<TAB>reason
# - path is repo-root-relative (e.g. opensnipping/src-tauri/src/lib.rs)
# - expires is enforced (expired entries fail)

declare -A allow_max
declare -A allow_expires

today="$(date -I)"

while IFS=$'\t' read -r path max_lines expires reason; do
  [[ -z "${path:-}" ]] && continue
  [[ "$path" =~ ^# ]] && continue

  if [[ -z "${max_lines:-}" || -z "${expires:-}" ]]; then
    echo "error: invalid allowlist row for '$path' (need: path<TAB>max_lines<TAB>expires<TAB>reason)" >&2
    exit 2
  fi

  allow_max["$path"]="$max_lines"
  allow_expires["$path"]="$expires"

done < "$ALLOWLIST_FILE"

fail=0
checked=0

while IFS= read -r -d '' relpath; do
  checked=$((checked + 1))
  abspath="$REPO_ROOT/$relpath"
  [[ -f "$abspath" ]] || continue

  lines="$(wc -l < "$abspath" | tr -d ' ')"

  max_allowed="$LIMIT_DEFAULT"
  expires=""

  if [[ -n "${allow_max[$relpath]:-}" ]]; then
    max_allowed="${allow_max[$relpath]}"
    expires="${allow_expires[$relpath]}"

    if [[ "$expires" < "$today" ]]; then
      echo "LOC FAIL (expired allowlist): $relpath has $lines lines (expired $expires)" >&2
      fail=1
      continue
    fi
  fi

  if (( lines > max_allowed )); then
    if [[ -n "${allow_max[$relpath]:-}" ]]; then
      echo "LOC FAIL (allowlisted max exceeded): $relpath has $lines lines (max $max_allowed; expires $expires)" >&2
    else
      echo "LOC FAIL: $relpath has $lines lines (limit $LIMIT_DEFAULT). Split it or add a temporary allowlist entry with expiry." >&2
    fi
    fail=1
  fi

done < <(git ls-files -z -- '*.rs' '*.ts' '*.tsx')

if (( fail != 0 )); then
  echo "\nChecked $checked files.\n" >&2
  exit 1
fi

echo "LOC OK: checked $checked files (limit $LIMIT_DEFAULT)."
