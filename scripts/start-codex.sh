#!/usr/bin/env bash
# Normal interactive Codex invocation; existing approval settings remain intact.
set -Eeuo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$root"
[[ $(cat .gcx-seed-id 2>/dev/null) == 'gcx-foundation-2026-10-21-v1' ]] || {
    printf 'This is not the expected gcx checkout.\n' >&2; exit 1;
}
command -v codex >/dev/null 2>&1 || {
    printf 'Codex is not on PATH. Open this checkout in your existing Codex installation and paste handoff/CODEX_START.txt.\n' >&2
    exit 1
}
grep -q '^publication_status = "published_and_verified_by_local_bootstrap"$' docs/github-state.toml || {
    printf 'Publish and verify the foundation first: bash scripts/publish-github.sh\n' >&2; exit 1;
}
command -v flock >/dev/null 2>&1 || { printf 'flock is required for the local worker launch guard.\n' >&2; exit 1; }
exec 8>.git/gcx-codex-launch.lock
flock -n 8 || { printf 'A worker launched by this script is already using this checkout.\n' >&2; exit 1; }
printf 'Starting the gcx assignment in %s. Do not start a second worker for this checkout.\n' "$root"
exec codex "$(cat handoff/CODEX_START.txt)"
