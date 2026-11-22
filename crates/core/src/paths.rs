use std::path::PathBuf;

const APP: &str = "ps4-studio";

/// XDG-based locations (`$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_CACHE_HOME` honoured by `dirs`).
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(APP))
}
pub fn data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(APP))
}
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join(APP))
}
