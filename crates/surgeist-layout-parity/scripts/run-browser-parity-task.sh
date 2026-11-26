#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

usage() {
    printf 'Usage: %s <parity-all|corpus-check|taffy-check>\n' "${0##*/}" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

task="$1"

case "${task}" in
    parity-all)
        unset SURGEIST_PARITY_FILTER
        export SURGEIST_PARITY_RAW_EXACT=1
        exec cargo test -j 1 --offline --locked -p surgeist-layout-parity --lib browser_parity::tests::runs_all_checked_in_browser_parity_xml -- --exact
        ;;
    corpus-check)
        unset SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
        unset SURGEIST_LAYOUT_GENERATE_FILTER
        unset SURGEIST_BROWSER_PATH
        unset SURGEIST_BROWSER_CACHE
        unset SURGEIST_BROWSER_VERSION
        exec cargo run -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --bin surgeist-layout-generate -- check-corpus
        ;;
    taffy-check)
        unset SURGEIST_LAYOUT_BROWSER_PARITY_ROOT
        unset SURGEIST_LAYOUT_GENERATE_FILTER
        unset SURGEIST_BROWSER_PATH
        unset SURGEIST_BROWSER_CACHE
        unset SURGEIST_BROWSER_VERSION
        exec cargo run -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --bin surgeist-layout-generate -- check-taffy-corpus
        ;;
    *)
        printf 'Unknown task: %s\n' "${task}" >&2
        usage
        exit 64
        ;;
esac
