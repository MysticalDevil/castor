use std::path::Path;

pub fn format_host(path: &Path, home: Option<&str>) -> String {
    let path_str = path.to_string_lossy();
    if let Some(h) = home
        && path_str.starts_with(h)
    {
        return path_str.replacen(h, "~", 1);
    }
    path_str.to_string()
}

pub fn get_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata()?.len();
            } else {
                size += get_dir_size(&path)?;
            }
        }
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_host_short() {
        assert_eq!(format_host(Path::new("/etc"), None), "/etc");
    }

    #[test]
    fn test_format_host_home() {
        let home = "/home/user";
        let path = Path::new("/home/user/projects/castor");
        assert_eq!(format_host(path, Some(home)), "~/projects/castor");
    }

    #[test]
    fn test_format_host_long_path() {
        let path = Path::new("/var/log/syslog");
        assert_eq!(format_host(path, None), "/var/log/syslog");
    }

    #[test]
    fn test_format_host_non_home_long() {
        let home = "/home/user";
        let path = Path::new("/usr/local/bin");
        assert_eq!(format_host(path, Some(home)), "/usr/local/bin");
    }
}
