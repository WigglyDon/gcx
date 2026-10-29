> Historical bootstrap/seed material. Superseded by AGENTS.md, docs/NTX_HANDOFF.md and NTX-5 including amendment6505289270. Do not recreate a checkout, repository or old qualification campaign from this file.

# Local setup

The intended remote is public `WigglyDon/gcx`. It was **not created by this chat**:
the available connector has no repository-creation action. The archive includes
all source files and a local GitHub CLI bootstrap; it does not need GitHub tokens
pasted into chat or into a script.

## Prerequisites

Linux/Bash 4+, Git, the authenticated GitHub CLI `gh`, and `sha256sum`/standard
utilities. Codex is needed only for `--start-codex`. Use your existing tools;
the bootstrap does not install packages or SDKs. If authentication is missing,
run `gh auth login --hostname github.com` locally and rerun. It verifies that
the authenticated account is exactly `WigglyDon`.

Save the delivered archive as `~/Downloads/gcx-seed.tar.gz`, then:

```bash
work="$(mktemp -d "$HOME/gcx-seed.XXXXXX")" &&
tar -xzf "$HOME/Downloads/gcx-seed.tar.gz" -C "$work" &&
bash "$work/gcx-seed/bootstrap.sh" "$HOME/gcx" --start-codex
```

The bootstrap creates a fresh local checkout from the supplied seed (so no
separate clone step is necessary), creates the public remote, pushes `main`,
enables issues, applies the prepared labels/topics, creates the four issue scopes,
commits a factual GitHub receipt and starts Codex with the included assignment.
It does not change global Git configuration or Codex approval settings.

An existing nonempty unrelated destination is refused, not overwritten. A partial
run can be resumed using the same bootstrap when the existing checkout carries
the same seed identity and has no unexpected changes. A nonempty unrelated remote
or divergent branch is refused. No force push, reset, cleanup, deletion or silent
private-to-public conversion is performed. Existing matching issues are reused.

If source publication succeeds but labels/issues/receipt fail, the script stops
and retains the checkout. Resolve the actual error, then run:

```bash
cd "$HOME/gcx"
bash scripts/publish-github.sh
bash scripts/start-codex.sh
```

After successful publication, another machine can use:

```bash
git clone https://github.com/WigglyDon/gcx.git
cd gcx
bash scripts/start-codex.sh
```

Do not use that clone command as evidence that the remote already exists.
Read `docs/github-state.toml` and the printed verified URLs.

## Fresh product Master

Attach the delivered `gcx-master-seed.md` to a new chat and say:

> Initialize as gcx's product Master from this seed. Verify the actual public
> repository and current issue/worker state before directing work. Evaluate the
> GameCube-specific design and guide the existing local Codex; do not duplicate it.

The same seed is versioned at `handoff/MASTER_SEED.md`. The local bootstrap makes
its GitHub location usable only after actual publication. The new Master needs
no additional private games, firmware or keys uploaded into chat.

## What the owner supplies later

One desired GameCube game and its legitimate local input, any exceptional local input
actually required by the evaluated route, and an explicit Fold operating window
when the first installable candidate is ready. No particular title, BIOS or
available phone window is assumed by the seed. Off-device work can start first.
