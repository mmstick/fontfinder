use std::env;
use std::fs::DirBuilder;
use std::io;
use std::path::{Path, PathBuf};

/// Recursively creates directories for a given path input.
pub fn recursively_create(dir: &Path) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(dir)
}

/// Obtains the path of the local font share, based on the current user's home directory.
pub fn font_cache() -> Option<PathBuf> {
    env::home_dir().map(|path| path.join(".local/share/fonts/"))
}

/// Returns true if the supplied font variant is found within the font directory.
pub fn font_exists(base: &Path, family: &str, variant: &str, uri: &str) -> bool {
    get_font_path(base, family, variant, uri).exists()
}

/// Obtains the complete path of a given font variant.
///
/// The path is constructed based on the name of the family, the variant to write,
/// and the extension of the file that will be obtained from the URI.
pub fn get_font_path(base: &Path, family: &str, variant: &str, uri: &str) -> PathBuf {
    let extension = uri.rfind('.').map_or("", |pos| &uri[pos..]);
    base.join(&[family, "_", variant, extension].concat())
}
