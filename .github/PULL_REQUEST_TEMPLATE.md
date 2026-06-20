## Summary

<!-- 何を変えたか・なぜ変えたか。1-3 行で -->

## Related issues

<!-- closes #xx / refs #yy -->

## Checklist

- [ ] Commit message follows [Conventional Commits](https://www.conventionalcommits.org/) (`feat:` / `fix:` / `chore:` / `docs:` / `refactor:` / `test:` / `perf:` / `ci:` / `style:`)
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Respects the 3-layer dependency direction (`limn-core` ← `limn-service` ← `limn-ui`)
- [ ] CHANGELOG entry — leave as is; release-please manages it
