//! Lexical Windows path operations; these do not inspect target file contents.
use crate::{Error, Result};

pub fn normalize_windows(path: &str) -> String {
    let replaced = path.replace('/', "\\");
    let without_prefix = if let Some(unc) = replaced.strip_prefix("\\\\?\\UNC\\") {
        format!("\\\\{unc}")
    } else {
        replaced
            .strip_prefix("\\\\?\\")
            .unwrap_or(&replaced)
            .to_owned()
    };
    let unc = without_prefix.starts_with("\\\\");
    let mut parts = Vec::new();
    for part in without_prefix.split('\\') {
        match part {
            "" | "." => {}
            ".." if parts
                .last()
                .is_some_and(|last: &&str| !last.ends_with(':') && *last != "..") =>
            {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("{}{}", if unc { "\\\\" } else { "" }, parts.join("\\")).to_lowercase()
}

pub fn is_absolute_windows(path: &str) -> bool {
    let bytes = path.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/'))
        || path.starts_with("\\\\")
}

pub fn is_within_windows(path: &str, parent: &str) -> bool {
    let path = normalize_windows(path);
    let parent = normalize_windows(parent);
    path == parent
        || path
            .strip_prefix(&parent)
            .is_some_and(|rest| rest.starts_with('\\'))
}

pub fn validate_absolute(path: &str) -> Result<()> {
    if !is_absolute_windows(path) || path.chars().any(|c| c == '\0' || c == '\r' || c == '\n') {
        return Err(Error::Invalid(
            "Use an absolute Windows data path without control characters".into(),
        ));
    }
    Ok(())
}

pub fn redact(text: &str, user_profile: &str) -> String {
    if user_profile.is_empty() {
        return text.to_owned();
    }
    // Match case-insensitively while slicing only at UTF-8 boundaries.
    let mut output = String::new();
    let mut remaining = text;
    while let Some((offset, end)) = remaining.char_indices().find_map(|(offset, _)| {
        let end = offset + user_profile.len();
        remaining
            .get(offset..end)
            .filter(|s| {
                s.replace('/', "\\").to_lowercase()
                    == user_profile.replace('/', "\\").to_lowercase()
            })
            .map(|_| (offset, end))
    }) {
        output.push_str(&remaining[..offset]);
        output.push_str("%USERPROFILE%");
        remaining = &remaining[end..];
    }
    output.push_str(remaining);
    output
}
