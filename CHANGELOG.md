# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the major version is `0`, breaking changes may land in any release.

## [Unreleased]

### Fixed

- `scripts/agent done` left the local branch behind. `ROOT` is derived from the
  script's own path, so inside a worktree it pointed at the directory being
  deleted; the script then aborted when its working directory vanished. The
  primary checkout is now resolved from `git worktree list`.
- `.githooks/pre-push` did not block direct pushes to the default branch. The
  branch name was resolved by piping `git symbolic-ref` into `sed`, and a
  pipeline's exit status is its last command's, so the `|| echo main` fallback
  never fired and the guard compared against an empty string.

[Unreleased]: https://github.com/oddurs/subcell/commits/main
