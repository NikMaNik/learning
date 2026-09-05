/// Parse markdown content into (english, russian) pairs.
///
/// Supported formats:
///   - `- word — слово` (dash or hyphen separator)
///   - `| english | russian |` table rows
pub fn parse_markdown(content: &str) -> Vec<(String, String)> {
    let mut words = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Format: `- english — russian` or `- english - russian`
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if let Some((en, ru)) = split_separator(rest) {
                let en = en.trim().to_string();
                let ru = ru.trim().to_string();
                if !en.is_empty() && !ru.is_empty() {
                    words.push((en, ru));
                }
            }
        }
        // Format: table row `| english | russian |`
        else if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
            if cells.len() >= 2 {
                let en = cells[0].to_string();
                let ru = cells[1].to_string();
                // Skip header row and separator row
                if en.to_lowercase() != "english"
                    && en.to_lowercase() != "en"
                    && !en.chars().all(|c| c == '-' || c == ' ')
                    && !en.is_empty()
                {
                    words.push((en, ru));
                }
            }
        }
        // Format: `english — russian` without dash prefix
        else if let Some((en, ru)) = split_separator(trimmed) {
            let en = en.trim().to_string();
            let ru = ru.trim().to_string();
            if !en.is_empty() && !ru.is_empty() {
                words.push((en, ru));
            }
        }
    }

    words
}

fn split_separator(s: &str) -> Option<(&str, &str)> {
    if let Some(pos) = s.find('\u{2014}') {
        Some((&s[..pos], &s[pos + 3..]))
    } else if let Some(pos) = s.find(" - ") {
        Some((&s[..pos], &s[pos + 3..]))
    } else {
        None
    }
}

pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_format() {
        let md = "- hello \u{2014} привет\n- world \u{2014} мир";
        let words = parse_markdown(md);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], ("hello".to_string(), "привет".to_string()));
        assert_eq!(words[1], ("world".to_string(), "мир".to_string()));
    }

    #[test]
    fn test_table_format() {
        let md = "| english | russian |\n|---------|----------|\n| hello | привет |";
        let words = parse_markdown(md);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], ("hello".to_string(), "привет".to_string()));
    }

    #[test]
    fn test_plain_format() {
        let md = "apple \u{2014} яблоко";
        let words = parse_markdown(md);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], ("apple".to_string(), "яблоко".to_string()));
    }

    #[test]
    fn test_empty_lines() {
        let md = "\n\n- a \u{2014} b\n\n";
        let words = parse_markdown(md);
        assert_eq!(words.len(), 1);
    }
}
