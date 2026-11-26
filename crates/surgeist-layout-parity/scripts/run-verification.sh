#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

usage() {
    printf 'Usage: %s <default|browser>\n' "${0##*/}" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

mode="$1"

case "${mode}" in
    default)
        "${script_dir}/run-cargo-task.sh" fmt-check
        "${script_dir}/run-cargo-task.sh" check
        "${script_dir}/run-cargo-task.sh" test
        "${script_dir}/run-cargo-task.sh" clippy
        ;;
    browser)
        "${script_dir}/run-cargo-task.sh" browser-check
        "${script_dir}/run-cargo-task.sh" browser-test
        "${script_dir}/run-cargo-task.sh" browser-clippy
        ;;
    *)
        printf 'Unknown mode: %s\n' "${mode}" >&2
        usage
        exit 64
        ;;
esac
