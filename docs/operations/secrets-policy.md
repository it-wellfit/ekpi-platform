# EKPI Secrets & Sensitive Data Handling Policy

## Core rules

1. **No secrets in the repository.** Do not commit passwords, API keys, tokens,
   private keys, or connection strings. CI scans the commit range for secrets
   and rejects pushes containing them.
2. **Configuration via environment variables and local config files.** Provide
   `.env.example` with placeholder names only (no real values). Real values live
   in `.env` (git-ignored) or environment, never in version control.
3. **Local-only governance prototype.** No enterprise authentication, no
   external-source permission enforcement, no AI-produced workflow drafts that
   self-publish. The prototype stores source *references*, not source-file
   contents.
4. **No prefilled organization facts.** Compliance deliverables must not
   prefill org-specific facts (`no_org_facts_prefilled` constraint).
5. **Sensitive data handling.** PII/org facts are never logged to stdout/stderr
   in the core path; audit events record governed actions, not credentials.

## `.env.example`

Placeholder names are documented here. Copy to `.env` and fill locally; `.env`
is git-ignored.

## Audit boundary

Audit trail (WORM + hash chain) records *that* an action happened and *by
which role*, not secrets. Evidence references point to a source document and
version; they do not copy authoritative source contents.

## Violation response

If a secret is accidentally committed: rotate the secret immediately, remove it
from history (rewrite + force-push only after team coordination), and record the
incident in `docs/operations/incident-log.md`.