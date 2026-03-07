use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use perl_parser::{CpanMapper, PerlAnalyzer};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser)]
#[command(name = "perl-to-rust")]
#[command(about = "AI-powered Perl 5 to Rust conversion tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a Perl project and output a structure report
    Analyze {
        /// Path to the Perl project directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Path to custom CPAN mappings TOML file
        #[arg(long)]
        cpan_mappings: Option<PathBuf>,

        /// Output format (json or text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Convert a Perl project to Rust
    Convert {
        /// Path to the Perl project directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output directory for the Rust project
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to custom CPAN mappings TOML file
        #[arg(long)]
        cpan_mappings: Option<PathBuf>,

        /// LLM provider to use (claude, mock)
        #[arg(long, default_value = "claude")]
        llm: String,

        /// Claude API key (or set ANTHROPIC_API_KEY env var)
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: Option<String>,

        /// Claude model to use
        #[arg(long, default_value = "claude-sonnet-4-6")]
        model: String,

        /// Run verification loop after conversion
        #[arg(long)]
        verify: bool,

        /// Maximum LLM tokens per request
        #[arg(long, default_value = "4096")]
        max_tokens: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Analyze {
            path,
            cpan_mappings,
            format,
        } => cmd_analyze(&path, cpan_mappings.as_deref(), &format),
        Commands::Convert {
            path,
            output,
            cpan_mappings,
            llm,
            api_key,
            model,
            verify,
            max_tokens,
        } => {
            cmd_convert(
                &path,
                output.as_deref(),
                cpan_mappings.as_deref(),
                &llm,
                api_key,
                &model,
                verify,
                max_tokens,
            )
            .await
        }
    }
}

fn cmd_analyze(path: &Path, cpan_mappings: Option<&Path>, format: &str) -> Result<()> {
    let analyzer = PerlAnalyzer::new();
    let project = analyzer
        .analyze_project(path)
        .context("Failed to analyze project")?;

    // Resolve CPAN mappings
    let mut mapper = CpanMapper::with_defaults();
    if let Some(mappings_path) = cpan_mappings {
        let custom = CpanMapper::from_toml(mappings_path)?;
        let custom_mappings = perl_parser::types::CpanMappings {
            modules: custom.all_mappings().clone(),
        };
        mapper.merge(&custom_mappings);
    }

    let mut project = project;
    mapper.resolve_dependencies(&mut project.cpan_dependencies);

    let report = analyzer.generate_report(&project);

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => print_text_report(&report),
    }

    Ok(())
}

fn print_text_report(report: &perl_parser::types::AnalysisReport) {
    println!("=== Perl Project Analysis ===\n");
    println!("Root: {}", report.project_root.display());
    println!("Total files: {}", report.total_files);
    println!("Total lines: {}", report.total_lines);
    println!("Regex patterns: {}", report.regex_count);
    println!("OOP modules: {}", report.oop_modules);
    println!("Complexity: {:?}", report.estimated_complexity);

    if !report.modules.is_empty() {
        println!("\n--- Modules ---");
        for m in &report.modules {
            println!(
                "  {} ({}) - {} subs, {} lines{}",
                m.package_name,
                m.path.display(),
                m.subroutine_count,
                m.line_count,
                if m.is_oop { " [OOP]" } else { "" }
            );
        }
    }

    if !report.scripts.is_empty() {
        println!("\n--- Scripts ---");
        for s in &report.scripts {
            println!(
                "  {} - {} subs, {} lines",
                s.path.display(),
                s.subroutine_count,
                s.line_count
            );
        }
    }

    if !report.cpan_dependencies.is_empty() {
        println!("\n--- CPAN Dependencies ---");
        for dep in &report.cpan_dependencies {
            let rust_equiv = dep
                .rust_equivalent
                .as_ref()
                .map(|e| format!(" → {}", e.crate_name))
                .unwrap_or_else(|| " → [no mapping]".to_string());
            println!("  {}{}", dep.module_name, rust_equiv);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_convert(
    path: &Path,
    output: Option<&Path>,
    cpan_mappings: Option<&Path>,
    llm_provider: &str,
    api_key: Option<String>,
    model: &str,
    verify: bool,
    max_tokens: u32,
) -> Result<()> {
    // Analyze the project first
    let analyzer = PerlAnalyzer::new();
    let project = analyzer
        .analyze_project(path)
        .context("Failed to analyze project")?;

    info!(
        modules = project.modules.len(),
        scripts = project.scripts.len(),
        "Project analyzed"
    );

    // Set up CPAN mapper
    let mut mapper = CpanMapper::with_defaults();
    if let Some(mappings_path) = cpan_mappings {
        let custom = CpanMapper::from_toml(mappings_path)?;
        let custom_mappings = perl_parser::types::CpanMappings {
            modules: custom.all_mappings().clone(),
        };
        mapper.merge(&custom_mappings);
    }

    // Set up LLM provider
    let llm: Box<dyn rust_generator::LlmProvider> = match llm_provider {
        "mock" => Box::new(rust_generator::MockLlmProvider::new(vec![])),
        _ => {
            let key = api_key
                .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                .context("Claude API key required. Set ANTHROPIC_API_KEY or use --api-key")?;
            Box::new(rust_generator::ClaudeProvider::new(key).with_model(model.to_string()))
        }
    };

    // Create generator
    let generator = rust_generator::RustGenerator::new(llm, mapper).with_max_tokens(max_tokens);

    // Convert
    info!("Starting conversion...");
    let rust_project = generator.convert_project(&project).await?;

    // Determine output directory
    let output_dir = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("converted");
        path.parent().unwrap_or(path).join(format!("{}-rust", name))
    });

    std::fs::create_dir_all(&output_dir)?;

    if verify {
        info!("Running verification loop...");
        let mut files = rust_project.files;
        let result = verifier::verify_and_fix(
            &generator,
            &mut files,
            &rust_project.cargo_toml,
            &output_dir,
        )
        .await?;

        if result.cargo_check_passed {
            info!(
                "Verification passed after {} fix attempts",
                result.fix_attempts
            );
        } else {
            eprintln!(
                "Verification failed after {} attempts. Errors:",
                result.fix_attempts
            );
            for error in &result.compiler_errors {
                eprintln!("  {}", error);
            }
        }
    } else {
        // Just write files without verification
        std::fs::write(output_dir.join("Cargo.toml"), &rust_project.cargo_toml)?;
        let src_dir = output_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        for file in &rust_project.files {
            let full_path = output_dir.join(&file.path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &file.content)?;
            info!(file = %file.path.display(), "Written");
        }
    }

    println!("\nConversion complete! Output: {}", output_dir.display());
    Ok(())
}
