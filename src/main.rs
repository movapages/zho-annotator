use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::io::{self, Read};
use zho_annotator::production_annotator::{
    AnnotationConfig, AnnotationStyle, OutputFormat, ProductionAnnotator,
};
use zho_annotator::{Script, TextNormalizer};

fn main() -> Result<()> {
    let matches = Command::new("zho-annotator")
        .version("1.0.0")
        .author("Chinese Text Annotator")
        .about("🇨🇳 Intelligent Chinese Text Annotator with pronunciation support")
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .value_name("TEXT")
                .help("Chinese text to annotate")
                .conflicts_with("file"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("File containing Chinese text to annotate")
                .conflicts_with("text"),
        )
        .arg(
            Arg::new("stdin")
                .long("stdin")
                .help("Read text from standard input")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with_all(["text", "file"]),
        )
        .arg(
            Arg::new("dict")
                .short('d')
                .long("dict")
                .value_name("PATH")
                .default_value("processed_dictionary.json")
                .help("Path to processed dictionary file"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .default_value("inline")
                .help("Output format: inline, json, brackets, ruby, table, rows"),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .value_name("STYLE")
                .default_value("pinyin")
                .help("Annotation style: pinyin, zhuyin, both"),
        )
        .arg(
            Arg::new("confidence")
                .long("confidence")
                .value_name("THRESHOLD")
                .default_value("0.3")
                .help("Minimum confidence threshold (0.0-1.0)"),
        )
        .arg(
            Arg::new("show-alternatives")
                .long("show-alternatives")
                .help("Show alternative pronunciations")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("show-confidence")
                .long("show-confidence")
                .help("Show confidence scores")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("traditional")
                .long("traditional")
                .help("Prefer traditional Chinese characters")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("examples")
                .long("examples")
                .help("Show usage examples")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("examples") {
        show_examples();
        return Ok(());
    }

    // Parse configuration
    let config = parse_config(&matches)?;
    let dict_path = matches.get_one::<String>("dict").unwrap();

    // Initialize annotator
    println!("🚀 Initializing Chinese Text Annotator...");
    let annotator = ProductionAnnotator::new(dict_path, config.clone())?;
    let (dict_entries, model_info) = annotator.get_stats();

    println!("✅ Ready! Dictionary: {} entries", dict_entries);
    if !model_info.is_empty() {
        println!("{}", model_info);
    }

    // Get input text
    let mut input_text = get_input_text(&matches)?;

    if input_text.trim().is_empty() {
        eprintln!("❌ Error: No input text provided");
        eprintln!("Use --help for usage information");
        return Ok(());
    }

    // Apply text normalization if requested
    println!("🔧 Applying text normalization...");
    let normalizer = TextNormalizer::new();
    let target_script = if config.use_traditional {
        Some(Script::TraditionalChinese)
    } else {
        Some(Script::SimplifiedChinese)
    };
    let normalized = normalizer.normalize(&input_text, target_script);

    if !normalized.changes.is_empty() {
        println!("📝 Normalization changes:");
        for change in &normalized.changes {
            println!(
                "  {} → {} ({})",
                change.original_char, change.normalized_char, change.reason
            );
        }
        input_text = normalized.normalized;
    } else {
        println!("✅ No normalization needed");
    }

    // Annotate text
    println!("\n🔤 Processing text...");
    let segments = annotator.annotate(&input_text)?;

    // Output results
    let output = annotator.format_output(&segments);
    println!("\n📝 Annotated Result:");
    println!("{}", output);

    // Show statistics
    let chinese_segments = segments.iter().filter(|s| s.is_chinese).count();
    let total_segments = segments.len();
    let avg_confidence = if chinese_segments > 0 {
        segments
            .iter()
            .filter(|s| s.is_chinese)
            .map(|s| s.confidence)
            .sum::<f32>()
            / chinese_segments as f32
    } else {
        0.0
    };

    println!("\n📊 Statistics:");
    println!("- Total segments: {}", total_segments);
    println!("- Chinese segments: {}", chinese_segments);
    println!("- Average confidence: {:.2}", avg_confidence);

    let high_confidence = segments
        .iter()
        .filter(|s| s.is_chinese && s.confidence > 0.8)
        .count();
    let low_confidence = segments
        .iter()
        .filter(|s| s.is_chinese && s.confidence < 0.5)
        .count();

    println!("- High confidence (>0.8): {}", high_confidence);
    println!("- Low confidence (<0.5): {}", low_confidence);

    Ok(())
}

fn parse_config(matches: &ArgMatches) -> Result<AnnotationConfig> {
    let output_format = match matches.get_one::<String>("format").unwrap().as_str() {
        "inline" => OutputFormat::Inline,
        "json" => OutputFormat::Json,
        "brackets" => OutputFormat::Brackets,
        "ruby" => OutputFormat::Ruby,
        "table" => OutputFormat::Table,
        "rows" => OutputFormat::Rows,
        _ => {
            eprintln!("❌ Invalid format. Using 'inline'");
            OutputFormat::Inline
        }
    };

    let annotation_style = match matches.get_one::<String>("style").unwrap().as_str() {
        "pinyin" => AnnotationStyle::Pinyin,
        "zhuyin" => AnnotationStyle::Zhuyin,
        "both" => AnnotationStyle::Both,
        _ => {
            eprintln!("❌ Invalid style. Using 'pinyin'");
            AnnotationStyle::Pinyin
        }
    };

    let confidence_threshold: f32 = matches
        .get_one::<String>("confidence")
        .unwrap()
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("❌ Invalid confidence threshold. Using 0.3");
            0.3
        });

    Ok(AnnotationConfig {
        output_format,
        annotation_style,
        confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        show_alternatives: matches.get_flag("show-alternatives"),
        show_confidence: matches.get_flag("show-confidence"),
        use_traditional: matches.get_flag("traditional"),
    })
}

fn get_input_text(matches: &ArgMatches) -> Result<String> {
    if let Some(text) = matches.get_one::<String>("text") {
        Ok(text.clone())
    } else if let Some(file_path) = matches.get_one::<String>("file") {
        Ok(std::fs::read_to_string(file_path)?)
    } else if matches.get_flag("stdin") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        // Interactive mode
        println!("💬 Enter Chinese text to annotate (Ctrl+D to finish):");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

fn show_examples() {
    println!("🇨🇳 Chinese Text Annotator - Usage Examples");
    println!("==========================================\n");

    println!("📝 Basic Usage:");
    println!("  zho-annotator -t \"荣耀归于乌克兰\"");
    println!("  # Output: 荣耀(róngyào)归(guī)于(yú)乌克兰(wūkèlán)\n");

    println!("🔧 Automatic Text Normalization (Kangxi radicals & variants):");
    println!("  zho-annotator -t \"⽅⾯問題\" --traditional");
    println!("  # Automatically normalizes ⽅⾯ → 方面, then annotates\n");

    println!("🎯 Different Output Formats:");
    println!("  zho-annotator -t \"我爱中国\" --format json");
    println!("  zho-annotator -t \"我爱中国\" --format brackets");
    println!("  zho-annotator -t \"我爱中国\" --format ruby\n");

    println!("🔤 Annotation Styles:");
    println!("  zho-annotator -t \"我爱中国\" --style pinyin");
    println!("  zho-annotator -t \"我爱中国\" --style zhuyin");
    println!("  zho-annotator -t \"我爱中国\" --style both\n");

    println!("⚙️  Advanced Options:");
    println!("  zho-annotator -t \"我爱中国\" --show-confidence --show-alternatives");
    println!("  zho-annotator -t \"我爱中国\" --confidence 0.7");
    println!("  zho-annotator -f input.txt --format table > output.tsv\n");

    println!("📄 File Processing:");
    println!("  zho-annotator -f chinese_text.txt");
    println!("  cat chinese_text.txt | zho-annotator --stdin");
    println!("  echo \"你好世界\" | zho-annotator --stdin --format json\n");

    println!("🎨 HTML Output:");
    println!("  zho-annotator -t \"学习中文\" --format ruby > output.html\n");

    println!("📊 Analysis Mode:");
    println!("  zho-annotator -t \"复杂的句子\" --format table --show-confidence\n");

    println!("💡 Pro Tips:");
    println!("  - Use --confidence 0.8 for high-quality annotations only");
    println!("  - Use --show-alternatives to see pronunciation variants");
    println!("  - JSON format is great for programmatic processing");
    println!("  - Ruby format works well for web display");
    println!("  - Table format is perfect for analysis and debugging");
}
