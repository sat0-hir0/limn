#!/usr/bin/env bash
# scripts/helpers/lib.sh — shared bash helpers for the verifier scripts.
#
# Source from a verifier with:
#     SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#     source "$SCRIPT_DIR/../helpers/lib.sh"

set -euo pipefail

# Colour codes — turn off if not a TTY or if NO_COLOR is set.
if [[ -t 2 && -z "${NO_COLOR:-}" ]]; then
    LIB_RED=$'\033[0;31m'
    LIB_YELLOW=$'\033[0;33m'
    LIB_GREEN=$'\033[0;32m'
    LIB_GREY=$'\033[0;90m'
    LIB_RESET=$'\033[0m'
else
    LIB_RED=''
    LIB_YELLOW=''
    LIB_GREEN=''
    LIB_GREY=''
    LIB_RESET=''
fi

# log_info <msg> — informational to stderr.
log_info() {
    printf '%s[info]%s %s\n' "$LIB_GREY" "$LIB_RESET" "$*" >&2
}

# log_warn <msg> — warning to stderr.
log_warn() {
    printf '%s[warn]%s %s\n' "$LIB_YELLOW" "$LIB_RESET" "$*" >&2
}

# log_error <msg> — error to stderr (does not exit).
log_error() {
    printf '%s[error]%s %s\n' "$LIB_RED" "$LIB_RESET" "$*" >&2
}

# log_ok <msg> — success to stderr.
log_ok() {
    printf '%s[ok]%s %s\n' "$LIB_GREEN" "$LIB_RESET" "$*" >&2
}

# die <msg> — log to stderr and exit 2 (distinguished from verifier
# findings, which use exit 1).
die() {
    log_error "$*"
    exit 2
}

# require_cmd <command> — die if a command is missing from PATH.
require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        die "required command not found: $cmd"
    fi
}

# repo_root — print the repository root (cd-friendly).
repo_root() {
    git rev-parse --show-toplevel
}

# json_finding <key=value...> — emit a single JSON line for a finding.
# Values are passed through `jq` for proper escaping.
json_finding() {
    require_cmd jq
    local args=("$@")
    local pairs=()
    for arg in "${args[@]}"; do
        local k="${arg%%=*}"
        local v="${arg#*=}"
        pairs+=("--arg" "$k" "$v")
    done
    jq -nc "${pairs[@]}" '$ARGS.named'
}
