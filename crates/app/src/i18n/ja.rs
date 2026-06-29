//! Japanese catalog (RFC 017 §5). Returning `Option` allows the catalog
//! to lag behind English; lookup falls back to the reference catalog.

use super::MessageKey;

pub fn message(key: MessageKey) -> Option<&'static str> {
    Some(match key {
        MessageKey::AppTitle => "PDF タイルビューア",
        MessageKey::DashboardHeading => "PDF を開いて始めましょう",
        MessageKey::DashboardHint => "ページをタイル状に並べて、すばやく見渡せます。",
        MessageKey::OpenPdfButton => "PDF を開く…",
        MessageKey::OpeningDocument => "ドキュメントを開いています…",
        MessageKey::RecentSessionsHeading => "このセッション",
        MessageKey::RecentSessionsEmpty => "開いたドキュメントがここに表示されます。",
        MessageKey::ViewerBackToDashboard => "← 戻る",
        MessageKey::ViewerPageCountLabel => "ページ数",
        MessageKey::ViewerPageOnePreview => "1 ページ目のプレビュー",
        MessageKey::RenderingPage => "ページを描画しています…",
        MessageKey::EngineUnavailableTitle => "PDF エンジンを利用できません",
        MessageKey::EngineUnavailableBody => {
            "同梱の PDFium ライブラリを読み込めませんでした。アプリケーションの再インストールで解決することがあります。"
        }
        MessageKey::ErrFileNotFound => "ファイルが見つかりませんでした。",
        MessageKey::ErrNotAFile => "ファイルではありません。",
        MessageKey::ErrWrongExtension => "開けるのは .pdf ファイルのみです。",
        MessageKey::ErrNotAPdf => "有効な PDF ドキュメントではありません。",
        MessageKey::ErrUnreadable => "ファイルを読み取れませんでした。",
        MessageKey::ErrEncryptedUnsupported => {
            "パスワード保護された PDF はまだサポートされていません。"
        }
        MessageKey::ErrPdfParseFailed => "PDF を解析できませんでした。",
        MessageKey::ErrTooLarge => "このドキュメントはサイズ制限を超えています。",
        MessageKey::ErrUnknown => "予期しないエラーが発生しました。",
        MessageKey::ErrRenderFailed => "ページを描画できませんでした。",
    })
}
