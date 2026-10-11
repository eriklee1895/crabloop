#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 1 ]; then
    printf 'usage: verify-playground.sh WORKSPACE\n' >&2
    exit 2
fi
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
work_abs=$(CDPATH= cd -- "$1" && pwd -P)
verify_tmp=$(mktemp -d "${TMPDIR:-/tmp}/crabloop-verify.XXXXXX")
trap 'rm -rf -- "$verify_tmp"' EXIT
CRABLOOP_SOLUTION="$work_abs/lib.rs" rustc --edition=2024 --test "$repo_dir/fixtures/verifier/verify.rs" -o "$verify_tmp/tests"
"$verify_tmp/tests"
