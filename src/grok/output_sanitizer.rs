/// Stateful sanitizer for Grok-specific markup split across chunks.
#[derive(Default)]
pub struct OutputSanitizer {
    /// Nesting depth of blocks being suppressed.
    depth: usize,
    /// Incomplete tag buffered across chunk boundaries.
    buf: String,
}

impl OutputSanitizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Strip Grok/XAI markup from a streamed text chunk.
    pub fn process(&mut self, text: &str) -> String {
        let input = if self.buf.is_empty() {
            text.to_string()
        } else {
            std::mem::take(&mut self.buf) + text
        };

        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();

        while i < chars.len() {
            if chars[i] == '<' {
                let start = i;
                i += 1;
                let mut tag_content = String::new();
                let mut found_close = false;
                while i < chars.len() {
                    if chars[i] == '>' {
                        found_close = true;
                        i += 1;
                        break;
                    }
                    tag_content.push(chars[i]);
                    i += 1;
                }

                if !found_close {
                    self.buf = chars[start..].iter().collect();
                    break;
                }

                let tag_lower = tag_content.to_lowercase();
                if is_suppressed_open_tag(&tag_lower) {
                    self.depth += 1;
                } else if is_suppressed_close_tag(&tag_lower) {
                    self.depth = self.depth.saturating_sub(1);
                } else if self.depth == 0 {
                    result.push('<');
                    result.push_str(&tag_content);
                    result.push('>');
                }
            } else if self.depth == 0 {
                result.push(chars[i]);
                i += 1;
            } else {
                i += 1;
            }
        }

        result
    }
}

fn is_suppressed_open_tag(tag: &str) -> bool {
    let trimmed = tag.trim_start_matches('/');
    !tag.starts_with('/')
        && (trimmed.starts_with("xai:")
            || trimmed.starts_with("grok:")
            || trimmed.starts_with("argument"))
}

fn is_suppressed_close_tag(tag: &str) -> bool {
    tag.starts_with('/')
        && (tag.starts_with("/xai:") || tag.starts_with("/grok:") || tag.starts_with("/argument"))
}

#[cfg(test)]
mod tests {
    use super::OutputSanitizer;

    #[test]
    fn strips_grok_markup_across_chunks() {
        let mut sanitizer = OutputSanitizer::new();

        let first = sanitizer.process("Xin chao<xai:tool_usage_card");
        let second = sanitizer.process(">noise</xai:tool_usage_card> ban");

        assert_eq!(first, "Xin chao");
        assert_eq!(second, " ban");
    }

    #[test]
    fn keeps_normal_xml_like_text() {
        let mut sanitizer = OutputSanitizer::new();

        assert_eq!(
            sanitizer.process("2 < 3 va <b>ok</b>"),
            "2 < 3 va <b>ok</b>"
        );
    }
}
