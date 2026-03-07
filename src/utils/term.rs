use unicode_width::UnicodeWidthStr;

/// Truncates a string to a maximum visual width, adding ".." if truncated.
pub fn truncate_visual(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;
    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + char_width + 2 > max_width {
            result.push_str("..");
            break;
        }
        result.push(c);
        current_width += char_width;
    }
    result
}

/// Formats a cell with fixed visual width and optional styling.
/// Note: Styling (colored) should be applied to the output of this if needed, 
/// but this handles the visual padding.
pub fn format_cell_raw(text: &str, width: usize) -> (String, usize) {
    let truncated = truncate_visual(text, width);
    let visual_w = truncated.width();
    (truncated, width.saturating_sub(visual_w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_visual_ascii() {
        assert_eq!(truncate_visual("hello world", 5), "hel..");
        assert_eq!(truncate_visual("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_visual_cjk() {
        // "你好世界" is 8 visual width
        assert_eq!(truncate_visual("你好世界", 5), "你..");
        assert_eq!(truncate_visual("你好", 4), "你好");
    }

    #[test]
    fn test_format_cell_raw() {
        let (text, pad) = format_cell_raw("test", 10);
        assert_eq!(text, "test");
        assert_eq!(pad, 6);
    }
}
