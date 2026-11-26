#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

usage() {
    printf 'Usage: %s <fmt|fmt-check|check|test|clippy>\n' "${0##*/}" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

task="$1"

case "${task}" in
    fmt)
        exec cargo fmt -p surgeist-layout-parity
        ;;
    fmt-check)
        exec cargo fmt -p surgeist-layout-parity -- --check
        ;;
    check)
        exec cargo check -j 1 --offline --locked -p surgeist-layout-parity --all-targets
        ;;
    test)
        exec cargo test -j 1 --offline --locked -p surgeist-layout-parity
        ;;
    clippy)
        exec cargo clippy -j 1 --offline --locked -p surgeist-layout-parity --all-targets -- -F unsafe-code -D warnings
        ;;
    *)
        printf 'Unknown task: %s\n' "${task}" >&2
        usage
        exit 64
        ;;
esac
