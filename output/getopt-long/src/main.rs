// Converted from: test-projects/getopt-sample/main.pl
// Original Perl:  Getopt::Long-based CLI argument parsing
//
// Conversion map:
//   GetOptions("verbose!"      => \$verbose)   → #[arg(long)]  verbose: bool
//   GetOptions("output|o=s"    => \$output)    → #[arg(long, short='o')] output: Option<String>
//   GetOptions("count|n=i"     => \$count)     → #[arg(long, short='n')] count: Option<i64>
//   GetOptions("rate=f"        => \$rate)      → #[arg(long)] rate: Option<f64>
//   GetOptions("file=s@"       => \@files)     → #[arg(long, num_args=1..)] file: Vec<String>
//   GetOptions("define=s%"     => \%options)   → #[arg(long, ...)] define: Vec<String> (key=val)
//   GetOptions("debug+"        => \$debug)     → #[arg(long, action=Count)] debug: u8
//   @ARGV (remaining)                          → trailing_args: Vec<String>

use clap::Parser;
use std::collections::HashMap;

/// Converted from Perl Getopt::Long sample
///
/// Original: GetOptions("verbose!" => \$verbose, "output|o=s" => \$output, ...)
#[derive(Parser, Debug)]
#[command(name = "getopt-sample", about = "Converted from Getopt::Long Perl script")]
struct Opts {
    /// Perl: "verbose!" => \$verbose  (negatable boolean)
    /// --verbose to enable, --no-verbose to disable
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Perl: "output|o=s" => \$output  (string with short alias)
    #[arg(long, short = 'o', default_value = "out.txt")]
    output: String,

    /// Perl: "count|n=i" => \$count  (integer with short alias)
    #[arg(long, short = 'n', default_value_t = 1)]
    count: i64,

    /// Perl: "rate=f" => \$rate  (floating-point)
    #[arg(long, default_value_t = 1.0)]
    rate: f64,

    /// Perl: "file=s@" => \@files  (multi-value string array)
    /// Use --file multiple times: --file a.txt --file b.txt
    #[arg(long, num_args = 1..)]
    file: Vec<String>,

    /// Perl: "define=s%" => \%options  (hash: key=value pairs)
    /// Use: --define key=value --define other=val
    #[arg(long, value_name = "KEY=VALUE")]
    define: Vec<String>,

    /// Perl: "debug+" => \$debug  (incremental counter)
    /// Use multiple times: -d -d -d → debug level 3
    #[arg(long, short = 'd', action = clap::ArgAction::Count)]
    debug: u8,

    /// Perl: @ARGV (remaining positional arguments)
    #[arg(trailing_var_arg = true)]
    trailing_args: Vec<String>,
}

/// Parse "key=value" strings from --define args (Perl's =s% hash option type).
/// Perl: my %options; GetOptions("define=s%" => \%options);
fn parse_hash_args(args: &[String]) -> HashMap<String, String> {
    args.iter()
        .filter_map(|s| {
            let mut parts = s.splitn(2, '=');
            let key = parts.next()?.to_string();
            let val = parts.next().unwrap_or("").to_string();
            Some((key, val))
        })
        .collect()
}

fn main() {
    let opts = Opts::parse();

    // Perl: if ($verbose) { print ... }
    if opts.verbose {
        println!("Verbose mode enabled");
        println!("Output: {}", opts.output);
        println!("Count: {}", opts.count);
        println!("Rate: {:.2}", opts.rate);
        println!("Debug level: {}", opts.debug);

        // Perl: printf "Files: %s\n", join(", ", @files) if @files;
        if !opts.file.is_empty() {
            println!("Files: {}", opts.file.join(", "));
        }

        // Perl: for my $k (sort keys %options) { printf "  %s = %s\n", $k, $options{$k}; }
        let options = parse_hash_args(&opts.define);
        let mut keys: Vec<&String> = options.keys().collect();
        keys.sort();
        for k in keys {
            println!("  {} = {}", k, options[k]);
        }
    }

    // Perl: for my $arg (@ARGV) { printf "Positional: %s\n", $arg; }
    for arg in &opts.trailing_args {
        println!("Positional: {}", arg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hash_args_simple() {
        let args = vec!["key=value".to_string(), "foo=bar".to_string()];
        let map = parse_hash_args(&args);
        assert_eq!(map["key"], "value");
        assert_eq!(map["foo"], "bar");
    }

    #[test]
    fn test_parse_hash_args_value_with_equals() {
        // Perl: --define url=http://example.com/path?a=b
        let args = vec!["url=http://example.com/path?a=b".to_string()];
        let map = parse_hash_args(&args);
        assert_eq!(map["url"], "http://example.com/path?a=b");
    }

    #[test]
    fn test_parse_hash_args_empty_value() {
        let args = vec!["flag=".to_string()];
        let map = parse_hash_args(&args);
        assert_eq!(map["flag"], "");
    }

    #[test]
    fn test_parse_hash_args_empty() {
        let map = parse_hash_args(&[]);
        assert!(map.is_empty());
    }
}
