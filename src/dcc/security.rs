//! DCC security utilities.
//!
//! Provides defenses against common DCC attack vectors:
//! - **Path traversal**: Filenames are sanitized and resolved paths are verified
//!   to stay within the download directory.
//! - **Private IP rejection**: Optionally rejects DCC offers from private,
//!   loopback, and link-local IP addresses.
//! - **Filename collision**: Automatically appends numeric suffixes when a file
//!   already exists.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

/// Check if an IP address is private/loopback (rejected by default for DCC)
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// Sanitize a filename received via DCC to prevent path traversal attacks
pub fn sanitize_filename(filename: &str) -> Option<String> {
    // Strip path components for both Unix and Windows-style paths
    // We must handle backslash manually since on Unix it's a valid char
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    // Also apply std Path for extra safety
    let name = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    // Reject empty names
    if name.is_empty() {
        return None;
    }

    // Remove control characters and path separators
    let sanitized: String = name
        .chars()
        .filter(|c| !c.is_control() && *c != '/' && *c != '\\' && *c != ':')
        .collect();

    // Strip leading dots (hidden files / directory traversal)
    let sanitized = sanitized.trim_start_matches('.');

    if sanitized.is_empty() {
        return None;
    }

    // Limit filename length
    let truncated = if sanitized.len() > 255 {
        &sanitized[..255]
    } else {
        sanitized
    };

    Some(truncated.to_string())
}

/// Resolve the full download path, ensuring it stays within the download directory
pub fn safe_download_path(download_dir: &Path, filename: &str) -> Option<PathBuf> {
    let sanitized = sanitize_filename(filename)?;
    let path = download_dir.join(&sanitized);

    // Canonicalize the download dir to compare prefixes
    // For non-existing dirs, we just do a basic check
    let canonical_dir = download_dir
        .canonicalize()
        .unwrap_or_else(|_| download_dir.to_path_buf());
    let canonical_path = canonical_dir.join(&sanitized);

    // Verify the resolved path is within the download directory
    if !canonical_path.starts_with(&canonical_dir) {
        return None;
    }

    // If file exists, add a numeric suffix
    if path.exists() {
        let stem = Path::new(&sanitized)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = Path::new(&sanitized)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for i in 1..1000 {
            let new_name = if ext.is_empty() {
                format!("{}_{}", stem, i)
            } else {
                format!("{}_{}.{}", stem, i, ext)
            };
            let new_path = download_dir.join(&new_name);
            if !new_path.exists() {
                return Some(new_path);
            }
        }
        return None;
    }

    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello.txt"), Some("hello.txt".into()));
        assert_eq!(
            sanitize_filename("../../../etc/passwd"),
            Some("passwd".into())
        );
        assert_eq!(
            sanitize_filename("..\\..\\windows\\system32"),
            Some("system32".into())
        );
        assert_eq!(sanitize_filename(".hidden"), Some("hidden".into()));
        assert_eq!(sanitize_filename("..."), None);
        assert_eq!(sanitize_filename(""), None);
        assert_eq!(
            sanitize_filename("normal file.pdf"),
            Some("normal file.pdf".into())
        );
    }

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
    }
}
