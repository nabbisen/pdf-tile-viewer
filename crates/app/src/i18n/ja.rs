//! Japanese catalog (RFC 017 §5). Returning `Option` allows the catalog
//! to lag behind English; lookup falls back to the reference catalog.

use super::MessageKey;

pub fn message(key: MessageKey) -> Option<&'static str> {
    Some(match key {
        MessageKey::AppTitle => "PDF タイルビューア",
        MessageKey::DashboardHeading => "PDF を開いて始めましょう",
        MessageKey::OpenPdfButton => "PDF を開く…",
        MessageKey::OpeningDocument => "ドキュメントを開いています…",
        MessageKey::RecentSessionsHeading => "このセッション",
        MessageKey::RecentSessionsEmpty => "開いたドキュメントがここに表示されます。",
        MessageKey::ViewerBackToDashboard => "← 戻る",
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
        MessageKey::ErrMultipleFilesDropped => "一度に開けるのは 1 つの PDF のみです。",
        MessageKey::DropZoneHint => "PDF をここにドロップするか「PDF を開く…」をクリック",
        MessageKey::RevealInFileManager => "ファイルマネージャーで表示",
        MessageKey::SearchButton => "検索",
        MessageKey::SearchClear => "クリア",
        MessageKey::SearchPlaceholder => "テキストを検索…",
        MessageKey::SearchSummaryMatches => "件一致",
        MessageKey::SearchNoMatches => "一致するテキストが見つかりません。",
        MessageKey::Searching => "検索中…",
        MessageKey::ZoomClose => "閉じる (Esc)",
        MessageKey::ZoomPrevPage => "← 前",
        MessageKey::ZoomNextPage => "次 →",
        MessageKey::ZoomPageIndicator => "ページ",
        MessageKey::ZoomScaleLabel => "ズーム",
        MessageKey::ZoomModeEnter => "ズーム",
        MessageKey::ColumnsLabel => "列数",
        MessageKey::ColumnsAuto => "自動",
        MessageKey::PageNumbersLabel => "ページ番号",
        MessageKey::JumpToPageLabel => "ページへ移動",
        MessageKey::JumpGoButton => "移動",
        MessageKey::ZenModeEnter => "禅",
        MessageKey::ZenModeExit => "禅モードを終了 (Esc)",
    })
}
