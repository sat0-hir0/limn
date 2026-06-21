# Limn

> フォルダ内の `.md` ファイル群を、キーボードファーストで編集する、シンプルで AI 統合されたネイティブエディタ。
>
> "Limn" は古英語で「輪郭を描く・浮かび上がらせる」の意。書き続けるうちに、思考の型がうっすら描き出されていく — そんな体験を支えるのが目標。

設計の入口は [docs/spec-handoff-gpui.md](docs/spec-handoff-gpui.md)。

## ステータス

M1 段階。`cargo run -p limn-ui` で `samples/hello.md` をブロックに parse して読み取り専用で表示するウィンドウが立ち上がる。編集は M2 で。

- [x] M0: 3 層クレート + テスト基盤 + CI
- [x] M1: gpui ウィンドウを開いて `.md` を 1 枚読み込んで表示
- [ ] M2: `.md` を 1 枚開いて編集・自動保存
- [ ] M3: `/` コマンドパレット
- [ ] M4: 「打って化ける」(Markdown 即時変換)
- [ ] M5: AI 統合 (選択→指示)
- [ ] M6: リンク・バックリンク・グラフビュー

## 前提

- Rust stable (rustup 経由)。`rust-toolchain.toml` で固定済み
- Windows / macOS / Linux で動くことを目指す (CI は当面 Ubuntu のみ。M1 で gpui 取り込み時にマトリクス化)

Rust のインストール:

```sh
# https://rustup.rs から rustup-init を入手し実行
rustc --version
cargo --version
```

## ビルド・テスト

```sh
cargo build --workspace
cargo test --workspace
cargo run -p limn-ui
```

開発前チェック (PR の CI と同じ):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 構成 (3 層)

```
crates/
├── limn-core/      Functional Core: ブロックツリー / Markdown 変換 / 補完エンジン (純粋ロジック)
├── limn-service/   Imperative Shell: .md I/O / リンク索引 / AI 呼び出し (副作用)
└── limn-ui/        gpui バインド: 描画 / 入力 / コマンドパレット
```

依存方向:

```
limn-ui ─→ limn-service ─→ limn-core
limn-ui ────────────────→ limn-core
```

逆向きの依存を入れない。`limn-core` は `std` のみで自己完結。

## コントリビュート

PR / Issue 歓迎です。詳細は [CONTRIBUTING.md](CONTRIBUTING.md)。
参加にあたっては [Code of Conduct](CODE_OF_CONDUCT.md) に同意してください。

## ライセンス

[Apache-2.0](LICENSE)
