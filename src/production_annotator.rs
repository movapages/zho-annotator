// Production-ready Chinese text annotator with text normalization
use crate::dictionary::Dictionary;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationConfig {
    pub output_format: OutputFormat,
    pub annotation_style: AnnotationStyle,
    pub confidence_threshold: f32,
    pub show_alternatives: bool,
    pub show_confidence: bool,
    pub use_traditional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Inline,   // 我(wǒ)爱(ài)中国(zhōng guó)
    Json,     // {"segments": [{"text": "我", "pinyin": "wǒ", "confidence": 0.95}]}
    Brackets, // 我[wǒ]爱[ài]中国[zhōng guó]
    Ruby,     // <ruby>我<rt>wǒ</rt></ruby>
    Table,    // Tabular format for analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationStyle {
    Pinyin,
    Zhuyin,
    Both,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Inline,
            annotation_style: AnnotationStyle::Pinyin,
            confidence_threshold: 0.3,
            show_alternatives: false,
            show_confidence: false,
            use_traditional: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedSegment {
    pub text: String,
    pub pinyin: Option<String>,
    pub zhuyin: Option<String>,
    pub confidence: f32,
    pub alternatives: Vec<String>,
    pub is_chinese: bool,
    pub position: usize,
}

pub struct ProductionAnnotator {
    dictionary: Dictionary,
    config: AnnotationConfig,
}

impl ProductionAnnotator {
    pub fn new(dict_path: &str, config: AnnotationConfig) -> Result<Self> {
        println!("🚀 Initializing Production Chinese Annotator");

        // Load dictionary
        println!("📚 Loading dictionary from {}...", dict_path);
        let dictionary = Dictionary::from_file(dict_path)?;
        println!(
            "✅ Dictionary loaded with {} entries",
            dictionary.entry_count()
        );

        println!("🎯 Production annotator ready!");

        Ok(Self { dictionary, config })
    }

    pub fn annotate(&self, text: &str) -> Result<Vec<AnnotatedSegment>> {
        let mut segments = Vec::new();

        // Auto-detect script if not explicitly set
        let use_traditional = if self.config.use_traditional {
            true
        } else {
            // Auto-detect script
            self.dictionary.detect_traditional(text)
        };

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Try to find the longest match in dictionary
            let text_slice: String = chars[i..].iter().collect();
            let match_result = self
                .dictionary
                .find_longest_match(&text_slice, 0, use_traditional);

            if let Some((matched_len, annotation_data)) = match_result {
                // Found dictionary match - use original characters for display
                let segment_text: String = chars[i..i + matched_len].iter().collect();
                let mut best_pinyin = None;
                let mut best_zhuyin = None;
                let mut confidence = 1.0;
                let mut alternatives = Vec::new();

                // Handle empty annotations (fallback to opposite trie)
                let final_annotation_data = if annotation_data.is_empty() {
                    // Try the opposite trie for any empty annotation
                    let fallback_result =
                        self.dictionary
                            .find_longest_match(&segment_text, 0, !use_traditional);
                    if let Some((_, fallback_data)) = fallback_result {
                        if !fallback_data.is_empty() {
                            fallback_data
                        } else {
                            annotation_data
                        }
                    } else {
                        annotation_data
                    }
                } else {
                    annotation_data
                };

                if final_annotation_data.len() == 1 {
                    // Single pronunciation - high confidence
                    best_pinyin = Some(final_annotation_data[0].pinyin.clone());
                    best_zhuyin = Some(final_annotation_data[0].zhuyin.clone());
                    confidence = 0.95;
                } else if final_annotation_data.len() > 1 {
                    // Multiple pronunciations - choose the best one
                    let pinyin_options: Vec<String> = final_annotation_data
                        .iter()
                        .map(|data| data.pinyin.clone())
                        .collect();

                    alternatives = pinyin_options.clone();

                    // Choose the best pronunciation using heuristics
                    let best_index = self.select_best_pronunciation(&final_annotation_data);
                    best_pinyin = Some(final_annotation_data[best_index].pinyin.clone());
                    best_zhuyin = Some(final_annotation_data[best_index].zhuyin.clone());
                    confidence = 0.8; // Medium confidence
                }

                segments.push(AnnotatedSegment {
                    text: segment_text,
                    pinyin: best_pinyin,
                    zhuyin: best_zhuyin,
                    confidence,
                    alternatives,
                    is_chinese: true,
                    position: i,
                });

                i += matched_len;
            } else {
                // No dictionary match - use original character for display
                let ch = chars[i];
                segments.push(AnnotatedSegment {
                    text: ch.to_string(),
                    pinyin: None,
                    zhuyin: None,
                    confidence: 1.0,
                    alternatives: Vec::new(),
                    is_chinese: self.is_chinese_char(ch),
                    position: i,
                });
                i += 1;
            }
        }

        Ok(segments)
    }

    pub fn format_output(&self, segments: &[AnnotatedSegment]) -> String {
        match self.config.output_format {
            OutputFormat::Inline => self.format_inline(segments),
            OutputFormat::Json => self.format_json(segments),
            OutputFormat::Brackets => self.format_brackets(segments),
            OutputFormat::Ruby => self.format_ruby(segments),
            OutputFormat::Table => self.format_table(segments),
        }
    }

    fn format_inline(&self, segments: &[AnnotatedSegment]) -> String {
        let mut result = String::new();

        for segment in segments {
            if segment.is_chinese && segment.confidence >= self.config.confidence_threshold {
                let annotation = match self.config.annotation_style {
                    AnnotationStyle::Pinyin => segment.pinyin.as_ref(),
                    AnnotationStyle::Zhuyin => segment.zhuyin.as_ref(),
                    AnnotationStyle::Both => segment.pinyin.as_ref(), // Primary annotation
                };

                if let Some(ann) = annotation {
                    result.push_str(&segment.text);
                    result.push('(');

                    // Concatenate pinyin for multi-character words (remove spaces)
                    let concatenated_ann = if segment.text.chars().count() > 1 {
                        ann.replace(" ", "")
                    } else {
                        ann.to_string()
                    };
                    result.push_str(&concatenated_ann);

                    if matches!(self.config.annotation_style, AnnotationStyle::Both) {
                        if let Some(zhuyin) = &segment.zhuyin {
                            result.push('/');
                            result.push_str(zhuyin);
                        }
                    }

                    if self.config.show_confidence {
                        result.push_str(&format!(":{:.2}", segment.confidence));
                    }

                    if self.config.show_alternatives && segment.alternatives.len() > 1 {
                        result.push('|');
                        result.push_str(&segment.alternatives[1..].join("|"));
                    }

                    result.push(')');
                } else {
                    result.push_str(&segment.text);
                }
            } else {
                result.push_str(&segment.text);
            }
        }

        result
    }

    fn format_json(&self, segments: &[AnnotatedSegment]) -> String {
        #[derive(Serialize)]
        struct JsonOutput {
            segments: Vec<JsonSegment>,
            metadata: JsonMetadata,
        }

        #[derive(Serialize)]
        struct JsonSegment {
            text: String,
            pinyin: Option<String>,
            zhuyin: Option<String>,
            confidence: f32,
            alternatives: Vec<String>,
            is_chinese: bool,
            position: usize,
        }

        #[derive(Serialize)]
        struct JsonMetadata {
            total_segments: usize,
            chinese_segments: usize,
            average_confidence: f32,
            annotation_style: String,
        }

        let json_segments: Vec<JsonSegment> = segments
            .iter()
            .map(|seg| JsonSegment {
                text: seg.text.clone(),
                pinyin: seg.pinyin.clone(),
                zhuyin: seg.zhuyin.clone(),
                confidence: seg.confidence,
                alternatives: seg.alternatives.clone(),
                is_chinese: seg.is_chinese,
                position: seg.position,
            })
            .collect();

        let chinese_count = segments.iter().filter(|s| s.is_chinese).count();
        let avg_confidence = if chinese_count > 0 {
            segments
                .iter()
                .filter(|s| s.is_chinese)
                .map(|s| s.confidence)
                .sum::<f32>()
                / chinese_count as f32
        } else {
            0.0
        };

        let output = JsonOutput {
            segments: json_segments,
            metadata: JsonMetadata {
                total_segments: segments.len(),
                chinese_segments: chinese_count,
                average_confidence: avg_confidence,
                annotation_style: format!("{:?}", self.config.annotation_style),
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_brackets(&self, segments: &[AnnotatedSegment]) -> String {
        let mut result = String::new();

        for segment in segments {
            if segment.is_chinese && segment.confidence >= self.config.confidence_threshold {
                let annotation = match self.config.annotation_style {
                    AnnotationStyle::Pinyin => segment.pinyin.as_ref(),
                    AnnotationStyle::Zhuyin => segment.zhuyin.as_ref(),
                    AnnotationStyle::Both => segment.pinyin.as_ref(),
                };

                if let Some(ann) = annotation {
                    result.push_str(&segment.text);
                    result.push('[');
                    result.push_str(ann);
                    result.push(']');
                } else {
                    result.push_str(&segment.text);
                }
            } else {
                result.push_str(&segment.text);
            }
        }

        result
    }

    fn format_ruby(&self, segments: &[AnnotatedSegment]) -> String {
        let mut result = String::new();

        for segment in segments {
            if segment.is_chinese && segment.confidence >= self.config.confidence_threshold {
                let annotation = match self.config.annotation_style {
                    AnnotationStyle::Pinyin => segment.pinyin.as_ref(),
                    AnnotationStyle::Zhuyin => segment.zhuyin.as_ref(),
                    AnnotationStyle::Both => segment.pinyin.as_ref(),
                };

                if let Some(ann) = annotation {
                    result.push_str("<ruby>");
                    result.push_str(&segment.text);
                    result.push_str("<rt>");
                    result.push_str(ann);
                    result.push_str("</rt></ruby>");
                } else {
                    result.push_str(&segment.text);
                }
            } else {
                result.push_str(&segment.text);
            }
        }

        result
    }

    fn format_table(&self, segments: &[AnnotatedSegment]) -> String {
        let mut result = String::new();
        result.push_str("Position\tText\tPinyin\tZhuyin\tConfidence\tAlternatives\n");

        for segment in segments {
            if segment.is_chinese {
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{:.3}\t{}\n",
                    segment.position,
                    segment.text,
                    segment.pinyin.as_deref().unwrap_or("-"),
                    segment.zhuyin.as_deref().unwrap_or("-"),
                    segment.confidence,
                    segment.alternatives.join("|")
                ));
            }
        }

        result
    }

    fn is_chinese_char(&self, c: char) -> bool {
        matches!(c as u32,
            0x4E00..=0x9FFF |   // CJK Unified Ideographs
            0x3400..=0x4DBF |   // CJK Extension A
            0x20000..=0x2A6DF | // CJK Extension B
            0x2F00..=0x2FDF     // Kangxi Radicals
        )
    }

    pub fn get_stats(&self) -> (usize, String) {
        let dict_entries = self.dictionary.entry_count();
        let model_info = "Dictionary-based annotation mode".to_string();
        (dict_entries, model_info)
    }

    pub fn set_config(&mut self, config: AnnotationConfig) {
        self.config = config;
    }

    /// Select the best pronunciation from multiple variants
    fn select_best_pronunciation(
        &self,
        annotations: &[crate::dictionary::AnnotationData],
    ) -> usize {
        if annotations.is_empty() {
            return 0;
        }

        // Simple heuristics to choose the best pronunciation
        for (i, annotation) in annotations.iter().enumerate() {
            let pinyin = &annotation.pinyin;

            // Prefer standard pronunciations over rare ones
            // Common patterns that indicate standard pronunciation:

            // 1. Prefer "wǒ" over "ě" for 我
            if pinyin == "wǒ" {
                return i;
            }

            // 2. Prefer "nǐ" over "nì" for 你
            if pinyin == "nǐ" {
                return i;
            }

            // 3. Prefer "hǎo" over "hào" for 好
            if pinyin == "hǎo" {
                return i;
            }

            // 4. Prefer "shì" over "sì" for 是
            if pinyin == "shì" {
                return i;
            }

            // 5. Prefer "de" over "dí" for 的
            if pinyin == "de" {
                return i;
            }

            // 6. Prefer "le" over "liǎo" for 了
            if pinyin == "le" {
                return i;
            }
        }

        // If no specific rules match, return the first one
        0
    }
}
