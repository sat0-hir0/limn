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
limn-ui ─→ limn-service ─→ limn-core
limn-ui ────────────────→ limn-core
```

- `limn-core`: `std` のみ。`tokio` / `gpui` / I/O 系のクレートを入れない
- `limn-service`: `limn-core` と非同期 / I/O 系のみ。`gpui` を入れない
- `limn-ui`: 全部 OK。ただし業務ロジックは書かない (= core / service に置く)

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
feat!: change the public API of `limn-core::Block`

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
  default ~150 patterns plus a handful of generic identity patterns
  (Anthropic / OpenAI org IDs, SSH key fingerprints). Run a
  full-history scan with `gitleaks detect --source . --log-opts="--all"`.

Per-tool personal areas (`.claude/`, `.cursor/`, `.codex/`, `.gemini/`)
are gitignored as developer-local.

### Local personal patterns (opt-in)

If you want gitleaks and `debt-scan` to also flag your own personally
identifying strings (developer username, real name, personal email),
opt in locally without touching committed files:

- **gitleaks**: copy the structure of `.gitleaks-local.toml` (already
  gitignored — see [`.gitignore`](.gitignore)) and adapt the patterns
  to your own identity. The shared lefthook hook auto-detects this
  file and uses it instead of `.gitleaks.toml` when present.
- **debt-scan**: export `DEBT_SCAN_PERSONAL_NAMES` as a comma-separated
  list of lowercase usernames you want to catch. With no value set the
  `personal-windows-path` count stays at zero.

  ```sh
  export DEBT_SCAN_PERSONAL_NAMES=alice
  cargo run -p debt-scan -- scan
  ```

Neither hook commits your name or email into the repo — they only
make the local checks louder for you.

## 技術的負債

未完了のままマージしたコードや、後で直すと決めたショートカットは
[`docs/debt/dashboard.md`](docs/debt/dashboard.md) に書く。
**解消条件 (どうなったら直すか) を必ず書くこと。** 書かないとただの愚痴になる。

- 機械計測値は [`debt-scan`](crates/debt-scan/) クレートが扱う。
  `cargo run -p debt-scan -- scan` で計測、`-- gate` で baseline 比較。
  CI が PR コメントに差分を出すので、増えたら理由を書くこと。
  当面は **report-only** (= 自動 fail なし) でスタート。
- 機械で数えにくいもの (= 設計判断、複数ファイル横断、上流待ち) は
  `docs/debt/dashboard.md` の Open debt 節に追記する。

**Tech debt は GitHub Issue では追わない**。 すべて
`docs/debt/dashboard.md` + `debt-scan` で完結させる方針。 GitHub Issue は
bug report と機能要望 (= contributor 向けの入口) に絞っている。

## Issue を立てるとき

GitHub Issue は **2 種類だけ**:

- `bug` — 何かが壊れている / 期待通り動かない
- `enhancement` — 新機能 / 既存機能の改善案

それ以外の質問・アイデア・雑談は
[GitHub Discussions](https://github.com/sat0-hir0/limn/discussions)
へ (公開直前まで disabled の場合あり)。

タイトルは Conventional Commits のタイプ表記に揃えると、 後で PR に
リンクするときに統一感が出ます (任意):

- `[bug] window fails to open on Wayland when GPU is absent`
- `[feat] add slash palette to limn-ui`
- `[docs] clarify M1 build instructions for Windows`

## 設計ドキュメント

着手前に [docs/spec-handoff-gpui.md](docs/spec-handoff-gpui.md) と [docs/testing-strategy.md](docs/testing-strategy.md) を読む。
