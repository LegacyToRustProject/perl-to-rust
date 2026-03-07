use crate::types::{CpanDependency, CpanMappings, RustEquivalent};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Manages CPAN module to Rust crate mappings.
pub struct CpanMapper {
    mappings: HashMap<String, String>,
}

impl CpanMapper {
    /// Load mappings from a TOML file.
    pub fn from_toml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read CPAN mappings from {}", path.display()))?;
        let parsed: CpanMappings =
            toml::from_str(&content).context("Failed to parse CPAN mappings TOML")?;
        Ok(Self {
            mappings: parsed.modules,
        })
    }

    /// Create mapper with built-in default mappings.
    pub fn with_defaults() -> Self {
        let mut mappings = HashMap::new();
        let defaults = [
            ("LWP::UserAgent", "reqwest"),
            ("HTTP::Tiny", "reqwest"),
            ("WWW::Mechanize", "reqwest"),
            ("JSON", "serde_json"),
            ("JSON::XS", "serde_json"),
            ("JSON::PP", "serde_json"),
            ("Cpanel::JSON::XS", "serde_json"),
            ("DBI", "sqlx"),
            ("DBD::mysql", "sqlx # with mysql feature"),
            ("DBD::Pg", "sqlx # with postgres feature"),
            ("DBD::SQLite", "sqlx # with sqlite feature"),
            ("DateTime", "chrono"),
            ("Time::Piece", "chrono"),
            ("Time::HiRes", "std::time"),
            ("File::Path", "std::fs"),
            ("File::Basename", "std::path::Path"),
            ("File::Spec", "std::path"),
            ("File::Find", "walkdir"),
            ("File::Temp", "tempfile"),
            ("File::Copy", "std::fs"),
            ("File::Slurp", "std::fs::read_to_string"),
            ("Path::Tiny", "std::path + std::fs"),
            ("Getopt::Long", "clap"),
            ("Getopt::Std", "clap"),
            ("Test::More", "# built-in test framework"),
            ("Test::Simple", "# built-in test framework"),
            ("Test::Deep", "# assert_eq! + custom matchers"),
            ("Test::Exception", "# #[should_panic] or Result"),
            ("Moose", "# struct + impl + traits"),
            ("Moo", "# struct + impl + traits"),
            ("Mouse", "# struct + impl + traits"),
            ("Class::Accessor", "# struct with pub fields"),
            ("Try::Tiny", "# Result<T,E> / anyhow"),
            ("Carp", "# anyhow / thiserror"),
            ("Log::Log4perl", "tracing"),
            ("Log::Dispatch", "tracing"),
            ("XML::LibXML", "quick-xml"),
            ("XML::Simple", "quick-xml + serde"),
            ("XML::Twig", "quick-xml"),
            ("Text::CSV", "csv"),
            ("Text::CSV_XS", "csv"),
            ("MIME::Base64", "base64"),
            ("Digest::SHA", "sha2"),
            ("Digest::MD5", "md5"),
            ("Encode", "encoding_rs"),
            ("IO::Socket::SSL", "rustls"),
            ("Net::SSLeay", "rustls"),
            ("IO::Socket::INET", "std::net::TcpStream"),
            ("Socket", "std::net"),
            ("POSIX", "libc + nix"),
            ("Storable", "serde + bincode"),
            ("Data::Dumper", "# dbg! macro / serde_json"),
            ("Scalar::Util", "# Rust type system"),
            ("List::Util", "# Iterator methods"),
            ("List::MoreUtils", "# Iterator methods"),
            ("YAML", "serde_yaml"),
            ("YAML::XS", "serde_yaml"),
            ("Template", "tera"),
            ("HTML::Template", "tera"),
            ("CGI", "axum # or actix-web"),
            ("Mojolicious", "axum"),
            ("Dancer", "axum"),
            ("Catalyst", "axum"),
            ("Plack", "axum / tower"),
            ("AnyEvent", "tokio"),
            ("IO::Async", "tokio"),
            ("POE", "tokio"),
            ("Parallel::ForkManager", "rayon / tokio::spawn"),
            ("threads", "std::thread / tokio"),
        ];

        for (perl, rust) in defaults {
            mappings.insert(perl.to_string(), rust.to_string());
        }

        Self { mappings }
    }

    /// Merge additional mappings (from a file or user config).
    pub fn merge(&mut self, other: &CpanMappings) {
        for (k, v) in &other.modules {
            self.mappings.insert(k.clone(), v.clone());
        }
    }

    /// Look up the Rust equivalent for a CPAN module.
    pub fn lookup(&self, module_name: &str) -> Option<RustEquivalent> {
        self.mappings.get(module_name).map(|crate_name| {
            let (name, notes) = if crate_name.contains('#') {
                let parts: Vec<&str> = crate_name.splitn(2, '#').collect();
                (
                    parts[0].trim().to_string(),
                    Some(parts[1].trim().to_string()),
                )
            } else {
                (crate_name.clone(), None)
            };
            RustEquivalent {
                crate_name: name,
                notes,
            }
        })
    }

    /// Resolve CPAN dependencies to their Rust equivalents.
    pub fn resolve_dependencies(&self, deps: &mut [CpanDependency]) {
        for dep in deps {
            if dep.rust_equivalent.is_none() {
                dep.rust_equivalent = self.lookup(&dep.module_name);
            }
        }
    }

    /// Get all known mappings.
    pub fn all_mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }
}

/// Parse a cpanfile and extract dependencies.
pub fn parse_cpanfile(content: &str) -> Vec<CpanDependency> {
    let re = regex::Regex::new(r#"requires\s+['"]([\w:]+)['"](?:\s*,\s*['"]?([^'";\s]+)['"]?)?"#)
        .unwrap();

    re.captures_iter(content)
        .map(|caps| CpanDependency {
            module_name: caps[1].to_string(),
            version: caps.get(2).map(|m| m.as_str().to_string()),
            rust_equivalent: None,
        })
        .collect()
}

/// Parse a Makefile.PL PREREQ_PM section.
pub fn parse_makefile_pl_prereqs(content: &str) -> Vec<CpanDependency> {
    let re = regex::Regex::new(r#"['"]([\w:]+)['"]\s*=>\s*['"]?([^'",\s}]+)['"]?"#).unwrap();
    let mut deps = Vec::new();

    // Find PREREQ_PM section
    if let Some(start) = content.find("PREREQ_PM") {
        let section = &content[start..];
        if let Some(brace_start) = section.find('{') {
            let section = &section[brace_start..];
            // Find matching close brace (simple heuristic)
            let mut depth = 0;
            let mut end = 0;
            for (i, c) in section.char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > 0 {
                let prereq_section = &section[..=end];
                for caps in re.captures_iter(prereq_section) {
                    deps.push(CpanDependency {
                        module_name: caps[1].to_string(),
                        version: Some(caps[2].to_string()),
                        rust_equivalent: None,
                    });
                }
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mappings() {
        let mapper = CpanMapper::with_defaults();
        let result = mapper.lookup("JSON").unwrap();
        assert_eq!(result.crate_name, "serde_json");
        assert!(result.notes.is_none());
    }

    #[test]
    fn test_mapping_with_notes() {
        let mapper = CpanMapper::with_defaults();
        let result = mapper.lookup("Moose").unwrap();
        assert!(result.notes.is_some());
    }

    #[test]
    fn test_unknown_module() {
        let mapper = CpanMapper::with_defaults();
        assert!(mapper.lookup("My::Custom::Module").is_none());
    }

    #[test]
    fn test_parse_cpanfile() {
        let content = r#"
requires 'Mojolicious', '9.0';
requires 'DBI';
requires 'JSON::XS', '>=3.0';
"#;
        let deps = parse_cpanfile(content);
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].module_name, "Mojolicious");
        assert_eq!(deps[0].version.as_deref(), Some("9.0"));
        assert_eq!(deps[1].module_name, "DBI");
        assert!(deps[1].version.is_none());
    }

    #[test]
    fn test_resolve_dependencies() {
        let mapper = CpanMapper::with_defaults();
        let mut deps = vec![
            CpanDependency {
                module_name: "JSON".to_string(),
                version: None,
                rust_equivalent: None,
            },
            CpanDependency {
                module_name: "Unknown::Module".to_string(),
                version: None,
                rust_equivalent: None,
            },
        ];
        mapper.resolve_dependencies(&mut deps);
        assert!(deps[0].rust_equivalent.is_some());
        assert!(deps[1].rust_equivalent.is_none());
    }

    #[test]
    fn test_from_toml() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            r#"[modules]
"My::Module" = "my_crate"
"Another::One" = "another # custom note"
"#,
        )
        .unwrap();

        let mapper = CpanMapper::from_toml(tmp.path()).unwrap();
        assert_eq!(mapper.lookup("My::Module").unwrap().crate_name, "my_crate");
        let another = mapper.lookup("Another::One").unwrap();
        assert_eq!(another.crate_name, "another");
        assert_eq!(another.notes.as_deref(), Some("custom note"));
    }
}
