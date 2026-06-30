# Going public — pre-launch runbook

Work through this file from top to bottom on the morning of the planned
release date. This is a single-maintainer, squash-strategy runbook.
Each step is written to be re-runnable independently.

## 0. Preconditions

- Git history will be **squashed into a single initial commit** at the
  time of release.
- Tag v0.1.0 and create a GitHub Release.
- Switch the repo from `private` to `public`.
- The product scope at v0.1.0 is read-only `.md` rendering. Feature
  completeness is not a precondition for going public.
- Known limitations (GPL contamination) are already honestly disclosed
  in `README.md` under "Known limitations" and in a GitHub Issue.
- `git config --global user.email` is set to
  `63108893+sat0-hir0@users.noreply.github.com` (GitHub email privacy
  format). The squashed initial commit will be authored with this
  identity, ensuring your personal email address does not appear in a
  public commit. Re-verify with `git config user.email` immediately
  before squashing.

## 1. Create a backup branch

The squash is a destructive operation. Save the private-era history to a
separate branch first.

```sh
cd ~/code/limn
git checkout main
git pull origin main          # pull the latest
git checkout -b backup/pre-squash-private-history
git push origin backup/pre-squash-private-history
git checkout main
```

`backup/pre-squash-private-history` now holds every commit from the
private era. If anything goes wrong, run
`git reset --hard backup/pre-squash-private-history` to recover.

> **Decision point**: should the backup branch stay public?
> - Leave it public → private-era commit messages and file history are
>   visible to contributors.
> - Remove it from the remote (keep only locally) → safer.
> - Recommendation: **delete the remote backup branch just before going
>   public; keep it locally only.** This minimizes exposure.

## 2. Reset CHANGELOG.md

The `CHANGELOG.md` accumulated by `release-please` reflects the
private-era commit history. Rewrite it from scratch for the public
release.

```sh
cat > CHANGELOG.md <<'EOF'
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-MM-DD

### Added

- Public release of the operational foundation.
- 3-crate workspace: `limn-core`, `limn-service`, `limn-ui`.
- `cargo run -p limn-ui` opens a single window and renders the
  embedded Welcome document (or a `.md` passed as a CLI argument) as
  parsed blocks (read-only).
- OSS governance scaffolding: `LICENSE` (Apache-2.0), `NOTICE`,
  `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, issue / PR
  templates, label policy.
- Quality gates: `cargo-deny`, `gitleaks`, `release-please`,
  `debt-scan`, lefthook hooks (pre-commit / pre-push / commit-msg).

### Known limitations

- License contamination via `gpui`'s transitive `sum_tree` →
  `ztracing` → `zlog` (GPL-3.0-or-later). Tracked in the project's
  GitHub Issues. Waiting on the upstream issue
  [zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470).

[0.1.0]: https://github.com/sat0-hir0/limn/releases/tag/v0.1.0
EOF
```

> Replace `MM-DD` with the actual release date.

## 3. Reset release-please state

The `.release-please-manifest.json` from the private era is probably at
`v0.0.X`. Pin it to `0.1.0`.

```sh
echo '{".":"0.1.0"}' > .release-please-manifest.json
```

Do not touch `release-please-config.json` — keep the existing
configuration as-is.

## 4. Squash history

This is the destructive step. Verify the backup branch from step 1
exists before proceeding.

```sh
git branch -a | grep backup/pre-squash-private-history
# → If the branch appears, you are clear. If not, return to step 1.
```

Two approaches for squashing main:

### Approach A: orphan branch (recommended)

Create a fresh history from an orphan branch and replace main:

```sh
# Review the current state of main
git log --oneline | head -5

# Start an orphan branch and commit everything as a single commit
git checkout --orphan limn-public
git add -A
git commit -m "$(cat <<'EOF'
feat: initial public release (v0.1.0)

Limn is a keyboard-first, AI-integrated native editor for collections
of .md files. This v0.1.0 publishes the operational foundation:

- 3-crate workspace (limn-core, limn-service, limn-ui).
- Read-only rendering of a single .md file via gpui (cargo run -p
  limn-ui shows an embedded Welcome doc by default; pass a path to
  open any .md).
- OSS governance: LICENSE (Apache-2.0), NOTICE, CODE_OF_CONDUCT,
  CONTRIBUTING, SECURITY, issue/PR templates, label policy.
- Quality gates: cargo-deny, gitleaks, release-please, debt-scan,
  lefthook hooks.

Known limitations are documented in README and tracked in GitHub
Issues (notably the gpui-transitive GPL contamination, awaiting
zed-industries/zed#55470).
EOF
)"

# Drop the old main and promote the orphan
git branch -D main
git branch -m limn-public main
```

### Approach B: `git rebase -i --root`

Interactive rebase is difficult to use through AI tooling.
**Approach A is recommended.**

## 5. Verify CI is green and force-push

Confirm the squashed main builds cleanly locally:

```sh
cargo build --workspace
cargo test --workspace
cargo run -p debt-scan -- scan
```

Once local checks pass, force-push:

```sh
git push origin main --force-with-lease
```

> `--force-with-lease` prevents overwriting pushes from other people.
> Use it as a habit even in a single-maintainer repo.

Wait for GitHub Actions to complete all CI jobs (5 jobs + labels).

## 6. Tag v0.1.0

```sh
git tag -a v0.1.0 -m "v0.1.0 — operational foundation"
git push origin v0.1.0
```

## 7. Create the GitHub Release

```sh
gh release create v0.1.0 \
    --title "v0.1.0 — operational foundation" \
    --notes-file CHANGELOG.md \
    --draft
```

Create as a draft first. Review the content, then publish:

```sh
gh release edit v0.1.0 --draft=false
```

## 8. Make the repo public

This is the final irreversible action. Confirm that all CI jobs are
green and that `README`, "Known limitations", `SECURITY`, `LICENSE`,
`NOTICE`, and `CHANGELOG` are all in order before proceeding.

```sh
gh repo edit sat0-hir0/limn --visibility public
```

Verify:

```sh
gh repo view sat0-hir0/limn --json visibility
# → {"visibility":"PUBLIC"}
```

## 9. Remove the backup branch from the remote

Delete the private-era history from the remote so it is not publicly
visible:

```sh
git push origin --delete backup/pre-squash-private-history
git branch -a | grep backup
# → Confirm only the local branch remains
```

## 10. Enable branch protection (within 5 minutes of going public)

Apply main branch protection as described in the
[git strategy doc](../development/git-strategy.md). This step is done
**after** going public so that the squash and force push in step 5 are
not blocked.

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

Protection settings applied:
- Force push disabled; deletion disabled
- All CI jobs (`rustfmt` / `clippy` / `test` / `cargo-deny` /
  `gitleaks` / `debt-metrics` / `coverage`) must be green
- No self-approval required (single-maintainer operation)
- Admin (the maintainer) override allowed — enables direct push to main
  in an emergency

Verify:

```sh
gh api repos/sat0-hir0/limn/branches/main/protection \
  --jq '{required_status_checks, enforce_admins, required_pull_request_reviews}'
```

## 11. Connect Codecov (within 5 minutes of going public)

`.github/workflows/coverage.yml` reads `${{ secrets.CODECOV_TOKEN }}`.
Register the repo with Codecov and add the token to Secrets after making
the repo public.

```sh
# 1) Enable the repo on the Codecov side
open https://codecov.io/gh/sat0-hir0/limn   # Sign up / Add new repository
# Copy the repository upload token

# 2) Add it as a GitHub Actions secret
gh secret set CODECOV_TOKEN --repo sat0-hir0/limn
# Paste the token when prompted

# 3) Confirm coverage reaches Codecov on the next push or pull request
```

`fail_ci_if_error: false` is set, so CI will not fail if the token is
missing — but the badge and pull request comment will not appear until
it is configured.

## 12. Announce (optional)

- Open the first post in GitHub Discussions: "v0.1.0 is now public."
- Announce on Hacker News / r/rust / Zenn / Twitter or equivalent
  (entirely at the maintainer's discretion).

## Rollback procedure (emergency only)

If you need to retract the public release for any reason:

```sh
# Make the repo private again
gh repo edit sat0-hir0/limn --visibility private

# Delete the v0.1.0 tag and release
gh release delete v0.1.0 --yes
git tag -d v0.1.0
git push origin :v0.1.0

# Restore main from the local backup branch
git checkout main
git reset --hard backup/pre-squash-private-history
git push origin main --force-with-lease
```

## Checklist (copy this when executing)

- [ ] Step 1: create and push backup branch
- [ ] Step 2: reset CHANGELOG.md
- [ ] Step 3: set `.release-please-manifest.json` to 0.1.0
- [ ] Step 4: squash history (approach A: orphan branch)
- [ ] Step 5: local build / test / debt-scan green + force-push
- [ ] Step 6: tag v0.1.0
- [ ] Step 7: GitHub Release (draft → publish)
- [ ] Step 8: make repo public
- [ ] Step 9: delete backup branch from remote
- [ ] Step 10: enable branch protection
- [ ] Step 11: connect Codecov + add `CODECOV_TOKEN` to Secrets
- [ ] Step 12: announce (optional)
