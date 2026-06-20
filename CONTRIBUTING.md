# Contributing

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

Conventional Commits に倣う:

- `feat: 新機能`
- `fix: バグ修正`
- `chore: 雑務 (ビルド/CI/依存更新)`
- `docs: ドキュメント`
- `refactor: 挙動を変えないリファクタ`
- `test: テスト追加・修正`

## 設計ドキュメント

着手前に [docs/spec-handoff-gpui.md](docs/spec-handoff-gpui.md) と [docs/testing-strategy.md](docs/testing-strategy.md) を読む。
