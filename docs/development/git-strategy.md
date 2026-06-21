# Git strategy

Limn is an OSS desktop application maintained by one person with AI
assistance, with plans to onboard contributors in the future. The
workflow is GitHub Flow optimized for that context.

## TL;DR

- **GitHub Flow** as the base, skewed **trunk-based** — incomplete
  features are hidden with a [feature flag](feature-flags.md) and merged
  into main rather than kept on long-lived branches
- **Branches**: short-lived (1–3 days) `feat/<topic>` `fix/<topic>`
  `docs/<topic>` `chore/<topic>` `refactor/<topic>`
- **Merge**: **squash-and-merge** (1 PR = 1 commit in main)
- **Release**: [`release-please`](https://github.com/googleapis/release-please)
  watches main → creates a release PR → merging it creates the tag and
  GitHub Release
- **Direct push to main**: only for trivial changes (typos / single-line
  doc edits / obviously correct CI fixes). Features and API changes
  require a pull request
- **History rewrite**: **never** after going public (sole exception:
  emergency response to a leaked secret)
- **Branch protection**: force push disabled, all CI jobs required, no
  self-approval needed (single-maintainer operation)

## Why GitHub Flow

| Option | Fit for Limn | Reason |
|---|---|---|
| **GitHub Flow** | Best | Standard for single-person OSS, works with release-please, immediately understood by contributors |
| Trunk-Based Development | Partial | We adopt the feature toggle approach, but a dedicated CI/feature flag infrastructure would be overkill |
| GitFlow | Poor | A `develop` branch is too heavy for a single-maintainer repo |
| GitLab Flow | Poor | Environment separation (staging / production) is not needed |
| Release Flow (Microsoft) | Poor | Parallel version support is unnecessary during v0.x |

GitHub Flow is skewed trunk-based so that incomplete features are hidden
with a [feature flag](feature-flags.md) and merged into main rather than
held on long-lived branches. Main is always in a "working and
releasable" state.

## Branch structure

```
main ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      ┃   ┃   ┃   ┃
      f1  f2  f3  f4   (short-lived feature branches, merged within 1–3 days)
```

### Branch naming

Aligning prefixes with
[Conventional Commits](https://www.conventionalcommits.org/) types
keeps branch names, PR titles, and commit messages consistent.

| Prefix | Purpose |
|---|---|
| `feat/<topic>` | New feature |
| `fix/<topic>` | Bug fix |
| `docs/<topic>` | Documentation only |
| `chore/<topic>` | Housekeeping (dependency updates, etc.) |
| `refactor/<topic>` | Refactor without behavior change |
| `test/<topic>` | Test additions or fixes only |
| `perf/<topic>` | Performance improvement |
| `ci/<topic>` | CI / Actions changes |

`<topic>` is a short kebab-case slug (e.g. `feat/m2-editing`,
`fix/wayland-startup`, `chore/bump-gpui-rev`).

### Branch lifetime

- Aim to merge within a few hours to 3 days
- If a branch runs past a week, **split the scope and ship the smaller
  part first**, or [merge incomplete code into main hidden behind a
  feature flag](feature-flags.md#stage-1-hidden)
- Long-lived branches carry a high risk of merge conflicts and stale code

## Direct push to main

### Allowed

- Typo fixes in docs
- Comment edits
- Obvious single-line CI config fixes
- Merging Dependabot pull requests
- Merging `release-please` pull requests
- Any change that is unambiguously mechanical

### Not allowed (pull request required)

- Feature additions or removals
- API or public behavior changes
- Design changes
- Multi-file changes that are not purely mechanical

## Merge strategy: squash-and-merge

- 1 PR = 1 commit in main
- The PR title becomes the squashed commit message →
  **PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/) format**
- Fine-grained commits such as "fix typo" or "address review" inside the
  PR do not appear in history
- `release-please` reads the squashed commits, so the PR title type
  flows directly into `CHANGELOG.md`

Example:

```
PR title: feat(ui): add slash palette
            ↓ squash-and-merge
main: <SHA> feat(ui): add slash palette
            ↓ release-please
CHANGELOG: ## Features
             * **ui:** add slash palette ([#N](...))
```

## Release strategy

[`release-please`](https://github.com/googleapis/release-please) drives
the release cycle:

1. release-please watches every push to main
2. It aggregates Conventional Commits and creates or updates a
   `chore: release v0.X.Y` pull request
3. Merging the release PR automatically:
   - cuts a `v0.X.Y` tag at `HEAD` of main
   - creates a GitHub Release
   - uses the `CHANGELOG.md` diff as release notes
4. During 0.x: **minor bump covers both feature additions and breaking
   changes** (allowed by SemVer 0.x)
5. **Patch (0.0.1 increments)** is reserved for bug fixes only
6. **1.0.0** is cut when AI integration (M5) is complete, or at another
   stability milestone

## History rewrite policy

- **Never rewrite history after going public** (rebase, amend, and force
  push are all prohibited)
- The only exception is **an emergency response to a leaked secret or
  similar incident** — follow a separate runbook for that
- Before going public (once, just before the v0.1.0 release), the
  private-era history will be squashed into a single initial commit.
  See
  [`maintainer-runbook/release-public.md`](../maintainer-runbook/release-public.md)

## Handling a broken main

- **Default: forward fix** — add a fix commit on top of the broken main
- Reverting the broken commit does not remove the underlying policy
  violation, so it does not actually solve the problem
- Exception: if the breakage constitutes a "feature is broken /
  production is down" situation, revert first to restore a working main,
  then open a separate PR for the permanent fix

## Branch protection (GitHub settings)

Apply the following protections to main after going public:

- Force push disabled
- Deletion disabled
- All CI jobs (`rustfmt` / `clippy` / `test` / `cargo-deny` /
  `gitleaks` / `debt-metrics` / `coverage`) must be green
- No self-approval required (single-maintainer operation)
- Admin (the maintainer) override **allowed** — enables direct push to
  main in an emergency

Apply via `gh`:

```sh
gh api repos/sat0-hir0/limn/branches/main/protection \
  --method PUT \
  --field required_status_checks[strict]=true \
  --field required_status_checks[contexts][]=rustfmt \
  --field required_status_checks[contexts][]=clippy \
  --field required_status_checks[contexts][]=test \
  --field required_status_checks[contexts][]=cargo-deny \
  --field required_status_checks[contexts][]=gitleaks \
  --field required_status_checks[contexts][]=debt-metrics \
  --field required_status_checks[contexts][]=coverage \
  --field enforce_admins=false \
  --field required_pull_request_reviews= \
  --field restrictions=
```

Apply this **after** the squash and force push that happens just before
going public (step 8 of the runbook). Enabling protection before that
step would block the force push.

## Commit author for AI-assisted work

Commits written with AI assistance (Claude, Codex, etc.) are authored
under **the maintainer's own identity (noreply email)**. Reasons:

- AI assistance is a standard assumption for this single-person OSS
  project; per-commit disclosure adds no value
- A consistent author identity makes history easier to read for
  contributors
- The use of AI tooling is disclosed for transparency in
  [`AGENTS.md`](../../AGENTS.md)

Exception: when you want to **explicitly credit a significant design
decision made in collaboration with AI**, adding
`Co-Authored-By: Claude <noreply@anthropic.com>` to the commit footer
is fine. This is optional, not the default.

## When AI opens a pull request

(Primarily relevant when using Claude Code or Codex)

1. Have the AI create the branch, commit, and push
2. **The maintainer (a human) opens the pull request** — this introduces
   a deliberate review step
3. After CI is green, the maintainer performs squash-and-merge

Do not allow the AI to open pull requests automatically, and do not
allow it to push directly to main — even for changes that would
otherwise qualify as trivial. Crossing that human boundary increases the
risk of accidents.

## Related docs

- [`feature-flags.md`](feature-flags.md) — three-stage model for incomplete features
- [`../maintainer-runbook/release-public.md`](../maintainer-runbook/release-public.md) — runbook for the v0.1.0 public release
- [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) — contributor summary
- [`../../README.md`](../../README.md) — project overview
