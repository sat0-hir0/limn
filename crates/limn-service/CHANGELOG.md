# Changelog

## 0.1.0 (2026-06-21)


### ⚠ BREAKING CHANGES

* limn-ui now requires the system GPU stack and the X11 / Wayland / Cocoa / DirectComposition deps that gpui drags in. CI installs the required Ubuntu packages; macOS and Windows users will need the equivalent. Crate-name path stays the same.
* the published crate names change. Anyone consuming `editor-core` / `editor-service` / `editor-ui` (none exist outside this repo yet) must update to `limn-*`.

### Features

* M1 — first vertical slice through limn-core, limn-service, limn-ui ([750e04b](https://github.com/sat0-hir0/limn/commit/750e04bc0fbe5114f8e409dbdee2a39e39d971f6))


### Refactoring

* rename project from editor to Limn ([adc57c0](https://github.com/sat0-hir0/limn/commit/adc57c0d76fa2e070d3a3d158f70767d45ef9902))
