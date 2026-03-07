// Getopt::Long → clap conversion library
//
// Maps Getopt::Long option specification patterns to clap constructs.
// This module documents the systematic conversion rules.

/// Getopt::Long option type specifiers → Rust/clap types
///
/// | Perl spec    | Perl type         | Rust type          | clap annotation            |
/// |--------------|-------------------|--------------------|----------------------------|
/// | `=s`         | String            | `String`           | `#[arg(long)]`             |
/// | `=i`         | Integer           | `i64`              | `#[arg(long)]`             |
/// | `=f`         | Float             | `f64`              | `#[arg(long)]`             |
/// | `=o`         | Extended int      | `i64`              | `#[arg(long)]`             |
/// | `!`          | Negatable bool    | `bool`             | `#[arg(long, action=...)]` |
/// | `+`          | Increment counter | `u8`               | `#[arg(long, action=Count)]`|
/// | `=s@`        | String array      | `Vec<String>`      | `#[arg(long, num_args=1..)]`|
/// | `=s%`        | String hash       | `Vec<String>`      | `#[arg(long)]` + parse     |
/// | (none)       | Flag              | `bool`             | `#[arg(long)]`             |
pub mod option_spec {
    /// Parse a Getopt::Long option specification string.
    /// Returns (name, aliases, opt_type, is_required, is_multi).
    pub fn parse_spec(spec: &str) -> OptionSpec {
        // Getopt::Long spec format: "primary|alias1|alias2=type" or "primary!"
        // The type specifier (=s, =i, =f, !, +) is always on the LAST pipe-segment.
        // The PRIMARY name is ALWAYS the FIRST pipe-segment (before stripping type).
        let parts: Vec<&str> = spec.split('|').collect();

        // Separate type specifier from the last segment
        let last_seg = parts[parts.len() - 1];
        let (last_name_part, opt_type) = if let Some(idx) = last_seg.find('=') {
            let name = &last_seg[..idx];
            let type_spec = &last_seg[idx + 1..];
            let t = match type_spec {
                "s" => OptType::Str,
                "s@" => OptType::StrArray,
                "s%" => OptType::StrHash,
                "i" => OptType::Int,
                "i@" => OptType::IntArray,
                "f" => OptType::Float,
                "o" => OptType::ExtInt,
                _ => OptType::Str,
            };
            (name, t)
        } else if last_seg.ends_with('!') {
            (&last_seg[..last_seg.len() - 1], OptType::Negatable)
        } else if last_seg.ends_with('+') {
            (&last_seg[..last_seg.len() - 1], OptType::Incremental)
        } else {
            (last_seg, OptType::Flag)
        };

        // Primary name is the first segment (no type suffix needed)
        let primary = if parts.len() == 1 {
            // Single segment: strip type suffix that was already handled
            last_name_part
        } else {
            parts[0]
        };

        // Aliases = all segments except first (with their type suffix stripped)
        let aliases: Vec<String> = if parts.len() > 1 {
            // Middle segments are pure name aliases; last segment we already stripped
            let mut a: Vec<String> = parts[1..parts.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
            a.push(last_name_part.to_string());
            a
        } else {
            vec![]
        };

        OptionSpec {
            name: primary.to_string(),
            aliases,
            opt_type,
        }
    }

    /// Generate a clap derive field annotation for this option spec.
    pub fn to_clap_field(spec: &OptionSpec) -> ClapField {
        let rust_type = match spec.opt_type {
            OptType::Flag => "bool".to_string(),
            OptType::Negatable => "bool".to_string(),
            OptType::Str => "Option<String>".to_string(),
            OptType::StrArray => "Vec<String>".to_string(),
            OptType::StrHash => "Vec<String>".to_string(),
            OptType::Int => "Option<i64>".to_string(),
            OptType::IntArray => "Vec<i64>".to_string(),
            OptType::Float => "Option<f64>".to_string(),
            OptType::ExtInt => "Option<i64>".to_string(),
            OptType::Incremental => "u8".to_string(),
        };

        let field_name = spec.name.replace('-', "_");

        let attrs = match spec.opt_type {
            OptType::Flag => {
                format!("#[arg(long)]")
            }
            OptType::Negatable => {
                // clap 4: --flag / --no-flag
                let aliases_attr = if !spec.aliases.is_empty() {
                    format!(
                        ", short = '{}'",
                        spec.aliases[0].chars().next().unwrap_or('?')
                    )
                } else {
                    String::new()
                };
                format!("#[arg(long{aliases_attr}, default_value_t = false)]")
            }
            OptType::Incremental => {
                "#[arg(long, action = clap::ArgAction::Count)]".to_string()
            }
            OptType::StrArray => {
                "#[arg(long, num_args = 1..)]".to_string()
            }
            OptType::StrHash => {
                // No native hash support in clap; use Vec<String> with "key=value" parsing
                "#[arg(long, value_name = \"KEY=VALUE\", num_args = 1..)]".to_string()
            }
            _ => {
                let alias_part = if !spec.aliases.is_empty() {
                    let first = spec.aliases[0].chars().next().unwrap_or('?');
                    format!(", short = '{first}'")
                } else {
                    String::new()
                };
                format!("#[arg(long{alias_part})]")
            }
        };

        ClapField {
            field_name,
            rust_type,
            attrs,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum OptType {
        Flag,
        Negatable,
        Str,
        StrArray,
        StrHash,
        Int,
        IntArray,
        Float,
        ExtInt,
        Incremental,
    }

    #[derive(Debug, Clone)]
    pub struct OptionSpec {
        pub name: String,
        pub aliases: Vec<String>,
        pub opt_type: OptType,
    }

    #[derive(Debug, Clone)]
    pub struct ClapField {
        pub field_name: String,
        pub rust_type: String,
        pub attrs: String,
    }
}

#[cfg(test)]
mod tests {
    use super::option_spec::*;

    #[test]
    fn test_parse_flag() {
        let spec = parse_spec("verbose");
        assert_eq!(spec.name, "verbose");
        assert_eq!(spec.opt_type, OptType::Flag);
    }

    #[test]
    fn test_parse_negatable() {
        let spec = parse_spec("verbose!");
        assert_eq!(spec.name, "verbose");
        assert_eq!(spec.opt_type, OptType::Negatable);
    }

    #[test]
    fn test_parse_string() {
        let spec = parse_spec("output=s");
        assert_eq!(spec.name, "output");
        assert_eq!(spec.opt_type, OptType::Str);
    }

    #[test]
    fn test_parse_string_with_alias() {
        // "output|o=s" → primary="output", alias=["o"], type=Str
        let spec = parse_spec("output|o=s");
        assert_eq!(spec.name, "output");
        assert_eq!(spec.aliases, vec!["o"]);
        assert_eq!(spec.opt_type, OptType::Str);
    }

    #[test]
    fn test_parse_integer() {
        let spec = parse_spec("count|n=i");
        assert_eq!(spec.name, "count");
        assert_eq!(spec.opt_type, OptType::Int);
    }

    #[test]
    fn test_parse_float() {
        let spec = parse_spec("rate=f");
        assert_eq!(spec.name, "rate");
        assert_eq!(spec.opt_type, OptType::Float);
    }

    #[test]
    fn test_parse_array() {
        let spec = parse_spec("file=s@");
        assert_eq!(spec.name, "file");
        assert_eq!(spec.opt_type, OptType::StrArray);
    }

    #[test]
    fn test_parse_hash() {
        let spec = parse_spec("define=s%");
        assert_eq!(spec.name, "define");
        assert_eq!(spec.opt_type, OptType::StrHash);
    }

    #[test]
    fn test_parse_incremental() {
        let spec = parse_spec("debug+");
        assert_eq!(spec.name, "debug");
        assert_eq!(spec.opt_type, OptType::Incremental);
    }

    #[test]
    fn test_clap_field_flag() {
        let spec = parse_spec("verbose");
        let field = to_clap_field(&spec);
        assert_eq!(field.field_name, "verbose");
        assert_eq!(field.rust_type, "bool");
        assert!(field.attrs.contains("long"));
    }

    #[test]
    fn test_clap_field_string() {
        let spec = parse_spec("output=s");
        let field = to_clap_field(&spec);
        assert_eq!(field.rust_type, "Option<String>");
    }

    #[test]
    fn test_clap_field_array() {
        let spec = parse_spec("file=s@");
        let field = to_clap_field(&spec);
        assert_eq!(field.rust_type, "Vec<String>");
        assert!(field.attrs.contains("num_args"));
    }

    #[test]
    fn test_clap_field_incremental() {
        let spec = parse_spec("debug+");
        let field = to_clap_field(&spec);
        assert_eq!(field.rust_type, "u8");
        assert!(field.attrs.contains("Count"));
    }

    #[test]
    fn test_generate_struct_code() {
        let specs = vec![
            "verbose!",
            "output|o=s",
            "count|n=i",
            "rate=f",
            "file=s@",
            "define=s%",
            "debug+",
        ];

        let fields: Vec<_> = specs
            .iter()
            .map(|s| {
                let spec = parse_spec(s);
                to_clap_field(&spec)
            })
            .collect();

        // Verify all fields generated
        assert_eq!(fields.len(), 7);
        assert_eq!(fields[0].field_name, "verbose");
        assert_eq!(fields[1].field_name, "output");
        assert_eq!(fields[6].field_name, "debug");
    }
}
