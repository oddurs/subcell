# Working in this repository

Instructions for coding agents. Follow them exactly; they override defaults.

## Attribution

Never attribute work to an AI, an assistant, or a model. Not in commit messages,
trailers, PR bodies, code comments, docs, changelogs, or release notes. No
`Co-Authored-By` naming a model, no "generated with" footers, no robot emoji.
The `commit-msg` hook rejects these, but do not rely on it — do not write them.

Everything here is published under the owner's name.

## The workflow

`main` only ever advances through a merged pull request. Never commit to it.

One unit of work, one worktree, one branch, one PR. Two agents must never share
a checkout.

```sh
scripts/agent doctor                    # verify the environment first
scripts/agent start feat/<slug>         # creates ../.worktrees/subcell/feat-<slug>
cd ../.worktrees/subcell/feat-<slug>
scripts/agent check
scripts/agent commit "feat(blit): ..."
scripts/agent pr
scripts/agent done                      # after it merges
```

Branch names: `<type>/<slug>` where type is `feat`, `fix`, `chore`, `docs`,
`perf`, `refactor` or `test`. When there is a backlog item, lead with its id:
`feat/0004-octant-table`.

## The task seam

All automation goes through one interface. Use it; do not call `cargo` directly
in scripts, hooks, or CI.

```sh
scripts/task fmt        scripts/task lint      scripts/task test
scripts/task fmt:check  scripts/task build     scripts/task docs
scripts/task check      # all of the above; what CI runs
```

## Commits

Conventional Commits, imperative mood, subject under 72 characters, no trailing
period. The body explains **why**; the diff already says what. Reference the
backlog item in a `Refs:` trailer.

```
fix(blit): correct the sextant exclusion offset

Masks above the right-column pattern were shifted by one, so every glyph
past the first exclusion was off by a codepoint. The exclusion count has to
be applied cumulatively.

Refs: 0012
```

## Code standards

- Clippy runs with `-D warnings`, including `pedantic`. If a lint is genuinely
  wrong for a piece of code, add a narrowly scoped `#[allow]` **with a comment
  saying why**. Never widen the workspace lint config to silence one site.
- `unsafe_code` is `forbid` at the workspace level.
- Every public item needs a doc comment; `missing_docs` is a warning that CI
  denies.
- New behaviour needs a test that would fail without the change.

## The rule this project rests on

**Resolution is a runtime property.** No code below the facade may assume a
tier, and no geometry may be stored in whole cells. If you find yourself writing
`x: u16` for a position, or branching on a specific terminal name, stop — that
is the mistake this whole design exists to prevent.

Every capability degrades independently. There is no "high" and "low" mode.

## Codepoints and escape sequences

This is the most likely place to ship a bug, because a wrong mapping still
compiles, still renders something, and still passes a test that was written from
the same wrong assumption.

- Never hand-write a glyph table that could be generated. See item `0004`.
- Assert independently verified anchors, not just self-consistency.
- Cite the source you checked against, in the PR and in a code comment.
- Malformed replies from a terminal must never panic, hang, or allocate
  unboundedly. A hostile terminal is in the threat model; see `SECURITY.md`.

## Documentation

`docs/` explains the *why*; rustdoc explains the *what*. When you change
behaviour that a doc describes, update the doc in the same PR.

Never claim something works when it does not. The README has an explicit "not
built yet" section and it must stay accurate — an aspirational README is worse
than no README.

<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a
Markdown file under `.cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead,
so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next` — what is ready to start. It excludes anything blocked by
   unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work.
   `cairn claim --next` picks and claims the top-ranked unclaimed item in one
   step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for
   fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why
   you chose something, what you tried, what to watch for.
4. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
5. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --json                 # ready work, ranked
cairn claim --next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate docs/roadmap.md
```

### Schema

- **Types**: `feature`, `bug`, `chore`, `docs`, `milestone`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active),
  `blocked` (active), `done` (done), `dropped` (dropped)
- **`milestone`**: names a `milestone` item, by key — what this ships in
- **`due`**: date, YYYY-MM-DD — when a milestone is meant to land
- **`part_of`**: names any items, by id, several allowed
- **`priority`**: one of p0, p1, p2, p3 — p0 is a release blocker
- **`effort`**: one of s, m, l, xl — rough size, not an estimate
- **`area`**: free text — subsystem this touches
- **Milestones**: `v0.1`, `v0.2`, `v0.3`
- **Saved views** (`cairn list --view NAME`): `now`, `next`, `triage`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to
   `cairn.toml` first.
3. Never hand-edit `docs/roadmap.md` — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->
