use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Deserialize)]
struct DictionaryEntry {
    sm: String,     // Simplified
    tr: String,     // Traditional
    pinyin: String, // Pinyin
    zhuyin: String, // Zhuyin/Bopomofo
}

#[derive(Debug, Clone, Serialize)]
struct AnnotationData {
    pinyin: String,
    zhuyin: String,
    traditional: String,
    simplified: String,
}

#[derive(Debug, Serialize)]
struct ProcessedData {
    // Flat word-to-annotations mapping for both scripts
    simplified_words: HashMap<String, Vec<AnnotationData>>,
    traditional_words: HashMap<String, Vec<AnnotationData>>,

    // Quick character-to-annotation lookup
    char_lookup: HashMap<String, Vec<AnnotationData>>,

    // Statistics
    stats: ProcessingStats,
}

#[derive(Debug, Serialize)]
struct ProcessingStats {
    total_entries: usize,
    unique_simplified_chars: usize,
    unique_traditional_chars: usize,
    max_word_length: usize,
    multi_char_entries: usize,
}

fn main() -> Result<()> {
    println!("Dictionary Processor - Creating optimized mapping files");
    println!("Loading enhanced_dictionary.json...");

    let dict_file = File::open("enhanced_dictionary.json")
        .context("Failed to open enhanced_dictionary.json")?;
    let reader = BufReader::new(dict_file);

    let entries: Vec<DictionaryEntry> =
        serde_json::from_reader(reader).context("Failed to parse JSON")?;

    println!("Loaded {} entries", entries.len());
    println!("Processing entries and building tries...");

    let processed = process_dictionary(entries)?;

    println!("Writing optimized mapping files...");

    // Write the main processed data
    let output_file = File::create("processed_dictionary.json")
        .context("Failed to create processed_dictionary.json")?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer(writer, &processed).context("Failed to write processed dictionary")?;

    // Print statistics
    println!("\nProcessing Complete!");
    println!("Statistics:");
    println!("  Total entries: {}", processed.stats.total_entries);
    println!(
        "  Unique simplified characters: {}",
        processed.stats.unique_simplified_chars
    );
    println!(
        "  Unique traditional characters: {}",
        processed.stats.unique_traditional_chars
    );
    println!("  Maximum word length: {}", processed.stats.max_word_length);
    println!(
        "  Multi-character entries: {}",
        processed.stats.multi_char_entries
    );
    println!("\nGenerated files:");
    println!("  - processed_dictionary.json (main lookup data)");

    Ok(())
}

fn process_dictionary(entries: Vec<DictionaryEntry>) -> Result<ProcessedData> {
    let mut simplified_words: HashMap<String, Vec<AnnotationData>> = HashMap::new();
    let mut traditional_words: HashMap<String, Vec<AnnotationData>> = HashMap::new();
    let mut char_lookup: HashMap<String, Vec<AnnotationData>> = HashMap::new();

    let mut unique_simplified = std::collections::HashSet::new();
    let mut unique_traditional = std::collections::HashSet::new();
    let mut max_word_length = 0;
    let mut multi_char_count = 0;

    for entry in &entries {
        let annotation = AnnotationData {
            pinyin: entry.pinyin.clone(),
            zhuyin: entry.zhuyin.clone(),
            traditional: entry.tr.clone(),
            simplified: entry.sm.clone(),
        };

        // Track statistics
        max_word_length = max_word_length.max(entry.sm.chars().count());
        max_word_length = max_word_length.max(entry.tr.chars().count());

        if entry.sm.chars().count() > 1 || entry.tr.chars().count() > 1 {
            multi_char_count += 1;
        }

        // Add to simplified words
        simplified_words
            .entry(entry.sm.clone())
            .or_insert_with(Vec::new)
            .push(annotation.clone());
        unique_simplified.insert(entry.sm.clone());

        // Add to traditional words (always, even if same as simplified)
        traditional_words
            .entry(entry.tr.clone())
            .or_insert_with(Vec::new)
            .push(annotation.clone());
        unique_traditional.insert(entry.tr.clone());

        // Add to character lookup for both simplified and traditional
        char_lookup
            .entry(entry.sm.clone())
            .or_insert_with(Vec::new)
            .push(annotation.clone());

        if entry.sm != entry.tr {
            char_lookup
                .entry(entry.tr.clone())
                .or_insert_with(Vec::new)
                .push(annotation);
        }
    }

    let stats = ProcessingStats {
        total_entries: entries.len(),
        unique_simplified_chars: unique_simplified.len(),
        unique_traditional_chars: unique_traditional.len(),
        max_word_length,
        multi_char_entries: multi_char_count,
    };

    Ok(ProcessedData {
        simplified_words,
        traditional_words,
        char_lookup,
        stats,
    })
}
