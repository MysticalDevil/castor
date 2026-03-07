use std::path::Path;

/// Formats the host path according to the rules:
/// - If within the provided home directory, use ~
/// - If too long, show top two parents and the leaf, using .. for the middle.
pub fn format_host(path: &Path, home: Option<&str>) -> String {
    let path_str = path.to_string_lossy();

    let display_path = if let Some(home_path) = home {
        if !home_path.is_empty() && path_str.starts_with(home_path) {
            path_str.replacen(home_path, "~", 1)
        } else {
            path_str.into_owned()
        }
    } else {
        path_str.into_owned()
    };

    let parts: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() > 3 {
        let first = parts[0];
        let second = parts[1];
        let last = parts.last().unwrap_or(&"");
        
        if first == "~" {
             format!("~/{}/../{}", second, last)
        } else {
             format!("/{}/{}/../{}", first, second, last)
        }
    } else {
        display_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_host_home() {
        let path = Path::new("/home/alice/projects/castor");
        let home = Some("/home/alice");
        assert_eq!(format_host(path, home), "~/projects/castor");
    }

    #[test]
    fn test_format_host_long_path() {
        let path = Path::new("/home/alice/work/clients/acme/project-x/module-y");
        let home = Some("/home/alice");
        // Result: ~/work/../module-y
        assert_eq!(format_host(path, home), "~/work/../module-y");
    }

    #[test]
    fn test_format_host_non_home_long() {
        let path = Path::new("/var/log/containers/pod-123/logs/app.log");
        assert_eq!(format_host(path, None), "/var/log/../app.log");
    }

    #[test]
    fn test_format_host_short() {
        let path = Path::new("/etc/hosts");
        assert_eq!(format_host(path, None), "/etc/hosts");
    }
}
