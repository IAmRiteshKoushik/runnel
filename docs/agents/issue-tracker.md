# Issue tracker

Issues and specs live in GitHub Issues for `IAmRiteshKoushik/runnel`.
Use the `gh` CLI from this repo.

## Operations

- Verify the target with `gh repo view --json nameWithOwner` before
  creating the first issue in a session.
- Create an issue with `gh issue create --title "..." --body-file <path>`.
- Read an issue with
  `gh issue view <number> --json number,title,body,labels,comments`.
- List issues with `gh issue list --state open`, using label filters
  when appropriate.
- Comment with `gh issue comment <number> --body-file <path>`.
- Apply or remove labels with
  `gh issue edit <number> --add-label "..."` or `--remove-label "..."`.
- Before closing an issue, post the explanation as a separate comment,
  then run `gh issue close <number>`.

Use temporary files for multiline issue bodies and comments.
Create issues only in this repo.

## Relationships

Use native GitHub sub-issues and issue dependencies when supported by
the installed CLI or GitHub API. Otherwise, add `Part of #<parent>`
and `Blocked by: #<number>` lines to the child issue body.

An issue is unblocked when every blocker is closed.

## Pull requests as a triage surface

PRs as a request surface: no.

## Skill instructions

"Publish to the issue tracker" means create a GitHub issue.
"Fetch the relevant ticket" means read the corresponding GitHub issue.

## Wayfinding

A wayfinding map is an issue labelled `wayfinder:map`.
Link child tickets as sub-issues, or use a task list in the map and
a `Part of #<map>` line in each child.

Child tickets use `wayfinder:research`, `wayfinder:prototype`,
`wayfinder:grilling`, or `wayfinder:task`.

The next ticket is the first open child in map order with no open
blockers and no assignee. Claim it with
`gh issue edit <number> --add-assignee @me`.

To resolve a ticket, comment with the answer, close it, and append
a short finding with a link to the map's Decisions-so-far section.
