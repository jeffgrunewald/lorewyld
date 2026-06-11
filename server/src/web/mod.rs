pub mod app;
pub mod auth_ui;
pub mod home;
pub mod modules;
pub mod nav;
pub mod roll;
pub mod settings_server;
pub mod settings_users;

#[derive(Debug, Clone)]
pub struct InstanceName(pub String);

/// Cache-busting token appended to the stylesheet URL (`?v=...`).
/// Derived from the stylesheet's mtime at server startup so browsers
/// that cached an older `/assets/style.css` heuristically (ServeDir
/// sends no Cache-Control) re-fetch after any change.
#[derive(Debug, Clone)]
pub struct StyleVersion(pub String);

impl StyleVersion {
    pub fn from_asset_mtime(path: &str) -> Self {
        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs().to_string());
        Self(mtime.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()))
    }
}
