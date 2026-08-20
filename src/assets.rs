use std::path::{Path, PathBuf};

/// Resolve a runtime asset path.
///
/// Installed builds keep `assets/` beside the executable. During development,
/// assets live under `data/` in the repository so they never get compiled into
/// the executable.
pub fn path(relative: impl AsRef<Path>) -> PathBuf {
    let relative = relative.as_ref();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let packaged = exe_dir.join("assets").join(relative);
            if packaged.exists() {
                return packaged;
            }
        }
    }

    let development = PathBuf::from("data").join(relative);
    if development.exists() {
        return development;
    }

    // Keep a useful error path even when the asset is missing.
    PathBuf::from("assets").join(relative)
}
