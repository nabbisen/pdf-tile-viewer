//! Build and runtime version metadata (RFC 014 §8).
//!
//! `BuildInfo` is populated at compile-time via `env!` macros and at runtime
//! when the PDFium engine is available.

/// Runtime + build metadata surfaced in diagnostics and the about panel.
#[derive(Clone, Debug)]
pub struct BuildInfo {
    pub app_version: &'static str,
    pub app_name: &'static str,
    /// Git commit SHA, populated by `VERGEN_GIT_SHA` or similar; `None` in
    /// builds without VCS metadata injection.
    pub git_commit: Option<&'static str>,
    pub build_profile: &'static str,
    pub target_triple: &'static str,
    /// Which bundled PDFium resolution was used (RFC 003 §7 source label).
    pub pdfium_source: String,
    /// PDFium chromium tag, populated from `PDFIUM_RELEASE_TAG` env var at
    /// build time when the packaging CI sets it.
    pub pdfium_release_tag: Option<&'static str>,
}

impl BuildInfo {
    /// Construct from compile-time constants.  Call once during app startup.
    pub fn from_compile_time() -> Self {
        BuildInfo {
            app_version: env!("CARGO_PKG_VERSION"),
            app_name: env!("CARGO_PKG_NAME"),
            git_commit: option_env!("VERGEN_GIT_SHA"),
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            target_triple: env!("TARGET"),
            pdfium_source: String::new(), // filled in by app after engine boot
            pdfium_release_tag: option_env!("PDFIUM_RELEASE_TAG"),
        }
    }

    /// One-line summary for toasts or title-bar tooltip.
    pub fn short_version(&self) -> String {
        match self.git_commit {
            Some(sha) => format!("{} ({:.8})", self.app_version, sha),
            None => self.app_version.to_string(),
        }
    }
}
