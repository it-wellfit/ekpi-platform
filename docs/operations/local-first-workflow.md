# EKPI Local-First Workflow

## Principle

All development occurs on the local clone. No feature, check, or test requires
network access to function. Remote collaboration (GitHub) is an *opt-in upgrade*
governed by explicit criteria (see MASTER PROMPT plan §S10), never a default.

## Offline-capable Git

- Git is the single source of truth. All commits are made locally first.
- Transport without a remote: `git bundle create <file> --all` to move history
  between machines; `git clone <bundle>` or `git bundle unbundle` to receive.
- Offline-usable commands: `git status`, `git diff`, `git log`, `git commit`,
  `git checkout` (local branches), `git stash`.
- Never gate behavior on a remote API, external LLM, or cloud service in the
  core adjudication path. PM4Py sidecar and LLM are optional oracles only;
  the deterministic engine decides (LLM proposes, engine decides).

## Per-change workflow (TDD + review)

1. Read AGENTS.md and affected code/tests.
2. Create or link an issue/spec with acceptance criteria and risk tier.
3. Write a failing regression test first.
4. Implement the minimal change to pass.
5. Rerun affected checks (`scripts/check.ps1` / `scripts/check.sh`).
6. Open a PR; require CI success + human approval before merge.
7. High-risk changes need an independent reviewer.

## Where to run checks

Run `scripts/check.ps1` (PowerShell) or `scripts/check.sh` (POSIX shell) from
the repo root. They cover both `eval/` and `platform/` crates: fmt, clippy,
tests, and are fully offline.

## No premature remoteization

Do not push to a remote, enable branch protection, or configure CI remotes until
all remote-collaboration upgrade criteria (plan §S10) are satisfied.