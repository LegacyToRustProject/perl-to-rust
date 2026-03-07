use anyhow::Result;
use perl_parser::types::{GeneratedFile, VerificationResult};
use rust_generator::RustGenerator;
use std::path::Path;
use tracing::{info, warn};

use crate::compiler;

/// Maximum number of fix attempts before giving up.
const MAX_FIX_ATTEMPTS: usize = 5;

/// Run the verification and fix loop for generated Rust code.
///
/// 1. Write generated code to a temp project
/// 2. Run `cargo check`
/// 3. If errors, ask LLM to fix
/// 4. Repeat until success or max attempts
pub async fn verify_and_fix(
    generator: &RustGenerator,
    files: &mut [GeneratedFile],
    cargo_toml: &str,
    project_dir: &Path,
) -> Result<VerificationResult> {
    // Write initial files
    write_project(project_dir, files, cargo_toml)?;

    let mut fix_attempts = 0;

    loop {
        // Run cargo check
        let check_result = compiler::cargo_check(project_dir)?;

        if check_result.success {
            info!(attempts = fix_attempts, "Verification passed");
            return Ok(VerificationResult {
                cargo_check_passed: true,
                compiler_errors: vec![],
                output_match: None,
                fix_attempts,
            });
        }

        fix_attempts += 1;
        if fix_attempts > MAX_FIX_ATTEMPTS {
            warn!("Max fix attempts ({}) reached", MAX_FIX_ATTEMPTS);
            return Ok(VerificationResult {
                cargo_check_passed: false,
                compiler_errors: check_result.errors,
                output_match: None,
                fix_attempts,
            });
        }

        info!(
            attempt = fix_attempts,
            errors = check_result.errors.len(),
            "Attempting to fix compiler errors"
        );

        // Fix each file that has errors
        for file in files.iter_mut() {
            let file_errors: Vec<String> = check_result
                .errors
                .iter()
                .filter(|e| {
                    e.contains(&file.path.to_string_lossy().to_string()) || e.contains("error[E")
                })
                .cloned()
                .collect();

            if !file_errors.is_empty() {
                let fixed = generator.fix_errors(&file.content, &file_errors).await?;
                file.content = fixed;
            }
        }

        // Re-write fixed files
        write_project(project_dir, files, cargo_toml)?;
    }
}

/// Run the full verification loop including output comparison.
pub async fn verify_with_output_comparison(
    generator: &RustGenerator,
    files: &mut [GeneratedFile],
    cargo_toml: &str,
    project_dir: &Path,
    perl_script: &Path,
    args: &[&str],
) -> Result<VerificationResult> {
    // First, get cargo check passing
    let mut result = verify_and_fix(generator, files, cargo_toml, project_dir).await?;

    if !result.cargo_check_passed {
        return Ok(result);
    }

    // Then compare outputs
    if compiler::perl_available() {
        match crate::comparator::compare_outputs(perl_script, project_dir, args) {
            Ok(comparison) => {
                if comparison.matches {
                    info!("Output matches!");
                    result.output_match = Some(comparison);
                } else {
                    warn!("Output mismatch detected");

                    // Try to fix output mismatch
                    let mut mismatch_attempts = 0;
                    let max_mismatch_attempts = 3;
                    let mut current_comparison = comparison;

                    while !current_comparison.matches && mismatch_attempts < max_mismatch_attempts {
                        mismatch_attempts += 1;
                        result.fix_attempts += 1;

                        // Fix the main file (first file or main.rs)
                        if let Some(main_file) = files.iter_mut().find(|f| {
                            f.path.to_string_lossy().contains("main")
                                || f.path.to_string_lossy().ends_with(".rs")
                        }) {
                            let fixed = generator
                                .fix_output_mismatch(
                                    &main_file.content,
                                    &current_comparison.perl_output,
                                    &current_comparison.rust_output,
                                )
                                .await?;
                            main_file.content = fixed;
                        }

                        write_project(project_dir, files, cargo_toml)?;

                        // Re-check compilation
                        let check = compiler::cargo_check(project_dir)?;
                        if !check.success {
                            // Need to fix compilation again
                            let sub_result =
                                verify_and_fix(generator, files, cargo_toml, project_dir).await?;
                            result.fix_attempts += sub_result.fix_attempts;
                            if !sub_result.cargo_check_passed {
                                result.cargo_check_passed = false;
                                result.compiler_errors = sub_result.compiler_errors;
                                return Ok(result);
                            }
                        }

                        // Re-compare
                        match crate::comparator::compare_outputs(perl_script, project_dir, args) {
                            Ok(new_comparison) => current_comparison = new_comparison,
                            Err(e) => {
                                warn!("Failed to compare outputs: {}", e);
                                break;
                            }
                        }
                    }

                    result.output_match = Some(current_comparison);
                }
            }
            Err(e) => {
                warn!("Failed to compare outputs: {}", e);
            }
        }
    }

    Ok(result)
}

/// Write project files to disk.
fn write_project(project_dir: &Path, files: &[GeneratedFile], cargo_toml: &str) -> Result<()> {
    // Write Cargo.toml
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Write source files
    for file in files {
        let full_path = project_dir.join(&file.path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, &file.content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use perl_parser::types::GeneratedFile;
    use std::path::PathBuf;

    #[test]
    fn test_write_project() {
        let tmp = tempfile::TempDir::new().unwrap();
        let files = vec![GeneratedFile {
            path: PathBuf::from("src/main.rs"),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
        }];
        let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;

        write_project(tmp.path(), &files, cargo_toml).unwrap();

        assert!(tmp.path().join("Cargo.toml").exists());
        assert!(tmp.path().join("src/main.rs").exists());
    }
}
