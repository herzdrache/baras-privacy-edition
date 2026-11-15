//! Centralized application data paths.

use std::path::PathBuf;

/// Returns the configuration directory for BARAS Privacy Edition.
///
/// Windows: %APPDATA%\baras-privacy
/// Other platforms: platform-specific config directory/baras-privacy
pub fn app_config_dir() -> PathBuf {
    if let Some(path) = test_profile_dir() {
        return path;
    }

    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("baras-privacy")
}

pub(crate) fn test_profile_dir() -> Option<PathBuf> {
    std::env::var_os("BARAS_PRIVACY_TEST_PROFILE")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}
