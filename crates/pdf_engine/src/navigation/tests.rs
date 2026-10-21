use super::*;

#[test]
fn disabled_action_type_mapping_covers_non_executable_actions() {
    assert_eq!(
        disabled_reason_for_action_type(PdfActionType::GoToDestinationInRemoteDocument),
        DisabledNavigationReason::RemoteDestination
    );
    assert_eq!(
        disabled_reason_for_action_type(PdfActionType::GoToDestinationInEmbeddedDocument),
        DisabledNavigationReason::EmbeddedDestination
    );
    assert_eq!(
        disabled_reason_for_action_type(PdfActionType::Launch),
        DisabledNavigationReason::LaunchAction
    );
    assert_eq!(
        disabled_reason_for_action_type(PdfActionType::Unsupported),
        DisabledNavigationReason::UnsupportedAction
    );
}
