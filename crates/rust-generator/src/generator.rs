use crate::llm::{LlmProvider, LlmRequest};
use crate::prompt;
use anyhow::{Context, Result};
use perl_parser::types::*;
use perl_parser::CpanMapper;
use regex::Regex;
use std::path::Path;
use tracing::info;

/// Generates Rust code from analyzed Perl projects using an LLM.
pub struct RustGenerator {
    llm: Box<dyn LlmProvider>,
    cpan_mapper: CpanMapper,
    max_tokens: u32,
    temperature: f32,
}

impl RustGenerator {
    pub fn new(llm: Box<dyn LlmProvider>, cpan_mapper: CpanMapper) -> Self {
        Self {
            llm,
            cpan_mapper,
            max_tokens: 4096,
            temperature: 0.0,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Convert an entire Perl project to Rust.
    pub async fn convert_project(&self, project: &PerlProject) -> Result<RustProject> {
        let mut files = Vec::new();
        let mut all_deps = Vec::new();

        // Build conversion context
        let cpan_mappings: Vec<(String, String)> = project
            .cpan_dependencies
            .iter()
            .filter_map(|dep| {
                self.cpan_mapper
                    .lookup(&dep.module_name)
                    .map(|equiv| (dep.module_name.clone(), equiv.crate_name))
            })
            .collect();

        let module_summaries: Vec<ModuleSummary> = project
            .modules
            .iter()
            .map(|m| ModuleSummary {
                path: m.path.clone(),
                package_name: m.package_name.clone(),
                subroutine_count: m.subroutines.len(),
                is_oop: m.is_oop,
                line_count: m.source.lines().count(),
            })
            .collect();

        let context = prompt::ConversionContext {
            cpan_mappings,
            project_modules: module_summaries,
            perl_version: project.perl_version.clone(),
        };

        // Convert each module
        for module in &project.modules {
            let rel_path = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(&module.path);
            let rust_path = perl_path_to_rust_path(rel_path);

            info!(module = %module.package_name, "Converting module");

            let rust_code = self
                .convert_file(&module.source, &rel_path.to_string_lossy(), &context)
                .await?;

            files.push(GeneratedFile {
                path: rust_path,
                content: rust_code,
            });
        }

        // Convert each script
        for script in &project.scripts {
            let rel_path = script
                .path
                .strip_prefix(&project.root)
                .unwrap_or(&script.path);
            let rust_path = perl_path_to_rust_path(rel_path);

            info!(script = %rel_path.display(), "Converting script");

            let rust_code = self
                .convert_file(&script.source, &rel_path.to_string_lossy(), &context)
                .await?;

            files.push(GeneratedFile {
                path: rust_path,
                content: rust_code,
            });
        }

        // Collect dependencies
        for dep in &project.cpan_dependencies {
            if let Some(equiv) = self.cpan_mapper.lookup(&dep.module_name) {
                if !equiv.crate_name.is_empty() && !equiv.crate_name.starts_with('#') {
                    let crate_name = equiv
                        .crate_name
                        .split_whitespace()
                        .next()
                        .unwrap_or(&equiv.crate_name);
                    if !all_deps.contains(&crate_name.to_string()) {
                        all_deps.push(crate_name.to_string());
                    }
                }
            }
        }

        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml(project, &all_deps).await?;

        Ok(RustProject {
            files,
            cargo_toml,
            dependencies: all_deps,
        })
    }

    /// Convert a single Perl file to Rust.
    async fn convert_file(
        &self,
        source: &str,
        file_path: &str,
        context: &prompt::ConversionContext,
    ) -> Result<String> {
        let system = prompt::system_prompt();
        let user_message = prompt::file_conversion_prompt(source, file_path, context);

        let request = LlmRequest {
            system_prompt: system,
            user_message,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self
            .llm
            .complete(&request)
            .await
            .context("LLM conversion failed")?;

        Ok(extract_rust_code(&response.content))
    }

    /// Request the LLM to fix compiler errors.
    pub async fn fix_errors(&self, rust_code: &str, errors: &[String]) -> Result<String> {
        let system = prompt::system_prompt();
        let user_message = prompt::fix_prompt(rust_code, errors);

        let request = LlmRequest {
            system_prompt: system,
            user_message,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self.llm.complete(&request).await?;
        Ok(extract_rust_code(&response.content))
    }

    /// Request the LLM to fix output mismatches.
    pub async fn fix_output_mismatch(
        &self,
        rust_code: &str,
        perl_output: &str,
        rust_output: &str,
    ) -> Result<String> {
        let system = prompt::system_prompt();
        let user_message = prompt::output_mismatch_prompt(rust_code, perl_output, rust_output);

        let request = LlmRequest {
            system_prompt: system,
            user_message,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self.llm.complete(&request).await?;
        Ok(extract_rust_code(&response.content))
    }

    async fn generate_cargo_toml(&self, project: &PerlProject, deps: &[String]) -> Result<String> {
        let project_name = project
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("converted-project");

        // Generate a reasonable Cargo.toml directly
        let mut toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
"#,
            project_name.replace(' ', "-").to_lowercase()
        );

        for dep in deps {
            let version = default_crate_version(dep);
            toml.push_str(&format!("{} = \"{}\"\n", dep, version));
        }

        Ok(toml)
    }
}

/// Convert a Perl file path to its Rust equivalent.
fn perl_path_to_rust_path(perl_path: &Path) -> std::path::PathBuf {
    let path_str = perl_path.to_string_lossy();

    // lib/Foo/Bar.pm → src/foo/bar.rs
    // script.pl → src/main.rs (or src/bin/script.rs)
    let path_str = path_str
        .replace("lib/", "src/")
        .replace(".pm", ".rs")
        .replace(".pl", ".rs")
        .replace("::", "/");

    // Convert CamelCase to snake_case for module files
    let path = std::path::PathBuf::from(&path_str);
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let snake = camel_to_snake(stem);
        if let Some(parent) = path.parent() {
            return parent.join(format!("{}.rs", snake));
        }
    }

    std::path::PathBuf::from(path_str)
}

/// Convert CamelCase to snake_case.
fn camel_to_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

/// Extract Rust code from LLM response (strip markdown fences).
fn extract_rust_code(response: &str) -> String {
    let fence_re = Regex::new(r"```(?:rust)?\s*\n([\s\S]*?)\n```").unwrap();

    if let Some(caps) = fence_re.captures(response) {
        return caps[1].to_string();
    }

    // If no fences, check for FILE markers
    if response.contains("// === FILE:") {
        return response.to_string();
    }

    // Return as-is (might be plain code)
    response.to_string()
}

/// Default versions for common crates.
fn default_crate_version(crate_name: &str) -> &str {
    match crate_name {
        "serde_json" => "1",
        "serde" => "1",
        "reqwest" => "0.12",
        "sqlx" => "0.8",
        "chrono" => "0.4",
        "clap" => "4",
        "tracing" => "0.1",
        "tracing-subscriber" => "0.3",
        "regex" => "1",
        "fancy-regex" | "fancy_regex" => "0.14",
        "quick-xml" | "quick_xml" => "0.37",
        "csv" => "1",
        "base64" => "0.22",
        "sha2" => "0.10",
        "md5" => "0.7",
        "walkdir" => "2",
        "tempfile" => "3",
        "anyhow" => "1",
        "thiserror" => "2",
        "tokio" => "1",
        "tera" => "1",
        "encoding_rs" => "0.8",
        "serde_yaml" => "0.9",
        "rayon" => "1",
        _ => "1",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perl_path_to_rust_path() {
        assert_eq!(
            perl_path_to_rust_path(Path::new("lib/MyModule.pm")),
            std::path::PathBuf::from("src/my_module.rs")
        );
        assert_eq!(
            perl_path_to_rust_path(Path::new("lib/Foo/BarBaz.pm")),
            std::path::PathBuf::from("src/Foo/bar_baz.rs")
        );
        assert_eq!(
            perl_path_to_rust_path(Path::new("script.pl")),
            std::path::PathBuf::from("script.rs")
        );
    }

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("MyModule"), "my_module");
        assert_eq!(camel_to_snake("HTTPClient"), "h_t_t_p_client");
        assert_eq!(camel_to_snake("simple"), "simple");
    }

    #[test]
    fn test_extract_rust_code() {
        let with_fence = "Here's the code:\n```rust\nfn main() {}\n```";
        assert_eq!(extract_rust_code(with_fence), "fn main() {}");

        let without_fence = "fn main() {}";
        assert_eq!(extract_rust_code(without_fence), "fn main() {}");
    }

    #[test]
    fn test_default_crate_versions() {
        assert_eq!(default_crate_version("serde_json"), "1");
        assert_eq!(default_crate_version("tokio"), "1");
        assert_eq!(default_crate_version("unknown-crate"), "1");
    }

    #[tokio::test]
    async fn test_generator_with_mock() {
        use crate::llm::MockLlmProvider;

        let mock = MockLlmProvider::new(vec![
            "fn greet(name: &str) -> String { format!(\"Hello, {}!\", name) }".to_string(),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2024\"".to_string(),
        ]);

        let generator = RustGenerator::new(Box::new(mock), CpanMapper::with_defaults());

        let project = PerlProject {
            root: std::path::PathBuf::from("/tmp/test"),
            modules: vec![PerlModule {
                path: std::path::PathBuf::from("/tmp/test/lib/Greeter.pm"),
                package_name: "Greeter".to_string(),
                source: "package Greeter;\nsub greet { return \"Hello, $_[1]!\" }\n1;".to_string(),
                subroutines: vec![],
                uses: vec![],
                variables: vec![],
                is_oop: false,
                parent_classes: vec![],
            }],
            scripts: vec![],
            cpan_dependencies: vec![],
            perl_version: None,
        };

        let result = generator.convert_project(&project).await.unwrap();
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].content.contains("fn greet"));
    }
}
