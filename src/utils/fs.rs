use std::path::Path;

/// Formats the host path according to the rules:
/// - If within HOME, use ~
/// - If too long, show top two parents and the leaf, using .. for the middle.
pub fn format_host(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let home = std::env::var("HOME").unwrap_or_default();

    let display_path = if !home.is_empty() && path_str.starts_with(&home) {
        path_str.replacen(&home, "~", 1)
    } else {
        path_str.into_owned()
    };

    let parts: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() > 3 {
        let first = parts[0];
        let second = parts[1];
        let last = parts.last().unwrap_or(&"");
        
        // Handle the case where the first part was "~"
        if first == "~" {
             // For ~ paths, we might want to keep ~ and the next part, then .. then last
             format!("~/{}/../{}", second, last)
        } else {
             format!("/{}/{}/../{}", first, second, last)
        }
    } else {
        display_path
    }
}
