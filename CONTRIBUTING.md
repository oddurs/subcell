# Contributing

Thanks for taking an interest. This project is small and the workflow is
deliberately strict, so that the wrong thing is hard to do by accident rather
than merely discouraged.

## Setup

```sh
git clone https://github.com/oddurs/subcell
cd subcell
scripts/setup          # wires git hooks — required
scripts/agent doctor   # verifies your environment
```

`scripts/setup` points `core.hooksPath` at the tracked `.githooks/`. Without it
you will get past checks locally that CI will then fail.

## The workflow

**`main` only ever advances through a merged pull request.** This is enforced by
a repository ruleset on the server and by `.githooks/pre-push` locally.

**One unit of work, one worktree, one branch, one PR.** Worktrees live outside
the repository at `../.worktrees/subcell/<branch>/`, so two pieces of work — or
two agents — never share a checkout and never fight over the index.

```sh
scripts/agent start fix/sextant-boundary
cd ../.worktrees/subcell/fix-sextant-boundary

# ... make the change ...

scripts/agent check
scripts/agent commit "fix(blit): correct the sextant exclusion offset"
scripts/agent pr

# after it merges:
scripts/agent done
```

`scripts/agent done` confirms the PR is merged, then removes the worktree, the
local branch, and the remote branch.

## Branch names

`<type>/<slug>`, where type is one of `feat`, `fix`, `chore`, `docs`, `perf`,
`refactor`, `test`. When the work has a backlog item, lead the slug with its
identifier: `fix/0012-sextant-boundary`.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), imperative mood,
subject under 72 characters, no trailing period. The `commit-msg` hook rejects
anything else.

```
fix(blit): correct the sextant exclusion offset

Masks above the right-column pattern were shifted by one, so every glyph
past U+1FB29 was off by a codepoint. The exclusion count has to be applied
cumulatively, not as a single boolean.

Refs: 0012
```

The body explains **why**; the diff already says what.

## Checks

Everything goes through one seam:

```sh
scripts/task fmt       # format in place
scripts/task check     # fmt:check && lint && test && build
```

CI runs exactly `scripts/task check`, so local and remote cannot drift. Clippy
runs with `-D warnings`, including `pedantic`. If a lint is genuinely wrong for
a piece of code, add a narrowly scoped `#[allow]` with a comment saying why —
never widen the workspace lint configuration to silence one site.

## What makes a good pull request

State the problem, the approach, and anything a reviewer should look at
sceptically. If a change has a known weakness, say so in the PR rather than
letting a reviewer find it.

For anything touching a glyph table or an escape sequence, include the source
you checked it against. Getting a codepoint mapping subtly wrong is the most
likely way to ship a bug here, and it will not show up as a test failure unless
the test encodes an independently verified anchor.

## Reviews

Approvals are not currently required to merge — with a single maintainer, a
mandatory approval would deadlock the repository rather than protect it. The
status check and the pull-request requirement are still enforced for everyone,
including the owner. This moves to one required approval as soon as there is a
second maintainer.

## The backlog

Work is tracked in the repository itself, as Markdown, with
[cairn](https://github.com/oddurs/cairn):

```sh
cairn next               # what is ready to work on now
cairn board              # kanban view
cairn show 3             # one item in full
cairn claim 3            # take it
```

Items live in `.cairn/`. Because they are plain files, changing one is a normal
part of a pull request and gets reviewed alongside the code.
