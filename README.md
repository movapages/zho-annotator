# Chinese Text Annotator (zho-annotator)

🇨🇳 Intelligent Chinese text annotator with pinyin and zhuyin support, featuring automatic text normalization and multiple output formats.

## Features

- **Multi-format Output**: Inline, JSON, brackets, ruby (HTML), and table formats
- **Dual Annotation Styles**: Pinyin, Zhuyin, or both
- **Automatic Text Normalization**: Built-in conversion of Kangxi radicals and character variants
- **Confidence Scoring**: Built-in confidence thresholds for quality control
- **Traditional/Simplified Support**: Automatic script detection and preference settings
- **Fast Processing**: Dictionary-based lookup with 800K+ entries
- **Cross-platform**: Pure Rust implementation
- **No AI/ML Dependencies**: Lightweight dictionary-based approach

## Installation

### From Source
```bash
git clone <repository-url>
cd zho-annotator
cargo build --release
```

### Prerequisites
You'll need the processed dictionary file (`processed_dictionary.json`) in the project root. This file contains the pronunciation data and is required for the annotator to function.

## Quick Start

```bash
# Basic annotation
./target/release/zho-annotator -t "你好世界"
# Output: 你好(nǐhǎo)世界(shìjiè)

# JSON output with confidence scores
./target/release/zho-annotator -t "你好世界" --format json --show-confidence

# Traditional Chinese with automatic normalization
./target/release/zho-annotator -t "⽅⾯問題" --traditional
# Automatically normalizes ⽅⾯ → 方面, then annotates

# Zhuyin annotation style
./target/release/zho-annotator -t "你好世界" --style zhuyin
# Output: 你好(ㄋㄧˇㄏㄠˇ)世界(ㄕˋㄐㄧㄝˋ)
```

## Usage

### Command Line Options

```bash
zho-annotator [OPTIONS]

Options:
  -t, --text <TEXT>             Chinese text to annotate
  -f, --file <FILE>             File containing Chinese text to annotate
      --stdin                   Read text from standard input
  -d, --dict <PATH>             Path to processed dictionary file [default: processed_dictionary.json]
      --format <FORMAT>         Output format: inline, json, brackets, ruby, table [default: inline]
      --style <STYLE>           Annotation style: pinyin, zhuyin, both [default: pinyin]
      --confidence <THRESHOLD>  Minimum confidence threshold (0.0-1.0) [default: 0.3]
      --show-alternatives       Show alternative pronunciations
      --show-confidence         Show confidence scores
      --traditional             Prefer traditional Chinese characters
      --examples                Show usage examples
  -h, --help                    Print help
  -V, --version                 Print version
```

### Output Formats

#### Inline (Default)
```bash
./target/release/zho-annotator -t "我爱中国"
# Output: 我(wǒ)爱(ài)中国(zhōngguó)
```

#### JSON
```bash
./target/release/zho-annotator -t "你好" --format json
# Output: {"segments": [{"text": "你好", "pinyin": "nǐ hǎo", ...}]}
```

#### Brackets
```bash
./target/release/zho-annotator -t "你好" --format brackets
# Output: 你[nǐ]好[hǎo]
```

#### Ruby (HTML)
```bash
./target/release/zho-annotator -t "你好" --format ruby
# Output: <ruby>你<rt>nǐ</rt></ruby><ruby>好<rt>hǎo</rt></ruby>
```

#### Table
```bash
./target/release/zho-annotator -t "你好" --format table
# Output: Position	Text	Pinyin	Zhuyin	Confidence	Alternatives
```

#### Rows (Two-line aligned)
```bash
./target/release/zho-annotator -t "你好世界" --format rows
# Output:
# 你好  世界
# nǐhǎo shìjiè
```

### Script Specification

**Important:** For accurate pronunciation, specify the script when dealing with ambiguous text:

#### Traditional Chinese
```bash
./target/release/zho-annotator -t "激光唱片" --traditional
# Output: 激光唱片(jīguāngchàngpiàn) - Mainland pronunciation
```

#### Auto-detection (Default)
```bash
./target/release/zho-annotator -t "激光唱片"
# Output: 激光唱片(léishèchàngpiàn) - May use Taiwan pronunciation
```

**Why this matters:**
- Some terms exist in both Traditional and Simplified forms with different pronunciations
- Auto-detection may choose Taiwan pronunciations for ambiguous text
- Use `--traditional` flag when your input is Traditional Chinese for consistent results

### Advanced Usage

#### File Processing
```bash
# Process a text file
./target/release/zho-annotator -f input.txt --format json > output.json

# Process from stdin
echo "你好世界" | ./target/release/zho-annotator --stdin --format table
```

#### Quality Control
```bash
# High confidence only
./target/release/zho-annotator -t "复杂句子" --confidence 0.8

# Show alternatives and confidence
./target/release/zho-annotator -t "复杂句子" --show-alternatives --show-confidence
```

#### Text Normalization
```bash
# Automatic normalization of Kangxi radicals and variants
./target/release/zho-annotator -t "⽅⾯問題"
# Automatically converts ⽅⾯ → 方面 for better lookup
```

## Library Usage

### Rust
```rust
use zho_annotator::production_annotator::{ProductionAnnotator, AnnotationConfig, OutputFormat, AnnotationStyle};

let config = AnnotationConfig {
    output_format: OutputFormat::Json,
    annotation_style: AnnotationStyle::Pinyin,
    confidence_threshold: 0.5,
    show_alternatives: true,
    show_confidence: true,
    use_traditional: false,
};

let annotator = ProductionAnnotator::new("processed_dictionary.json", config)?;
let segments = annotator.annotate("你好世界")?;
let output = annotator.format_output(&segments);
```

## Project Structure

```
zho-annotator/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   ├── production_annotator.rs # Main annotation logic
│   ├── dictionary.rs           # Dictionary loading and lookup
│   └── dict_processor.rs       # Dictionary processing tool
├── processed_dictionary.json   # Required: pronunciation data (800K+ entries)
├── Cargo.toml                 # Project configuration
└── README.md                  # This file
```

## Dictionary Processing

The project includes a dictionary processor to create optimized lookup files:

```bash
cargo run --bin dict-processor
```

This processes `enhanced_dictionary.json` into the optimized `processed_dictionary.json` format.

## Performance

- **Dictionary Size**: 800K+ entries
- **Processing Speed**: ~1000 characters/second
- **Memory Usage**: ~50MB for dictionary loading
- **Binary Size**: ~1MB (release build)

## License

MIT License - see LICENSE file for details.
