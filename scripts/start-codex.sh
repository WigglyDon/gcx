#!/usr/bin/env bash
# Fresh interactive successor; retain installed Codex configuration.
set -Eeuo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$root"
[[ -f AGENTS.md && -f docs/NTX_HANDOFF.md ]] || { printf 'Missing ntx handoff.\n' >&2; exit 1; }
command -v codex >/dev/null 2>&1 || { printf 'Codex is not on PATH.\n' >&2; exit 1; }
command -v flock >/dev/null 2>&1 || { printf 'flock is required.\n' >&2; exit 1; }
exec 8>.git/ntx-codex-launch.lock
flock -n 8 || { printf 'A launcher already holds this checkout.\n' >&2; exit 1; }
printf 'Starting the ntx successor in %s; the outgoing worker must have stopped.\n' "$root"
exec codex -C "$root" "$(cat handoff/CODEX_START.txt)"
