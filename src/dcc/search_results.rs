use crate::app::state::SearchResultItem;
use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;

/// Check if a filename looks like a SearchBot results file.
pub fn is_search_results_file(filename: &str) -> bool {
    filename.to_lowercase().starts_with("searchbot_results_")
}

/// Parse search results text into a title and list of items.
///
/// The format is:
/// - Lines 1-5: header metadata
/// - Line 6: blank separator
/// - Lines 7+: result lines like `!BotName filename  ::INFO:: size [::HASH:: hash]`
pub fn parse_search_results(text: &str) -> (String, Vec<SearchResultItem>) {
    let lines: Vec<&str> = text.lines().collect();

    // Extract title from header (look for the search query in early lines)
    let title = extract_title_from_header(&lines);

    // Find the first blank line to skip the header
    let start = lines
        .iter()
        .position(|l| l.trim().is_empty())
        .map(|i| i + 1)
        .unwrap_or(0);

    let mut items = Vec::new();
    for line in &lines[start..] {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(item) = parse_result_line(line) {
            items.push(item);
        }
    }

    (title, items)
}

/// Extract a human-readable title from the header lines.
fn extract_title_from_header(lines: &[&str]) -> String {
    // Look for a line containing "for:" or similar pattern in the header
    for line in lines.iter().take(6) {
        let line = line.trim();
        // Common pattern: "Search results for: <query>"
        if let Some(rest) = line.strip_prefix("Search results for:") {
            let t = rest.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
        // Also check for just "for:" anywhere
        if let Some(pos) = line.to_lowercase().find("for:") {
            let t = line[pos + 4..].trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }
    // Fallback: use the first non-empty header line
    lines
        .iter()
        .take(5)
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .unwrap_or_else(|| "Search Results".to_string())
}

/// Parse a single result line into a SearchResultItem.
///
/// Format: `!BotName filename  ::INFO:: size [::HASH:: hash]`
fn parse_result_line(line: &str) -> Option<SearchResultItem> {
    if !line.starts_with('!') {
        return None;
    }

    // Split at ::INFO:: to get command part and metadata
    let (command_part, info_part) = if let Some(pos) = line.find("  ::INFO::") {
        (&line[..pos], line[pos + 10..].trim())
    } else {
        (line, "")
    };

    let command = command_part.trim().to_string();

    // Parse !BotName filename from command
    let rest = &command[1..]; // skip '!'
    let (bot, filename) = if let Some(space_pos) = rest.find(' ') {
        (
            rest[..space_pos].to_string(),
            rest[space_pos + 1..].trim().to_string(),
        )
    } else {
        (rest.to_string(), String::new())
    };

    // Extract size from info part (strip any ::HASH:: suffix)
    let size = if let Some(hash_pos) = info_part.find("::HASH::") {
        info_part[..hash_pos].trim().to_string()
    } else {
        info_part.trim().to_string()
    };

    Some(SearchResultItem {
        command,
        bot,
        filename,
        size,
    })
}

/// Extract search results from a zip file containing a .txt results file.
pub fn extract_search_results_from_zip(zip_path: &Path) -> Result<(String, Vec<SearchResultItem>)> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("Failed to open zip: {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read zip archive: {}", zip_path.display()))?;

    // Find the first .txt file in the archive
    let txt_index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index(i)
                .map(|f| f.name().to_lowercase().ends_with(".txt"))
                .unwrap_or(false)
        })
        .with_context(|| "No .txt file found in zip archive")?;

    let mut txt_file = archive.by_index(txt_index)?;
    let mut contents = String::new();
    txt_file
        .read_to_string(&mut contents)
        .with_context(|| "Failed to read .txt file from zip")?;

    Ok(parse_search_results(&contents))
}
