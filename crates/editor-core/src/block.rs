//! ブロックツリーの最小定義。M0 では構造のスケルトンのみ。

/// ブロックの種類。M0 は最小限。今後 Heading / List / Code / Table / Image 等を足す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
}

/// 1 つのブロック。テキスト本体 + 種類 + 子ブロック。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub text: String,
    pub children: Vec<Block>,
}

impl Block {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self {
            kind: BlockKind::Paragraph,
            text: text.into(),
            children: Vec::new(),
        }
    }
}
