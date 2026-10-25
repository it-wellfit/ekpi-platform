# Community skill assessment

- **Status:** Active; decision recorded, no additional bundle installed
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Installed skill set:** [skill setup](skill-setup.md)
- **Issue tracker:** [issue tracker](issue-tracker.md)

## Decision

The repository uses the **Matt Pocock Skills** set at the pinned release recorded in [skill setup](skill-setup.md). Two alternative community distributions were evaluated on 2026-10-26 and **not installed**:

| Candidate | Decision | Primary reason |
| --- | --- | --- |
| [mattpocock/skills](https://github.com/mattpocock/skills) | Installed (27 skills) | Direct upstream, MIT, tagged release, documented scope, alive and reviewable |
| [Ulteig/agent-skills](https://github.com/Ulteig/agent-skills) | Not installed | Diverged fork with no release; advantage over upstream is not demonstrated |
| [fdarkaou/agent-skills](https://github.com/fdarkaou/agent-skills) | Not installed | Curated 11-skill subset of upstream; no release; overlaps the installed set |

This is not a judgement that forks are inherently better or worse. It is a decision for this repository at this date, based on the criteria below and the evidence available.

## Evaluation criteria

| Criterion | What was checked | Why it matters |
| --- | --- | --- |
| Compatibility | Skill format/frontmatter, invocation model, alignment with OpenCode and the existing workflow | An incompatible bundle adds maintenance without usable capability |
| Documentation | README, skill instructions, changelog, scope statement, upstream provenance | Unclear scope makes behavior unpredictable |
| Verifiability | Tagged release, pinned commit, reviewable diff, tests or validation evidence | A moving `main` cannot be audited or reproduced |
| Security | License, provenance, third-party notices, permission/tool expectations, no credential requests | Skills influence agent actions and can import unsafe instructions |
| Maintenance | Release cadence, commit recency, open issues, responsiveness, forks/divergence | Abandoned or heavily diverged content becomes a silent liability |
| Cost | Installation, overlap, review burden, model/context cost, upgrade path | More skills are not free; unused or duplicated skills degrade routing |

## Evidence (observed 2026-10-26)

### Matt Pocock Skills (installed)

- License: MIT.
- Latest release observed: `v1.3.1` (2026-10-04).
- Latest commit observed: `5beb35b` (2026-10-19).
- Repository exposes a documented skill tree and an explicit scope/changelog.
- 27 skills were installed under `.opencode/skills/`; frontmatter was validated and the gate was run on the real repository (10/10 steps at that time).
- The set covers research, grilling, domain modeling, spec, tickets, TDD, implementation, review, diagnosis, prototype, retro and handoff.

**Residual risk:** the installed copy is a snapshot. Upgrades require reading the upstream changelog, revalidating frontmatter/routing, and rerunning the gate. Do not update skills in the middle of a release without recording the old/new revision.

### Ulteig/agent-skills (not installed)

- License: MIT.
- Releases: none observed.
- Latest commit observed: `7c67475` (2026-10-23).
- Repository presents itself as a fork derived from Matt Pocock's work.
- Comparison against upstream observed: **diverged, 27 commits ahead / 121 behind**, total 27 divergent commits.
- No release tag, no independent evaluation record and no demonstrated capability that the upstream set lacks.

**Reason not installed:** a diverged fork with no release cannot be pinned or audited as a stable dependency, and the divergence is large enough that upstream updates cannot be merged mechanically. Installing it would create a second, competing skill source with unclear provenance.

### fdarkaou/agent-skills (not installed)

- License: MIT; third-party notices state that the `skills/matt` content is redistributed from Matt Pocock under MIT.
- Releases: none observed.
- Latest commit observed: `1acbb6b` (2026-07-16).
- README describes an **11-skill curated bundle** for Codex/Claude Code covering shaping, domain documentation, triage, planning, TDD, Chrome validation, subagent orchestration and review.
- Open issues observed: 10.
- The bundle overlaps the 27 installed skills and adds a small number of workflow variants; it is not an independent capability set.

**Reason not installed:** no tagged release, curated overlap with the installed set, and 10 open issues without a maintenance/release signal. The incremental value does not justify a second source of truth for skill behavior.

## What would change the decision

Re-evaluate a candidate only if all of the following are true:

1. It publishes a tagged release or a pinned, reviewable commit with a changelog.
2. It documents compatibility and any required permissions/tools.
3. It has provenance and license clarity, including third-party notices.
4. It provides a capability the installed set lacks, with a concrete repository use case.
5. The install/upgrade path is reversible and the gate can validate the result.
6. The maintenance signal is credible: recent commits, addressed issues and a named maintainer.

A security review is required before installing any skill that can execute commands, access the network or handle credentials.

## Maintenance rules

- Record the source repository, release/commit and install date in [skill setup](skill-setup.md).
- Do not install from a moving branch without recording the exact commit.
- Keep the installed skill set minimal; remove a skill that is unused or duplicated rather than accumulating coverage.
- Re-run the repository gate after any skill change. A skill change is a tooling change, not a documentation-only change.
- Treat skill instructions as untrusted external content until reviewed; they do not override `AGENTS.md` or this repository's safety rules.

## Update triggers

Update this page when a candidate is evaluated, installed, upgraded or removed, or when the evaluation criteria change. Keep the dated evidence; do not rewrite history to make a past decision look better.
