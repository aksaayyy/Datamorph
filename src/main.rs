use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, HumanDuration};
use std::io::{self, Read, Write};
use std::time::Instant;
use std::path::Path;

mod ast;
mod adapters;
mod csv_adapter;
mod error;
mod query;
mod validation;

use error::{DataMorphError, Result};
use ast::Value;
use adapters::{get_adapter_by_name, Adapter};
use validation::SchemaValidator;

const BANNER: &str = r#"
    ___    _     _             _       
   / _ \  | |   | |           (_)      
  / /_\ \ | |__ | |__    ___   _  __ _ 
 |  _  | | '_ \| '_ \  / _ \ | |/ _` |
 | | | | | |_) | |_) || (_) || | (_| |
 \_| |_/ |_.__/|_.__/  \___/ | |\__,_|
                              _/ |      
                             |__/       
"#;

#[derive(Parser)]
#[command(
    name = "datamorph",
    version = "0.1.0",
    about = "Universal data format transformer — convert, query, validate, diff & repair",
    long_about = None,
    next_line_help = true,
    arg_required_else_help = true
)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    #[arg(long, global = true)]
    no_color: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert {
        input: Option<String>,
        output: Option<String>,
        #[arg(short = 'f', long)]
        from: Option<String>,
        #[arg(short = 't', long)]
        to: String,
        #[arg(short = 'p', long)]
        pretty: bool,
        #[arg(short = 'i', long)]
        in_place: bool,
        #[arg(long)]
        verify: bool,
    },
    Query {
        input: String,
        query: String,
        #[arg(short, long)]
        format: Option<String>,
    },
    Validate {
        input: String,
        #[arg(short, long)]
        schema: String,
    },
    Repair {
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    Diff {
        file1: String,
        file2: String,
    },
    Lint {
        inputs: Vec<String>,
        #[arg(short, long)]
        fix: bool,
    },
}

fn print_banner(verbose: bool) {
    if verbose {
        println!("{}", BANNER.bright_cyan().bold());
        println!("{}", "Universal Data Format Transformer".bright_white().italic());
        println!("{}", "━".repeat(50).bright_black());
        println!();
    }
}

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message(message.to_string());
    spinner
}

fn detect_format(path: &str) -> Result<String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("json") => Ok("json".to_string()),
        Some("yaml" | "yml") => Ok("yaml".to_string()),
        Some("toml") => Ok("toml".to_string()),
        Some("csv") => Ok("csv".to_string()),
        Some("xml") => Ok("xml".to_string()),
        Some("ini") => Ok("ini".to_string()),
        _ => {
            let content = read_input(Some(path.to_string()))?;
            detect_format_from_content(&content)
        }
    }
}

fn detect_format_from_content(content: &str) -> Result<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok("json".to_string());
    }
    let first_char = trimmed.chars().next().unwrap();
    match first_char {
        '{' | '[' => Ok("json".to_string()),
        '=' => Ok("toml".to_string()),
        _ => {
            // YAML detection: contains colon and not JSON object/array start
            if trimmed.contains(':') && !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                Ok("yaml".to_string())
            } else if trimmed.contains(',') || trimmed.contains('\t') || trimmed.contains(';') {
                Ok("csv".to_string())
            } else {
                Ok("json".to_string())
            }
        }
    }
}

fn read_input(input: Option<String>) -> Result<String> {
    match input {
        Some(path) if path == "-" => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        Some(path) => {
            std::fs::read_to_string(&path).map_err(|e| DataMorphError::IoError(e))
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(output: Option<String>, content: String) -> Result<()> {
    match output {
        Some(path) if path == "-" => {
            io::stdout().write_all(content.as_bytes())?;
            Ok(())
        }
        Some(path) => {
            std::fs::write(&path, content).map_err(|e| DataMorphError::IoError(e))
        }
        None => {
            io::stdout().write_all(content.as_bytes())?;
            Ok(())
        }
    }
}

fn attempt_repair(input: &str, format: &str) -> Result<String> {
    match format {
        "json" => {
            let mut fixed = input.to_string();
            fixed = fixed.replace(",}", "}");
            fixed = fixed.replace(",]", "]");
            let open_braces = fixed.matches('{').count();
            let close_braces = fixed.matches('}').count();
            let open_brackets = fixed.matches('[').count();
            let close_brackets = fixed.matches(']').count();
            for _ in 0..(open_braces - close_braces) {
                fixed.push('}');
            }
            for _ in 0..(open_brackets - close_brackets) {
                fixed.push(']');
            }
            Ok(fixed)
        }
        _ => Ok(input.to_string()),
    }
}

fn count_fields(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.len(),
        Value::Array(arr) => arr.len(),
        _ => 1,
    }
}

fn run_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Convert { input, output, from, to, pretty: _, in_place, verify } => {
            let start = Instant::now();
            let input_str = read_input(input.clone())?;
            let from_format = if let Some(ref fmt) = from {
                fmt.to_lowercase()
            } else if let Some(path) = input.as_ref() {
                if path == "-" {
                    detect_format_from_content(&input_str)?
                } else {
                    detect_format(path)?
                }
            } else {
                detect_format_from_content(&input_str)?
            };
            let adapter = get_adapter_by_name(&from_format)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(from_format.clone()))?;
            let value = adapter.parse(&input_str)?;
            let to_format = to.to_lowercase();
            let out_adapter = get_adapter_by_name(&to_format)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(to_format.clone()))?;
            let output_str = out_adapter.serialize(&value)?;

            if *verify {
                let verify_adapter = get_adapter_by_name(&from_format)
                    .ok_or_else(|| DataMorphError::UnsupportedFormat(from_format.clone()))?;
                let roundtrip = verify_adapter.parse(&output_str)?;
                if roundtrip != value {
                    eprintln!("{} Round-trip verification failed", "⚠".yellow().bold());
                } else {
                    println!("{} Round-trip verified", "✓".green());
                }
            }

            if *in_place {
                if let Some(ref path) = input {
                    let backup_path = format!("{}.bak", path);
                    std::fs::copy(path, &backup_path)?;
                    std::fs::write(path, &output_str)?;
                    println!("{} Converted {} → {} (backup: {})",
                        "✓".green(), path.cyan(), to_format.white().bold(), backup_path.dimmed());
                }
            } else {
                write_output(output.clone(), output_str)?;
                println!("{} {} → {} ({:.2?})",
                    "✓".green(), from_format.white().bold(), to_format.white().bold(),
                    HumanDuration(start.elapsed()));
            }
            Ok(())
        }

        Commands::Query { input, query, format } => {
            let input_str = read_input(Some(input.clone()))?;
            let fmt = detect_format(input)?;
            let adapter = get_adapter_by_name(&fmt)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(fmt))?;
            let value = adapter.parse(&input_str)?;
            let expr = query::parse_upl(query)
                .map_err(|e| DataMorphError::QueryError(e.to_string()))?;
            let result = query::UplEvaluator::evaluate(&expr, &value)
                .map_err(|e| DataMorphError::QueryError(e.to_string()))?;
            let out_fmt = format.as_deref().unwrap_or("json");
            let out_adapter = get_adapter_by_name(out_fmt)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(out_fmt.to_string()))?;
            let output_str = out_adapter.serialize(&result)?;
            write_output(None, output_str)?;
            Ok(())
        }

        Commands::Validate { input, schema } => {
            // Read and parse input data
            let input_str = read_input(Some(input.clone()))?;
            let fmt = if input == "-" {
                detect_format_from_content(&input_str)?
            } else {
                detect_format(input)?
            };
            let adapter = get_adapter_by_name(&fmt)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(fmt.clone()))?;
            let value = adapter.parse(&input_str)?;

            // Load and compile JSON Schema
            let validator = SchemaValidator::from_file(schema)?;

            // Validate the data
            validator.validate_ast(&value)?;

            println!("{} {} is valid according to schema {}", "✓".green(), input.green(), schema.green());
            Ok(())
        }

        Commands::Repair { input, output } => {
            let input_str = read_input(Some(input.clone()))?;
            let fmt = detect_format(input)
                .or_else(|_| detect_format_from_content(&input_str))?;
            let adapter = get_adapter_by_name(&fmt)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(fmt.clone()))?;
            match adapter.parse(&input_str) {
                Ok(_) => {
                    println!("{} No repairs needed — valid {}", "✓".green(), fmt.bold());
                }
                Err(_) => {
                    let repaired = attempt_repair(&input_str, &fmt)?;
                    let out_path = output.as_ref().map(|s| s.as_str()).unwrap_or("-");
                    write_output(Some(out_path.to_string()), repaired)?;
                    println!("{} Repairs applied", "✓".green());
                }
            }
            Ok(())
        }

        Commands::Diff { file1, file2 } => {
            let file1_str = if file1 == "-" {
                read_input(None)?
            } else {
                std::fs::read_to_string(file1)?
            };
            let file2_str = if file2 == "-" {
                read_input(None)?
            } else {
                std::fs::read_to_string(file2)?
            };
            let fmt1 = if file1 == "-" { detect_format_from_content(&file1_str)? } else { detect_format(file1)? };
            let fmt2 = if file2 == "-" { detect_format_from_content(&file2_str)? } else { detect_format(file2)? };
            if fmt1 != fmt2 {
                return Err(DataMorphError::DiffError(
                    format!("Cannot compare different formats: {} vs {}", fmt1, fmt2)));
            }
            let adapter = get_adapter_by_name(&fmt1)
                .ok_or_else(|| DataMorphError::UnsupportedFormat(fmt1.clone()))?;
            let val1 = adapter.parse(&file1_str)?;
            let val2 = adapter.parse(&file2_str)?;
            if val1 == val2 {
                println!("{} Files are semantically identical", "✓".green());
            } else {
                println!("{} Files differ", "✗".red().bold());
                println!("  {} — {} bytes, {} fields", file1, file1_str.len(), count_fields(&val1));
                println!("  {} — {} bytes, {} fields", file2, file2_str.len(), count_fields(&val2));
            }
            Ok(())
        }

        Commands::Lint { inputs, fix } => {
            println!("{} Linting {} file(s)...", "🔍".yellow(), inputs.len());
            for input in inputs {
                match detect_format(input) {
                    Ok(format) => {
                        let content = std::fs::read_to_string(input)?;
                        let adapter = get_adapter_by_name(&format)
                            .ok_or_else(|| DataMorphError::UnsupportedFormat(format.clone()))?;
                        match adapter.parse(&content) {
                            Ok(_) => {
                                println!("  {} {} {}", "✓".green(), input.green(), format.dimmed());
                            }
                            Err(e) => {
                                println!("  {} {}: {}", "✗".red(), input.red(), e);
                                if *fix {
                                    let repaired = attempt_repair(&content, &format)?;
                                    std::fs::write(input, repaired)?;
                                    println!("    {} Fixed", "→".cyan());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("  {} {}: {}", "⚠".yellow(), input.yellow(), e);
                    }
                }
            }
            Ok(())
        }
    }
}

fn main() {
    let cli = Cli::parse();
    if cli.no_color {
        colored::control::set_override(false);
    }
    print_banner(cli.verbose);
    if let Err(err) = run_command(&cli.command) {
        eprintln!("{} {}", "Error:".red().bold(), err);
        std::process::exit(1);
    }
}