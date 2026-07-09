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
            "PDF を開く、描画する、検索するには同梱の PDF エンジンライブラリが必要ですが、そのライブラリが見つかりませんでした。"
        }
        MessageKey::EngineUnavailableDevelopmentHelp => {
            "アプリをインストールした場合は、公式リリースアーカイブ全体をもう一度ダウンロードしてディレクトリに展開し、`bin/` を `resources/` から離して移動せずに起動してください。ソースからビルドしている場合は、PDFium のセットアップについて `docs/src/contributors/dev.md` を参照してください。"
        }
        MessageKey::EngineUnavailablePackagedHelp => {
            "このインストールは不完全なようです。公式リリースアーカイブを展開し直し、`bin/` と `resources/` を同じディレクトリに置いたままにしてください。"
        }
        MessageKey::DiagnosticDetailsLabel => "診断情報",
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
        MessageKey::PasswordPromptTitle => "パスワードが必要です",
        MessageKey::PasswordPromptBody => "この PDF はパスワードで保護されています。",
        MessageKey::PasswordPromptField => "パスワード",
        MessageKey::PasswordPromptRejected => "パスワードが受け付けられませんでした。",
        MessageKey::PasswordPromptOpen => "開く",
        MessageKey::PasswordPromptCancel => "キャンセル",
        MessageKey::RevealInFileManager => "ファイルマネージャーで表示",
        MessageKey::OutlinePanelLabel => "アウトライン",
        MessageKey::OutlineClose => "アウトラインを閉じる",
        MessageKey::OutlineLoading => "アウトラインを読み込んでいます…",
        MessageKey::OutlineEmpty => "アウトラインはありません",
        MessageKey::OutlineUnavailable => "アウトラインを利用できません",
        MessageKey::OutlineUntitled => "無題",
        MessageKey::OutlineUnsupported => "未対応のアウトライン先です",
        MessageKey::OutlineExpand => "展開",
        MessageKey::OutlineCollapse => "折りたたむ",
        MessageKey::SearchButton => "検索",
        MessageKey::SearchClear => "クリア",
        MessageKey::SearchPlaceholder => "テキストを検索…",
        MessageKey::SearchSummaryMatches => "件一致",
        MessageKey::SearchNoMatches => "一致するテキストが見つかりません。",
        MessageKey::Searching => "検索中…",
        MessageKey::SearchErrorPrefix => "エラー",
        MessageKey::SearchPagesPrefix => "ページ",
        MessageKey::SearchMatchBadgeSuffix => "件一致",
        MessageKey::CloseSearch => "検索を閉じる",
        MessageKey::ZoomClose => "閉じる (Esc)",
        MessageKey::ZoomPrevPage => "← 前",
        MessageKey::ZoomNextPage => "次 →",
        MessageKey::ZoomIn => "拡大",
        MessageKey::ZoomOut => "縮小",
        MessageKey::PageZoomView => "ページ拡大表示",
        MessageKey::PageImageAltPrefix => "ページ",
        MessageKey::ZoomPageIndicator => "ページ",
        MessageKey::ZoomScaleLabel => "ズーム",
        MessageKey::ZoomTextSelectionUnavailable => "このページではテキスト選択を利用できません。",
        MessageKey::ZoomModeEnter => "ズーム",
        MessageKey::MoreControls => "その他の操作",
        MessageKey::ColumnsLabel => "列数",
        MessageKey::ColumnsAuto => "自動",
        MessageKey::PageNumbersLabel => "ページ番号",
        MessageKey::JumpToPageLabel => "ページへ移動",
        MessageKey::JumpGoButton => "移動",
        MessageKey::ZenModeEnter => "禅",
        MessageKey::ZenModeExit => "禅モードを終了 (Esc)",
        MessageKey::ReopenDocument => "再度開く",
        MessageKey::PageCountSuffix => "ページ",
    })
}
