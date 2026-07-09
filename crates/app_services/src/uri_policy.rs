//! External URI policy for PDF-provided navigation targets (RFC 026).
//!
//! Extracted URI strings are untrusted document metadata. This module is the
//! authoritative policy layer for deciding whether a URI may be handed to a
//! platform opener after explicit user confirmation.

use domain::navigation::NavigationResourceLimits;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalUriOpenDecision {
    Allowed { normalized_uri: String },
    Rejected(ExternalUriRejectReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalUriRejectReason {
    Empty,
    TooLong,
    RelativeOrSchemeLess,
    InvalidScheme,
    DisallowedScheme,
    ContainsControlCharacter,
    ContainsWhitespace,
    MissingAuthority,
    MissingMailtoAddress,
}

pub fn validate_external_uri_for_open(
    raw_uri: &str,
    limits: &NavigationResourceLimits,
) -> ExternalUriOpenDecision {
    if raw_uri.len() > limits.max_uri_bytes {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::TooLong);
    }

    let trimmed = raw_uri.trim_ascii();
    if trimmed.is_empty() {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::Empty);
    }
    if trimmed.len() > limits.max_uri_bytes {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::TooLong);
    }
    if trimmed.chars().any(|ch| ch.is_ascii_control()) {
        return ExternalUriOpenDecision::Rejected(
            ExternalUriRejectReason::ContainsControlCharacter,
        );
    }
    if trimmed.chars().any(|ch| ch.is_ascii_whitespace()) {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::ContainsWhitespace);
    }

    let Some(colon_index) = trimmed.find(':') else {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::RelativeOrSchemeLess);
    };
    let scheme = &trimmed[..colon_index];
    if !valid_scheme(scheme) {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::InvalidScheme);
    }

    let scheme_lower = scheme.to_ascii_lowercase();
    let rest = &trimmed[colon_index + 1..];
    match scheme_lower.as_str() {
        "http" | "https" => validate_http_like(&scheme_lower, rest),
        "mailto" => validate_mailto(rest),
        _ => ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::DisallowedScheme),
    }
}

fn validate_http_like(scheme_lower: &str, rest: &str) -> ExternalUriOpenDecision {
    let Some(after_slashes) = rest.strip_prefix("//") else {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingAuthority);
    };
    let authority_end = after_slashes
        .find(['/', '?', '#'])
        .unwrap_or(after_slashes.len());
    if authority_end == 0 {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingAuthority);
    }
    ExternalUriOpenDecision::Allowed {
        normalized_uri: format!("{scheme_lower}:{rest}"),
    }
}

fn validate_mailto(rest: &str) -> ExternalUriOpenDecision {
    if rest.is_empty() {
        return ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingMailtoAddress);
    }
    ExternalUriOpenDecision::Allowed {
        normalized_uri: format!("mailto:{rest}"),
    }
}

fn valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests;
