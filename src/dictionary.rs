use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationData {
    pub pinyin: String,
    pub zhuyin: String,
    pub traditional: String,
    pub simplified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrieNode {
    pub annotations: Vec<AnnotationData>,
    pub children: BTreeMap<char, TrieNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedData {
    pub simplified_words: std::collections::HashMap<String, Vec<AnnotationData>>,
    pub traditional_words: std::collections::HashMap<String, Vec<AnnotationData>>,
    pub char_lookup: std::collections::HashMap<String, Vec<AnnotationData>>,
    pub stats: ProcessingStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_entries: usize,
    pub unique_simplified_chars: usize,
    pub unique_traditional_chars: usize,
    pub max_word_length: usize,
    pub multi_char_entries: usize,
}

pub struct Dictionary {
    data: ProcessedData,
}

impl Dictionary {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Failed to open dictionary file: {}",
                path.as_ref().display()
            )
        })?;

        let reader = BufReader::new(file);
        let data: ProcessedData =
            serde_json::from_reader(reader).context("Failed to parse processed dictionary JSON")?;

        Ok(Dictionary { data })
    }

    pub fn entry_count(&self) -> usize {
        self.data.stats.total_entries
    }

    pub fn max_word_length(&self) -> usize {
        self.data.stats.max_word_length
    }

    /// Find the longest match starting from the given position in text
    pub fn find_longest_match(
        &self,
        text: &str,
        start_pos: usize,
        use_traditional: bool,
    ) -> Option<(usize, Vec<AnnotationData>)> {
        let chars: Vec<char> = text.chars().collect();
        if start_pos >= chars.len() {
            return None;
        }

        let words = if use_traditional {
            &self.data.traditional_words
        } else {
            &self.data.simplified_words
        };

        let mut longest_match: Option<(usize, Vec<AnnotationData>)> = None;

        // Try all possible substring lengths starting from start_pos
        for len in 1..=chars.len() - start_pos {
            let word: String = chars[start_pos..start_pos + len].iter().collect();

            if let Some(annotations) = words.get(&word) {
                // Found a match, update longest_match
                longest_match = Some((len, annotations.clone()));
            }
        }

        longest_match
    }

    /// Quick character lookup for single characters
    pub fn lookup_char(&self, ch: &str) -> Option<&Vec<AnnotationData>> {
        self.data.char_lookup.get(ch)
    }

    /// Detect if text is primarily traditional Chinese
    pub fn detect_traditional(&self, text: &str) -> bool {
        let mut traditional_count = 0;
        let mut simplified_count = 0;

        for ch in text.chars() {
            let ch_str = ch.to_string();

            // Check if character exists in traditional words
            if self.data.traditional_words.contains_key(&ch_str) {
                traditional_count += 1;
            }

            // Check if character exists in simplified words
            if self.data.simplified_words.contains_key(&ch_str) {
                simplified_count += 1;
            }
        }

        // If we have more traditional-specific characters, assume traditional
        // This is a simple heuristic - could be improved
        traditional_count > simplified_count
    }
}
