# Seed verification

Validated 2026-10-21 in the authoring environment. This records only this project's
foundation checks: no Dolphin compilation, Android package, gameplay, actual GitHub
publication, store approval or device operation ran here.

Bash syntax, local Markdown links, JSON/TOML/YAML metadata, prepared issue/label
references and payload checksums were checked. The complete Master handoff matches
the separately delivered file. This package contains source text and instructions,
not incorporated emulator source or proprietary game/firmware/save materials.

The bootstrap test harness used simulated GitHub CLI responses, actual local bare
Git repositories and an inert Codex launcher. All 11 cases passed:

1. fresh publication, four issues, receipt, Codex prompt and idempotent retry.
2. retry after accepted issue interruption reuses issue without duplication.
3. wrong GitHub account refuses before checkout/publication.
4. unrelated destination preserved.
5. non-404 lookup failure does not create a repository.
6. existing private repository is not made public.
7. unrelated remote history is not overwritten.
8. unrelated tag-only remote is not treated as empty.
9. dirty retry preserves work and does not publish it.
10. payload checksum mismatch refuses before checkout.
11. launch guard refuses a second script-launched Codex worker.

An earlier long test invocation timed out; no incomplete run was counted as a pass.
The complete final harness finished successfully after reducing mock-process startup
overhead. A tag-only remote was also hardened to count as nonempty, not a safe new
repository. Those are bootstrap checks, not evidence of external GitHub writes.

Actual publication requires the owner's existing local tools, authenticated
WigglyDon GitHub access and network. The script verifies returned metadata/issues
and the final commit before recording success. Source/work remains preserved on
failure; no force push, private-to-public conversion or broad cleanup is performed.
The launch lock guards concurrent uses of the provided launcher; it cannot detect
or stop every independently started external Codex process.
