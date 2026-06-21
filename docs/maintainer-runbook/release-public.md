# Going public — pre-launch runbook

明日 (= 公開予定日) の朝にこのファイルを上から順になぞる。 1 人運用、
squash 戦略前提の手順書。 各ステップは独立して再走できる形にしてある。

## 0. 前提

- 公開時点で git history を **initial commit 1 つに squash する**
- v0.1.0 タグを切り、 GitHub Release を作る
- repo を `private` → `public` に切り替える
- 製品としては M1 まで (= read-only .md 表示)。 製品完成は問わない
- 既知の問題 (= GPL contamination) は `README.md` の "Known limitations" と
  GitHub Issue で正直に開示済み

## 1. backup branch を切る

squash は破壊操作。 必ず私的履歴を別 branch に逃がす。

```sh
cd ~/code/limn
git checkout main
git pull origin main          # 最新を取り込む
git checkout -b backup/pre-squash-private-history
git push origin backup/pre-squash-private-history
git checkout main
```

これで `backup/pre-squash-private-history` が **private** だった時代の
全 commit を保持する。 trouble 時は `git reset --hard
backup/pre-squash-private-history` で巻き戻せる。

> **判断ポイント**: backup branch を public にも残すか?
> - 残す → 私的時代の commit message / file 履歴が contributor に見える
> - 残さない (= 公開直前に backup branch を local だけにする) → 安全
> - 推奨: **公開直前に push 削除 + local だけ残す**。 リスクを下げる方向

## 2. CHANGELOG.md / リリース履歴のリセット

`release-please` が積み上げてきた CHANGELOG は私的時代の commit 単位で書かれている。 公開時点で 1 から書き直す。

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
- `cargo run -p limn-ui` opens a single window and renders
  `samples/hello.md` as parsed blocks (read-only).
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

> 公開日付 (= `MM-DD`) は実際の公開日に置き換える。

## 3. release-please の state を初期化する

private 時代の `.release-please-manifest.json` は `v0.0.X` に居る
はず。 これを `0.1.0` に固定する。

```sh
echo '{".":"0.1.0"}' > .release-please-manifest.json
```

`release-please-config.json` は触らない (= 既存設定をそのまま使う)。

## 4. history を squash する

ここが破壊操作。 backup branch (step 1) があることを確認:

```sh
git branch -a | grep backup/pre-squash-private-history
# → 表示されればOK。 されなければ step 1 に戻る
```

main を squash する 2 つの方式:

### 方式 A: `git reset --soft <initial-commit-target>` + 1 commit

すべての変更をワーキングツリーに残したまま history を切る:

```sh
# main の現状を確認
git log --oneline | head -5

# initial commit を作るために空 branch から始める
git checkout --orphan limn-public
git add -A
git commit -m "$(cat <<'EOF'
feat: initial public release (v0.1.0)

Limn is a keyboard-first, AI-integrated native editor for collections
of .md files. This v0.1.0 publishes the operational foundation:

- 3-crate workspace (limn-core, limn-service, limn-ui).
- Read-only rendering of a single .md file via gpui (cargo run -p
  limn-ui shows samples/hello.md).
- OSS governance: LICENSE (Apache-2.0), NOTICE, CODE_OF_CONDUCT,
  CONTRIBUTING, SECURITY, issue/PR templates, label policy.
- Quality gates: cargo-deny, gitleaks, release-please, debt-scan,
  lefthook hooks.

Editing (M2) and the slash-palette (M3) land on top of this base.

Known limitations are documented in README and tracked in GitHub
Issues (notably the gpui-transitive GPL contamination, awaiting
zed-industries/zed#55470).
EOF
)"

# 旧 main を捨てて新しい main を立てる
git branch -D main
git branch -m limn-public main
```

### 方式 B: そのまま `git rebase -i --root` で squash

`-i` (interactive) は AI 経由では使いにくいので **方式 A 推奨**。

## 5. CI の green を確認する

squash 後の main で全 CI が green になることを確認:

```sh
# squash 直後の commit がローカルでビルドできるか確認
cargo build --workspace
cargo test --workspace
cargo run -p debt-scan -- scan
```

ローカル green を確認したら強制 push:

```sh
git push origin main --force-with-lease
```

> `--force-with-lease` は他者の push を破壊しないオプション。 1 人運用
> でも習慣として使う。

GitHub Actions の CI が完了するまで待つ (= 5 ジョブ + labels)。

## 6. v0.1.0 タグを切る

```sh
git tag -a v0.1.0 -m "v0.1.0 — operational foundation"
git push origin v0.1.0
```

## 7. GitHub Release を作る

```sh
gh release create v0.1.0 \
    --title "v0.1.0 — operational foundation" \
    --notes-file CHANGELOG.md \
    --draft
```

`--draft` で先に作って、 中身を確認してから公開する。

```sh
gh release edit v0.1.0 --draft=false
```

## 8. repo を public にする

最後の不可逆操作。 全 CI green / README / Known limitations / SECURITY /
LICENSE / NOTICE / CHANGELOG が出揃ったことを再確認してから:

```sh
gh repo edit sat0-hir0/limn --visibility public
```

確認:

```sh
gh repo view sat0-hir0/limn --json visibility
# → {"visibility":"PUBLIC"}
```

## 9. backup branch を local だけに残す

remote から backup branch を削除して、 私的時代の履歴を公開しない:

```sh
git push origin --delete backup/pre-squash-private-history
git branch -a | grep backup
# → ローカルだけ残っていることを確認
```

## 10. 公開後の announce 動線 (= 任意)

- GitHub Discussions に "v0.1.0 公開しました" の最初の post を立てる
- Hacker News / r/rust / Zenn / Twitter 等で告知 (= ユーザー任意)

## 巻き戻し手順 (= 緊急時のみ)

何かしらの理由で公開を撤回したい場合:

```sh
# repo を private に戻す
gh repo edit sat0-hir0/limn --visibility private

# v0.1.0 タグと release を削除
gh release delete v0.1.0 --yes
git tag -d v0.1.0
git push origin :v0.1.0

# main を backup から戻す (= local backup branch が残っている前提)
git checkout main
git reset --hard backup/pre-squash-private-history
git push origin main --force-with-lease
```

## チェックリスト (= 実行時にコピーする)

- [ ] step 1: backup branch 作成 + push
- [ ] step 2: CHANGELOG.md リセット
- [ ] step 3: .release-please-manifest.json を 0.1.0 に
- [ ] step 4: history squash (= 方式 A: orphan branch)
- [ ] step 5: ローカル build / test / debt-scan green + 強制 push
- [ ] step 6: v0.1.0 タグ
- [ ] step 7: GitHub Release (= draft → 公開)
- [ ] step 8: repo を public に
- [ ] step 9: backup branch を remote から削除
- [ ] step 10: announce (= 任意)
