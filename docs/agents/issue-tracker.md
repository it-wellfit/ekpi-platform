**Repository:** ekpi-platform  
**Issue tracker:** GitHub Issues  
**PRs as a request surface:** no

## Bootstrap status

This checkout has no Git remote, and the existing GitHub repository URL has not been supplied. Git for Windows and GitHub CLI are installed, but gh auth status reports no authenticated GitHub host. Connect the intended existing repository as origin and authenticate with gh auth login before creating issues, linking tickets, opening PRs, or provisioning labels. Do not create a new repository or fall back to local markdown tickets.

## Conventions

- Create an issue with gh issue create --title "..." --body-file <path>.
- Read an issue with gh issue view <number> --json number,title,body,labels,comments.
- List issues with gh issue list --state open --json number,title,body,labels,comments.
- Comment, label, and close with gh issue comment, gh issue edit, and gh issue close.
- A multi-ticket feature uses one parent map/spec issue and linked child tickets. Prefer GitHub native sub-issues and issue dependencies; if unavailable, include Part of #<parent> and Blocked by: #<issue> in the child body.
- Specs live in issues and link to applicable governance requirement IDs and repo documents. A PR links its issue and spec.
- Use the issue forms in `.github/ISSUE_TEMPLATE/`: `feature.yml` for a proposed behavior change, `ticket.yml` for one independently verifiable slice, `improvement.yml` for an accepted retro/review/incident action, and `bug.yml` for a defect. The forms require requirement/contract IDs, risk tier, phase-gate impact and verification evidence so the issue can be traced before work starts.
- A missing field is written as `None` with a reason, not left blank or filled with an invented value. A proposed requirement stays "not yet accepted" until the accountable owner accepts it.

## Wayfinding operations

Used by /wayfinder:

- Create the map issue with label wayfinder:map; it holds Notes, Decisions-so-far, Fog, and a task list.
- Create child issues with wayfinder:research, wayfinder:prototype, wayfinder:grilling, or wayfinder:task. Add native dependency edges where available.
- Claim a ready ticket by assigning it to the current user. Resolve it by recording the answer, closing the ticket, and appending a context pointer to the map.
- Provision the labels listed in docs/agents/triage-labels.md plus the wayfinder:* labels after the remote and authentication are configured. Missing labels make gh issue create --label fail.

## When a skill says

- “Publish to the issue tracker”: create a GitHub issue.
- “Fetch the relevant ticket”: read the GitHub issue using gh issue view.
- “Make a PR”: use /pr, then gh pr create only after local checks and review are complete.
