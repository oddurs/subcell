# Security policy

## Supported versions

`subcell` is pre-`0.1`. Only the latest release receives fixes; there are no
maintained release branches yet. This section will grow a table once `0.1.0`
ships.

## Reporting a vulnerability

Please report privately through GitHub Security Advisories:

**<https://github.com/oddurs/subcell/security/advisories/new>**

Do not open a public issue for a security problem.

Include what you can — affected crate and version, what an attacker gains, and
a reproduction. A proof of concept is welcome but not required.

## What to expect

- An acknowledgement within three working days.
- An initial assessment, including whether it is in scope, within a week.
- Credit in the advisory and the changelog, unless you would rather not be
  named.

## Scope

`subcell` writes escape sequences to a terminal and parses replies from it. The
interesting attack surface is therefore:

- **Reply parsing.** A hostile or buggy terminal can send arbitrary bytes in
  response to a probe. Parsers must not panic, hang, or allocate unboundedly on
  malformed input.
- **Escape sequence construction.** Application data reaching the terminal
  unescaped could inject control sequences. Anything that interpolates
  caller-supplied text into an escape sequence is in scope.
- **Image payloads.** Dimension and length mismatches must be rejected rather
  than trusted.

Out of scope: denial of service caused by the caller asking for an
unreasonably large render, and anything requiring the attacker to already
control the process.
