# Contributing

このプロジェクトに参加するすべての人は [Code of Conduct](CODE_OF_CONDUCT.md) に従ってください。

## 開発フロー

1. issue を立てる (or 既存の issue を pick up)
2. ブランチを切る (`feat/<topic>`, `fix/<topic>`, `chore/<topic>` 等)
3. ローカルで以下が全部 green であることを確認:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
4. PR を開く。CI が green になったらレビュー依頼

## 3 層の依存方向ルール

以下の依存方向のみを許容。逆向きは reviewer がブロックする。

```
editor-ui ─→ editor-service ─→ editor-core
editor-ui ────────────────────→ editor-core
```

- `editor-core`: `std` のみ。`tokio` / `gpui` / I/O 系のクレートを入れない
- `editor-service`: `editor-core` と非同期 / I/O 系のみ。`gpui` を入れない
- `editor-ui`: 全部 OK。ただし業務ロジックは書かない (= core / service に置く)

## コミットメッセージ

[Conventional Commits](https://www.conventionalcommits.org/) に**準拠必須** (release-please が CHANGELOG とバージョン bump を自動でやるため)。

タイプ:

| type | 用途 | CHANGELOG への露出 |
|---|---|---|
| `feat` | 新機能 | ✅ Features |
| `fix` | バグ修正 | ✅ Bug Fixes |
| `perf` | パフォーマンス改善 | ✅ Performance |
| `refactor` | 挙動を変えないリファクタ | ✅ Refactoring |
| `docs` | ドキュメント | ✅ Documentation |
| `test` | テスト追加・修正 | 非表示 |
| `ci` | CI / Actions 変更 | 非表示 |
| `chore` | 雑務 (依存更新等) | 非表示 |
| `style` | フォーマット差分のみ | 非表示 |

スコープは任意: `feat(core): add block parser` のように書ける。

### 破壊的変更

```
feat!: change the public API of `editor-core::Block`

BREAKING CHANGE: `Block::new` now requires a parent reference.
```

`!` または `BREAKING CHANGE:` フッターで release-please が major bump (1.0.0 以降のみ。0.x の間は minor 扱い)。

### CHANGELOG.md

人間が編集しないこと (release-please が PR で更新する)。

## AI tooling (optional)

This repo ships with vendor-neutral AI development scaffolding. Using any of
it (or any specific tool) is **optional** and **not required to be
disclosed**.

- [`AGENTS.md`](AGENTS.md) — onboarding context readable by Claude Code,
  Codex, Cursor, Gemini CLI, GitHub Copilot, and others.
- [`.skillshare/`](.skillshare/) — project-scoped skills managed by
  [skillshare](https://github.com/runkids/skillshare). Run
  `skillshare sync` once to install them into your AI tool of choice.
- [`lefthook.yml`](lefthook.yml) — Git hooks: `pre-commit` runs
  `gitleaks` over the staged diff; `pre-push` runs
  `cargo fmt --check / clippy -D / test` plus a second `gitleaks` sweep;
  `commit-msg` checks the Conventional Commits format. Install with
  `lefthook install`.
- [`.gitleaks.toml`](.gitleaks.toml) — Secret-scanning rules: the
  default ~150 patterns plus a handful of patterns that catch personal
  information (paths, emails). Run a full-history scan with
  `gitleaks detect --source . --log-opts="--all"`.

Per-tool personal areas (`.claude/`, `.cursor/`, `.codex/`, `.gemini/`)
are gitignored as developer-local.

## 設計ドキュメント

着手前に [docs/spec-handoff-gpui.md](docs/spec-handoff-gpui.md) と [docs/testing-strategy.md](docs/testing-strategy.md) を読む。
