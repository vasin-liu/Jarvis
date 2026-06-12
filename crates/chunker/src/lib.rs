use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkDraft {
    pub ord: i64,
    pub text: String,
    pub loc: String,
    pub token_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkerConfig {
    pub max_chars: usize,
    pub overlap_chars: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_chars: 800,
            overlap_chars: 80,
        }
    }
}

/// Split plain text into chunks with line-range locations (`L{start}-L{end}`).
pub fn chunk_text(text: &str, config: &ChunkerConfig) -> Vec<ChunkDraft> {
    if text.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start_line = 0usize;
    let mut buf = String::new();

    let flush = |chunks: &mut Vec<ChunkDraft>,
                 buf: &mut String,
                 start_line: &mut usize,
                 end_line: usize| {
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            return;
        }
        let ord = chunks.len() as i64;
        let loc = if *start_line == end_line {
            format!("L{}", *start_line + 1)
        } else {
            format!("L{}-L{}", *start_line + 1, end_line + 1)
        };
        chunks.push(ChunkDraft {
            ord,
            text: trimmed.to_string(),
            loc,
            token_count: trimmed.split_whitespace().count() as i64,
        });
        buf.clear();
        *start_line = end_line + 1;
    };

    for (idx, line) in lines.iter().enumerate() {
        let prospective = if buf.is_empty() {
            line.len()
        } else {
            buf.len() + 1 + line.len()
        };

        if prospective > config.max_chars && !buf.is_empty() {
            flush(&mut chunks, &mut buf, &mut start_line, idx.saturating_sub(1));
        }

        if line.len() > config.max_chars {
            if !buf.is_empty() {
                flush(&mut chunks, &mut buf, &mut start_line, idx.saturating_sub(1));
            }
            for piece in split_long_line(line, config.max_chars, config.overlap_chars) {
                let ord = chunks.len() as i64;
                chunks.push(ChunkDraft {
                    ord,
                    text: piece.clone(),
                    loc: format!("L{}", idx + 1),
                    token_count: piece.split_whitespace().count() as i64,
                });
            }
            start_line = idx + 1;
            continue;
        }

        if buf.is_empty() {
            start_line = idx;
            buf.push_str(line);
        } else {
            buf.push('\n');
            buf.push_str(line);
        }
    }

    if !buf.is_empty() {
        flush(&mut chunks, &mut buf, &mut start_line, lines.len() - 1);
    }

    chunks
}

fn split_long_line(line: &str, max_chars: usize, overlap_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    while start < line.len() {
        let end = (start + max_chars).min(line.len());
        out.push(line[start..end].to_string());
        if end == line.len() {
            break;
        }
        start = end.saturating_sub(overlap_chars).max(start + 1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_no_chunks() {
        let cfg = ChunkerConfig::default();
        assert!(chunk_text("", &cfg).is_empty());
    }

    #[test]
    fn short_text_becomes_single_chunk() {
        let cfg = ChunkerConfig {
            max_chars: 200,
            overlap_chars: 20,
        };
        let chunks = chunk_text("hello\nworld", &cfg);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "hello\nworld");
        assert_eq!(chunks[0].loc, "L1-L2");
        assert_eq!(chunks[0].ord, 0);
    }

    #[test]
    fn splits_when_exceeding_max_chars() {
        let cfg = ChunkerConfig {
            max_chars: 10,
            overlap_chars: 2,
        };
        let text = "aaaa\nbbbb\ncccc\ndddd";
        let chunks = chunk_text(text, &cfg);
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(c.text.len() <= 10);
            assert!(c.loc.starts_with('L'));
        }
    }

    #[test]
    fn long_single_line_splits_with_overlap() {
        let cfg = ChunkerConfig {
            max_chars: 8,
            overlap_chars: 2,
        };
        let chunks = chunk_text("abcdefghijklmnop", &cfg);
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].loc, "L1");
    }
}
