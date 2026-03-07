use anyhow::Result;
use perl_parser::types::OutputComparison;
use std::path::Path;

use crate::compiler;

/// Compare the output of a Perl script with its Rust equivalent.
pub fn compare_outputs(
    perl_script: &Path,
    rust_project_dir: &Path,
    args: &[&str],
) -> Result<OutputComparison> {
    let perl_output = compiler::run_perl(perl_script, args)?;
    let rust_output = compiler::run_binary(rust_project_dir, args)?;

    let matches = normalize_output(&perl_output) == normalize_output(&rust_output);

    let diff = if matches {
        None
    } else {
        Some(unified_diff(&perl_output, &rust_output))
    };

    Ok(OutputComparison {
        perl_output,
        rust_output,
        matches,
        diff,
    })
}

/// Normalize output for comparison (trim trailing whitespace, normalize line endings).
fn normalize_output(output: &str) -> String {
    output
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Generate a simple unified diff between two strings.
fn unified_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let mut diff = String::new();
    diff.push_str("--- expected (Perl)\n");
    diff.push_str("+++ actual (Rust)\n");

    let max_len = expected_lines.len().max(actual_lines.len());
    for i in 0..max_len {
        let exp = expected_lines.get(i).copied().unwrap_or("");
        let act = actual_lines.get(i).copied().unwrap_or("");

        if exp != act {
            if i < expected_lines.len() {
                diff.push_str(&format!("-{}\n", exp));
            }
            if i < actual_lines.len() {
                diff.push_str(&format!("+{}\n", act));
            }
        } else {
            diff.push_str(&format!(" {}\n", exp));
        }
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_output() {
        assert_eq!(normalize_output("hello  \nworld  \n\n"), "hello\nworld");
    }

    #[test]
    fn test_normalize_output_matching() {
        let a = "Hello, World!\n";
        let b = "Hello, World!\r\n";
        assert_eq!(normalize_output(a), normalize_output(b));
    }

    #[test]
    fn test_unified_diff() {
        let diff = unified_diff("hello\nworld", "hello\nearth");
        assert!(diff.contains("-world"));
        assert!(diff.contains("+earth"));
        assert!(diff.contains(" hello"));
    }

    #[test]
    fn test_unified_diff_identical() {
        let diff = unified_diff("hello\nworld", "hello\nworld");
        // Should not contain diff markers (lines starting with - or + but not the header)
        let body: Vec<&str> = diff.lines().skip(2).collect(); // skip --- and +++ headers
        assert!(body.iter().all(|l| l.starts_with(' ')));
    }
}
