# Security policy

## Reporting a vulnerability

If you believe you've found a security vulnerability in Limn, please
report it privately rather than opening a public issue.

The preferred channel is **GitHub Security Advisories**:

1. Go to <https://github.com/sat0-hir0/limn/security/advisories/new>
2. Fill in the description, affected component, and reproduction steps
3. Submit — only project maintainers will see it

If you cannot use Security Advisories, email the maintainer:
`sat0hir0.0897@gmail.com`. Please include "limn security" in the
subject line.

## What to expect

- Acknowledgement within a few days
- A coordinated fix and release once the issue is confirmed
- Credit in the release notes if you'd like (let us know how to credit you)

## Supported versions

Limn is pre-1.0; only the latest release on `main` receives security
fixes. Older tags are best-effort.

## Scope

In scope:

- The Limn binaries and library crates (`limn-core`, `limn-service`,
  `limn-ui`)
- The `debt-scan` developer tool
- Build / release infrastructure in this repository

Out of scope:

- Bugs in upstream dependencies (gpui, wgpu, etc.) — please report
  those upstream; you're welcome to also open a Limn issue so we can
  track it
- Issues that require a malicious build of Limn itself (we assume
  you trust the source you build from)
