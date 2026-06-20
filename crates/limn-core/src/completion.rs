//! 補完エンジンの抽象。
//!
//! spec-handoff-gpui.md §6 に従い、「候補を作る人 (Provider)」と
//! 「いつ出すか決める人 (Policy)」を分離する。
//!
//! M0: trait のみ。実装は M3 以降。

/// 補完候補。プロバイダが返す最小単位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub label: String,
    pub insert_text: String,
}

/// 入力コンテキスト。カーソル直前の文字列など。M0 は空構造体だけ置く。
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub preceding_text: String,
}

/// 候補プロバイダ。同期・無遅延を前提とする (AI 等の非同期は別 trait で後付け予定)。
pub trait Provider {
    fn name(&self) -> &str;
    fn provide(&self, ctx: &Context) -> Vec<Candidate>;
}
