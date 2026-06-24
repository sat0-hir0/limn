## Summary

<!-- What changed and why. 1-3 lines. -->

## Related issues

<!-- closes #xx / refs #yy -->

## Type of Change

<!-- Check all that apply. The auto-label bot also infers from the Conventional Commits prefix; cross-check. -->
- [ ] feat (new feature)
- [ ] fix (bug fix)
- [ ] refactor / docs / chore / test / perf / ci / style
- [ ] **Breaking change** (= UI behaviour, public API, runtime contract, or CLI semantics changes)
- [ ] **Layer-crossing** (= touches more than one of `limn-core` / `limn-service` / `limn-ui`)

## Testing

<!-- Concrete repro + expected outcome. Skip if the change is doc-only.
     Screenshots / screen recordings are welcome for UI changes. -->

**Reproduce:**

```
cargo run -p limn-ui -- ./samples/hello.md
```

**Expected:**

<!-- What a reviewer (or your future self) should see / measure / log. One sentence. -->

**Untested / known gaps:**

<!-- Areas this PR does NOT cover. e.g. "symlink follow is intentionally
     left for Wave 6", "Linux build not yet verified". State `NONE` if
     coverage is complete. The pr-review bot also greps for `#[ignore]`,
     `TODO`, `FIXME` added in this diff; cross-check. -->

**Platform notes:**

<!-- Only fill if the change is platform-sensitive (= gpui main thread,
     Windows IME, HiDPI, GPU rendering, symlink handling). State `n/a`
     otherwise. -->

## Checklist (automated)

`lefthook` and CI verify these on every push. Hand-checking is **not
required** — these checkboxes exist so you can see what's enforced.

- [ ] Conventional Commits commit message
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] 3-layer dependency direction respected (`limn-core` ← `limn-service` ← `limn-ui`)
- [ ] CHANGELOG entry — leave as is; release-please manages it
