#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cd "${repo_root}"

"${script_dir}/run-cargo-task.sh" fmt-check
"${script_dir}/run-cargo-task.sh" check
"${script_dir}/run-cargo-task.sh" test
"${script_dir}/run-cargo-task.sh" clippy
