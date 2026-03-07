use crate::types::{PerlRegex, RegexOperation};
use regex::Regex;

/// Extracts and analyzes Perl regex patterns from source code.
pub struct PerlRegexAnalyzer {
    match_re: Regex,
    subst_slash_re: Regex,
    subst_pipe_re: Regex,
    tr_slash_re: Regex,
    tr_pipe_re: Regex,
}

/// Features that require fancy-regex instead of the standard regex crate.
const FANCY_FEATURES: &[&str] = &[
    r"\k<",      // Named backreference
    r"\k'",      // Named backreference (alternate)
    "(?<=",      // Lookbehind
    "(?<!",      // Negative lookbehind
    "(?(DEFINE", // Conditional pattern
    "(?{",       // Code block
    "(??{",      // Postponed code block
    "(?1)",      // Recursive subpattern
    "(?R)",      // Recursive pattern
];

impl PerlRegexAnalyzer {
    pub fn new() -> Self {
        Self {
            match_re: Regex::new(
                r#"=~\s*(?:m\s*([/|{(\[])(.+?)(?:[/|}\)]])([gimsxce]*)|/(.+?)/([gimsxce]*))"#,
            )
            .unwrap(),
            // s/pattern/replacement/flags
            subst_slash_re: Regex::new(r#"=~\s*s/(.+?)/(.+?)/([gimsxce]*)"#).unwrap(),
            // s|pattern|replacement|flags
            subst_pipe_re: Regex::new(r#"=~\s*s\|(.+?)\|(.+?)\|([gimsxce]*)"#).unwrap(),
            // tr/from/to/flags  or y/from/to/flags
            tr_slash_re: Regex::new(r#"=~\s*(?:tr|y)/(.+?)/(.+?)/([cdsr]*)"#).unwrap(),
            tr_pipe_re: Regex::new(r#"=~\s*(?:tr|y)\|(.+?)\|(.+?)\|([cdsr]*)"#).unwrap(),
        }
    }

    /// Extract all regex operations from Perl source code.
    pub fn extract_regexes(&self, source: &str) -> Vec<PerlRegex> {
        let mut results = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            // Check substitutions first (more specific)
            let mut found_subst = false;
            for caps in self
                .subst_slash_re
                .captures_iter(line)
                .chain(self.subst_pipe_re.captures_iter(line))
            {
                let pattern = caps[1].to_string();
                let replacement = caps[2].to_string();
                let modifiers = caps[3].to_string();
                let needs_fancy = self.needs_fancy_regex(&pattern);

                results.push(PerlRegex {
                    pattern,
                    modifiers,
                    operation: RegexOperation::Substitute { replacement },
                    needs_fancy_regex: needs_fancy,
                    line: line_num + 1,
                });
                found_subst = true;
            }

            // Check transliterations
            let mut found_tr = false;
            for caps in self
                .tr_slash_re
                .captures_iter(line)
                .chain(self.tr_pipe_re.captures_iter(line))
            {
                let pattern = caps[1].to_string();
                let replacement = caps[2].to_string();
                let modifiers = caps[3].to_string();

                results.push(PerlRegex {
                    pattern,
                    modifiers,
                    operation: RegexOperation::Transliterate { replacement },
                    needs_fancy_regex: false,
                    line: line_num + 1,
                });
                found_tr = true;
            }

            // Check matches (skip if already found a substitution/tr on this line)
            if !found_subst && !found_tr {
                for caps in self.match_re.captures_iter(line) {
                    let pattern = caps
                        .get(2)
                        .or(caps.get(4))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    let modifiers = caps
                        .get(3)
                        .or(caps.get(5))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    let needs_fancy = self.needs_fancy_regex(&pattern);

                    results.push(PerlRegex {
                        pattern,
                        modifiers,
                        operation: RegexOperation::Match,
                        needs_fancy_regex: needs_fancy,
                        line: line_num + 1,
                    });
                }
            }
        }

        results
    }

    /// Check if a pattern requires fancy-regex.
    fn needs_fancy_regex(&self, pattern: &str) -> bool {
        FANCY_FEATURES.iter().any(|feat| pattern.contains(feat))
    }

    /// Convert a Perl regex pattern to a Rust-compatible pattern.
    pub fn to_rust_pattern(&self, perl_regex: &PerlRegex) -> RustRegexConversion {
        let mut pattern = perl_regex.pattern.clone();
        let mut use_fancy = perl_regex.needs_fancy_regex;
        let mut warnings = Vec::new();

        // Convert (?<name>...) (Perl 5.10+) to (?P<name>...) for Rust regex
        let named_cap_re = Regex::new(r#"\(\?<(\w+)>"#).unwrap();
        if named_cap_re.is_match(&pattern) {
            pattern = named_cap_re.replace_all(&pattern, "(?P<$1>").to_string();
        }

        // Perl's \N{U+XXXX} → \x{XXXX}
        let unicode_re = Regex::new(r#"\\N\{U\+([0-9A-Fa-f]+)\}"#).unwrap();
        if unicode_re.is_match(&pattern) {
            pattern = unicode_re.replace_all(&pattern, r"\x{$1}").to_string();
        }

        // Handle modifiers
        let mut case_insensitive = false;
        let mut multi_line = false;
        let mut dot_all = false;
        let mut extended = false;
        let mut global = false;

        for c in perl_regex.modifiers.chars() {
            match c {
                'i' => case_insensitive = true,
                'm' => multi_line = true,
                's' => dot_all = true,
                'x' => extended = true,
                'g' => global = true,
                'e' => {
                    warnings.push(
                        "The /e modifier (eval replacement) has no direct Rust equivalent. Manual conversion required.".to_string(),
                    );
                }
                _ => {}
            }
        }

        // Build inline flags if needed
        let mut inline_flags = String::new();
        if case_insensitive {
            inline_flags.push('i');
        }
        if multi_line {
            inline_flags.push('m');
        }
        if dot_all {
            inline_flags.push('s');
        }
        if extended {
            inline_flags.push('x');
        }

        if !inline_flags.is_empty() {
            pattern = format!("(?{inline_flags}){pattern}");
        }

        // Check for backreferences (\1, \2, etc.)
        let backref_re = Regex::new(r#"\\(\d+)"#).unwrap();
        if backref_re.is_match(&pattern) {
            use_fancy = true;
        }

        let crate_name = if use_fancy { "fancy_regex" } else { "regex" };

        RustRegexConversion {
            pattern,
            crate_name: crate_name.to_string(),
            global,
            warnings,
        }
    }
}

impl Default for PerlRegexAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of converting a Perl regex to Rust.
#[derive(Debug, Clone)]
pub struct RustRegexConversion {
    pub pattern: String,
    pub crate_name: String,
    pub global: bool,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_match() {
        let analyzer = PerlRegexAnalyzer::new();
        let source =
            "if ($line =~ /^(\\d{4})-(\\d{2})-(\\d{2})$/) {\n    print \"matched\\n\";\n}\n";
        let regexes = analyzer.extract_regexes(source);
        assert_eq!(regexes.len(), 1);
        assert!(matches!(regexes[0].operation, RegexOperation::Match));
        assert!(!regexes[0].needs_fancy_regex);
    }

    #[test]
    fn test_extract_substitution() {
        let analyzer = PerlRegexAnalyzer::new();
        let source = "$text =~ s/foo/bar/g;";
        let regexes = analyzer.extract_regexes(source);
        assert_eq!(regexes.len(), 1);
        match &regexes[0].operation {
            RegexOperation::Substitute { replacement } => assert_eq!(replacement, "bar"),
            _ => panic!("Expected Substitute"),
        }
        assert!(regexes[0].modifiers.contains('g'));
    }

    #[test]
    fn test_extract_transliterate() {
        let analyzer = PerlRegexAnalyzer::new();
        let source = "$text =~ tr/a-z/A-Z/;";
        let regexes = analyzer.extract_regexes(source);
        assert_eq!(regexes.len(), 1);
        assert!(matches!(
            regexes[0].operation,
            RegexOperation::Transliterate { .. }
        ));
    }

    #[test]
    fn test_needs_fancy_regex() {
        let analyzer = PerlRegexAnalyzer::new();
        let source = "$text =~ /(?<=prefix)\\w+/;";
        let regexes = analyzer.extract_regexes(source);
        assert_eq!(regexes.len(), 1);
        assert!(regexes[0].needs_fancy_regex);
    }

    #[test]
    fn test_to_rust_pattern_simple() {
        let analyzer = PerlRegexAnalyzer::new();
        let perl_re = PerlRegex {
            pattern: r"^\d{4}-\d{2}-\d{2}$".to_string(),
            modifiers: String::new(),
            operation: RegexOperation::Match,
            needs_fancy_regex: false,
            line: 1,
        };
        let result = analyzer.to_rust_pattern(&perl_re);
        assert_eq!(result.crate_name, "regex");
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_to_rust_pattern_with_modifiers() {
        let analyzer = PerlRegexAnalyzer::new();
        let perl_re = PerlRegex {
            pattern: "hello".to_string(),
            modifiers: "gi".to_string(),
            operation: RegexOperation::Match,
            needs_fancy_regex: false,
            line: 1,
        };
        let result = analyzer.to_rust_pattern(&perl_re);
        assert!(result.pattern.contains("(?i)"));
        assert!(result.global);
    }

    #[test]
    fn test_named_capture_conversion() {
        let analyzer = PerlRegexAnalyzer::new();
        let perl_re = PerlRegex {
            pattern: "(?<year>\\d{4})-(?<month>\\d{2})".to_string(),
            modifiers: String::new(),
            operation: RegexOperation::Match,
            needs_fancy_regex: false,
            line: 1,
        };
        let result = analyzer.to_rust_pattern(&perl_re);
        assert!(result.pattern.contains("(?P<year>"));
        assert!(result.pattern.contains("(?P<month>"));
    }
}
