#!/usr/bin/env bash
# One-time, retryable repository publication. No credentials are stored here.
set -Eeuo pipefail
export GH_HOST=github.com
export GH_PROMPT_DISABLED=1
export GIT_TERMINAL_PROMPT=0
repo='WigglyDon/gcx'
owner='WigglyDon'
seed_id='gcx-foundation-2026-10-21-v1'
description='A game-first, Rust-first GameCube player from Machine Rhythm. Project foundation; Android first.'
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$root"
die() { printf 'gcx bootstrap: %s\n' "$*" >&2; exit 1; }
trap 'printf "Publication stopped at line %s. Existing source and remote work were retained.\n" "$LINENO" >&2' ERR
for tool in git gh sha256sum flock awk sed grep cmp; do
    command -v "$tool" >/dev/null 2>&1 || die "Missing $tool; use your normal local installation, then rerun."
done
[[ -d .git && ! -L .git ]] || die 'Run this inside the fresh seeded checkout, not an unrelated worktree.'
[[ $(cat .gcx-seed-id 2>/dev/null) == "$seed_id" ]] || die 'Seed identity does not match.'
[[ ! -L .local && ! -L .local/bootstrap ]] || die 'Refusing redirected private bootstrap storage.'
mkdir -p .local/bootstrap
exec 9>.local/bootstrap/publish.lock
flock -n 9 || die 'Another bootstrap is using this checkout.'

gh auth status --hostname github.com >/dev/null 2>&1 || die 'Sign in locally with: gh auth login --hostname github.com'
identity=$(gh api user --jq '[.login, (.id | tostring)] | @tsv')
IFS=$'\t' read -r login user_id <<< "$identity"
[[ "$login" == "$owner" && "$user_id" =~ ^[0-9]+$ ]] || die "Authenticated account is not $owner. Switch accounts locally; nothing was published."

# Configure a commit identity only in this new repository, and only if absent.
if ! git config user.name >/dev/null; then git config user.name "$owner"; fi
if ! git config user.email >/dev/null; then
    git config user.email "${user_id}+${owner}@users.noreply.github.com"
fi
if ! git rev-parse --verify HEAD >/dev/null 2>&1; then
    [[ $(git symbolic-ref --short HEAD) == main ]] || die 'The seed must start on main.'
    sha256sum --quiet --check .seed/payload.sha256 || die 'Seed verification failed; preserve the files and inspect the difference.'
    mapfile -t seed_paths < <(sed -E 's/^[0-9a-f]{64}  //' .seed/payload.sha256)
    ((${#seed_paths[@]} > 0)) || die 'The seed manifest is empty.'
    git add -- "${seed_paths[@]}" .seed/payload.sha256
    git commit -m 'Initialize gcx product, GameCube feasibility, Master and Codex handoffs'
fi
[[ $(git branch --show-current) == main ]] || die 'Not on main; no automatic branch switch will be made.'
[[ -z $(git status --porcelain --untracked-files=normal) ]] || die 'Unexpected local changes exist. Preserve/reconcile them before publishing.'

# A failed authenticated read is not treated as an empty repository unless 404.
metadata_error=.local/bootstrap/repository-read.stderr
if metadata=$(gh api "repos/$repo" --jq '[.full_name, (.private | tostring), (.permissions.push | tostring)] | @tsv' 2>"$metadata_error"); then
    :
else
    if ! grep -q 'HTTP 404' "$metadata_error"; then
        cat "$metadata_error" >&2
        die 'Repository lookup failed for a reason other than absence; creation was not attempted.'
    fi
    gh repo create "$repo" --public --description "$description"
    metadata=$(gh api "repos/$repo" --jq '[.full_name, (.private | tostring), (.permissions.push | tostring)] | @tsv')
fi
IFS=$'\t' read -r full_name private can_push <<< "$metadata"
[[ "$full_name" == "$repo" && "$private" == false && "$can_push" == true ]] || die 'Owner, public visibility or push permission did not match. No visibility conversion will be attempted.'

expected_remote="https://github.com/$repo.git"
if origin=$(git config --get remote.origin.url 2>/dev/null); then
    case "$origin" in
        "$expected_remote"|"https://github.com/$repo"|"git@github.com:$repo.git") ;;
        *) die "Existing origin is not $repo; it was not changed." ;;
    esac
else
    git remote add origin "$expected_remote"
fi
# Invoke the existing gh credential provider without changing global Git config.
git_remote() { git -c credential.helper= -c 'credential.helper=!gh auth git-credential' "$@"; }
refs=$(git_remote ls-remote --heads --tags origin)
if [[ -n "$refs" ]]; then
    remote_main=$(awk '$2=="refs/heads/main" {print $1}' <<< "$refs")
    [[ -n "$remote_main" ]] || die 'Nonempty remote has no main. Refusing to seed an unrelated repository.'
    git_remote fetch --no-tags origin refs/heads/main:refs/remotes/origin/main
    git merge-base --is-ancestor refs/remotes/origin/main HEAD || die 'Remote main is unrelated or ahead. No force push, reset or automatic merge was performed.'
fi
git_remote push --set-upstream origin main

# Only ordinary new-project metadata: no collaborator, secret, auth or security changes.
gh api --method PATCH "repos/$repo" -f default_branch=main -f description="$description" \
    -F has_issues=true -F has_wiki=false -F has_projects=false >/dev/null
gh repo edit "$repo" --add-topic rust --add-topic gamecube --add-topic emulation --add-topic android >/dev/null
existing_labels=$(gh api "repos/$repo/labels?per_page=100" --paginate --jq '.[].name')
while IFS=$'\t' read -r name color label_description; do
    [[ -n "$name" ]] || continue
    if ! grep -Fxq -- "$name" <<< "$existing_labels"; then
        gh label create "$name" --repo "$repo" --color "$color" --description "$label_description"
    fi
done < .seed/labels.tsv

# Enumerate actual issues, not eventually consistent search results.
issue_rows=$(gh api "repos/$repo/issues?state=all&per_page=100" --paginate \
    --jq '.[] | select(.pull_request == null) | [.number, .title, .html_url] | @tsv')
receipt=.local/bootstrap/github-state.next.toml
root_commit=$(git rev-list --max-parents=0 HEAD)
[[ "$root_commit" =~ ^[0-9a-f]{40}$ ]] || die 'Unexpected multi-root history; inspect before generating a bootstrap receipt.'
cat > "$receipt" <<EOF
schema_version = 1
repository = "$repo"
repository_url = "https://github.com/$repo"
publication_status = "published_and_verified_by_local_bootstrap"
seed_id = "$seed_id"
seed_root_commit = "$root_commit"
# Records source publication and issue identities, not worker execution or gameplay.
EOF
while IFS=$'\t' read -r title body labels; do
    [[ -n "$title" && -f "$body" ]] || die 'Malformed prepared issue record.'
    matches=$(awk -F '\t' -v wanted="$title" '$2==wanted {print $1 "\t" $3}' <<< "$issue_rows")
    [[ "$matches" != *$'\n'* ]] || die "Multiple existing issues match: $title. Resolve without creating another."
    if [[ -n "$matches" ]]; then
        IFS=$'\t' read -r number issue_url <<< "$matches"
    else
        issue_url=$(gh issue create --repo "$repo" --title "$title" --body-file "$body" --label "$labels")
        [[ "$issue_url" =~ ^https://github\.com/WigglyDon/gcx/issues/([0-9]+)$ ]] || die 'Issue creation did not return the expected URL; inspect actual state before retrying.'
        number=${BASH_REMATCH[1]}
    fi
    [[ "$number" =~ ^[0-9]+$ ]] || die 'Unexpected issue number.'
    observed=$(gh api "repos/$repo/issues/$number" --jq '[.number, .title, .html_url] | @tsv')
    [[ "$observed" == "$number"$'\t'"$title"$'\t'"$issue_url" ]] || die 'Issue readback did not match the intended destination.'
    issue_key=${title%%:*}
    cat >> "$receipt" <<EOF

[[issues]]
key = "$issue_key"
number = $number
title = "$title"
url = "$issue_url"
EOF
    printf 'Verified %s\n' "$issue_url"
done < .seed/issues.tsv

if ! cmp -s "$receipt" docs/github-state.toml; then
    cp -- "$receipt" docs/github-state.toml
    git add -- docs/github-state.toml
    git commit -m 'Record verified gcx repository and issue bootstrap'
    git_remote push origin main
fi
remote_main=$(git_remote ls-remote origin refs/heads/main | awk 'NR==1 {print $1}')
[[ "$remote_main" == "$(git rev-parse HEAD)" ]] || die 'Final remote main differs from local HEAD; publication not claimed complete.'
api_main=$(gh api "repos/$repo/git/ref/heads/main" --jq '.object.sha')
[[ "$api_main" == "$(git rev-parse HEAD)" ]] || die 'GitHub API did not confirm the published main commit.'
final_meta=$(gh api "repos/$repo" --jq '[.full_name, (.private | tostring), .default_branch] | @tsv')
[[ "$final_meta" == "$repo"$'\tfalse\tmain' ]] || die 'Final owner/visibility/default branch verification failed.'
printf '\nPublic foundation verified: https://github.com/%s\nSource: %s\n' "$repo" "$remote_main"
printf 'Fresh Master: handoff/MASTER_SEED.md\nLocal worker: bash scripts/start-codex.sh\n'
