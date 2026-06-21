---
name: commit-message
description: Generate a Conventional Commits message from staged git changes, compatible with release-please. Use when the user asks for a commit message, says "what should the commit message be", or asks to commit without giving an explicit message.
---

# commit-message

Generate a Conventional Commits formatted message from the currently
staged changes.

## When to use

- The user asks for a commit message or asks to write one.
- The user asks "what should the commit message be?".
- The user asks to commit without giving an explicit message.

## Steps

1. Run `git diff --staged --stat` first to get an overview of what
   changed. If nothing is staged, ask the user to stage their changes
   and stop.
2. Run `git diff --staged` to read the actual changes.
3. Identify the dominant change type:
   - `feat` — new user-visible capability
   - `fix` — bug fix
   - `chore` — housekeeping (deps, CI, build, no functional change)
   - `docs` — documentation only
   - `refactor` — code restructure with no behavior change
   - `test` — tests only
   - `perf` — performance improvement
   - `ci` — CI / GitHub Actions changes
   - `style` — formatting only
4. Pick an optional scope from the affected crate(s): `core`, `service`,
   `ui`. Omit the scope if the change crosses crates or is repo-wide.
5. Write the subject line:
   - Imperative mood ("add X" not "added X").
   - Under 72 characters.
   - No trailing period.
6. Add a body explaining **why** the change is being made. The diff
   already shows what; the body should give the reason. Wrap at ~72
   columns. One paragraph is usually enough.
7. If the change is a breaking change to a public API, append `!` after
   the type/scope and add a `BREAKING CHANGE:` footer.
8. Output the final message inside a fenced block so the user can copy
   it or pipe it to `git commit -F -`.

## Output format

```
<type>(<scope>): <subject>

<body>
```

Examples:

```
feat(core): add Markdown round-trip parser

Implements the first half of the parse → serialize round trip that the
testing-strategy.md calls "the lifeline" for this project. Tests are
still ignored until the serialize side lands.
```

```
chore: bump rust-toolchain to 1.97 stable

Pulls in the new const generics defaults that the upcoming gpui
integration relies on.
```

## Do not

- Do not invent context that is not present in the diff.
- Do not write `update`, `change`, or `fix bug` as the subject — be
  specific about *what* changed and *why*.
- Do not touch `CHANGELOG.md`; release-please manages it.
- Do not add `Co-Authored-By` trailers unless the user asks.
