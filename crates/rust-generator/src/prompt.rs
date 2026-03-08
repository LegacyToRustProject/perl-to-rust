use perl_parser::types::*;

/// Build the system prompt for Perl-to-Rust conversion.
pub fn system_prompt() -> String {
    r#"You are an expert Perl 5 to Rust converter. You convert Perl code to idiomatic, safe Rust code.

## Core Principles

1. **Make implicit explicit.** Perl relies heavily on implicit behavior ($_, context sensitivity, auto-vivification). In Rust, everything must be explicit.

2. **Prefer static types.** Infer concrete types from context whenever possible. Only use dynamic types (enums) as a last resort.

3. **Use idiomatic Rust.** Prefer iterators over loops, Result/Option over exceptions, owned types where appropriate.

4. **Preserve behavior exactly.** The Rust output must produce identical results to the Perl input for the same inputs.

## Variable Conversion Rules

- `$scalar` → infer type from usage (String, i64, f64, bool, etc.)
- `@array` → `Vec<T>`
- `%hash` → `HashMap<K, V>`
- `$_` (implicit variable) → named explicit variable in each context
- Scalar context of array (`my $count = @arr`) → `.len()`
- List context of array (`my @copy = @arr`) → `.clone()`

## Regex Conversion

- Simple patterns: use `regex` crate
- Lookbehind, backreferences: use `fancy_regex` crate
- Named captures: `(?<name>...)` → `(?P<name>...)` (already Rust-compatible)
- `/g` modifier → `replace_all()` or `find_iter()`
- `/e` modifier → manual closure-based replacement
- `tr/a-z/A-Z/` → `.to_uppercase()` or char mapping

## Iterator and Closure Rules (CRITICAL)

**Closure parameter depth for slices** — common error source:
When iterating over `&[T]`, `.iter()` yields `&T`. Use `|&x|` not `|&&x|`:

  // WRONG: |&&x| causes E0308 mismatched types
  nums.iter().any(|&&x| x > 1.0);
  // CORRECT: &[f64].iter() yields &f64, pattern |&x| gives f64
  nums.iter().any(|&x| x > 1.0);
  // ALSO CORRECT: explicit deref
  nums.iter().any(|x| *x > 1.0);

Perl `grep { $_ > 0 } @list` → `list.iter().filter(|&x| *x > 0.0)`
Perl `map { $_ * 2 } @list` → `list.iter().map(|&x| x * 2.0)` (Copy) or `.map(|x| x.clone())`

**Numbered captures** `$1, $2`:
  // Perl: if ($text =~ /^(\d+)-(\d+)$/) { print "$1 $2\n"; }
  // Rust: if let Some(caps) = re.captures(text) { println!("{} {}", &caps[1], &caps[2]); }

**Named captures** `$+{name}`:
  // Perl: $+{level}
  // Rust: &caps["level"]  (or caps.name("level").unwrap().as_str())

## DBI → SQLx Conversion

- `DBI->connect($dsn)` → `sqlx::MySqlPool::connect(&url).await?`
- `$dbh->prepare($sql); $sth->execute($v)` → `sqlx::query($sql).bind(v).execute(&pool).await?`
- `$sth->fetchrow_hashref()` → `query_as::<_, T>(sql).fetch_one(&pool).await?`
- `$sth->fetchall_arrayref({})` → `query_as::<_, T>(sql).fetch_all(&pool).await?`
- Transaction `begin_work/commit/rollback` → `pool.begin().await? / tx.commit().await?`
- Use `#[derive(sqlx::FromRow)]` for structs mapped from `fetchrow_hashref`

## CGI.pm → Axum Conversion

- `my $q = CGI->new` → Axum extractors (no CGI object needed)
- `$q->param('name')` → `Query<Params>` or `Form<Params>` extractor
- `print $q->header('text/html'); print $body` → return `Html<String>`
- `print $q->header('application/json')` → return `Json<T>`
- `$q->header(-status => '404 ...')` → `(StatusCode::NOT_FOUND, body).into_response()`
- `print $q->redirect($url)` → `Redirect::to(url)`
- `if ($action eq 'x') { ... } elsif ($action eq 'y') { ... }` → `Router::new().route()`
- Global DB handle `$dbh` → `State<Arc<AppState>>` extractor with connection pool

## OOP Conversion (bless-based)

- `package Foo` → `struct Foo`
- `bless { field => value }, $class` → `Foo { field: value }`
- `sub new { ... }` → `fn new(...) -> Self`
- Methods (first arg is `$self`) → `fn method(&self, ...)`
- `@ISA` / `use parent` → trait implementation
- Moose/Moo attributes → struct fields with builder pattern

## Output Format

Return ONLY valid Rust code. Do not include explanations outside code blocks.
If multiple files are needed, separate them with:
```
// === FILE: path/to/file.rs ===
```

Always include necessary `use` statements and `Cargo.toml` dependencies."#
        .to_string()
}

/// Build a conversion prompt for a single Perl file.
pub fn file_conversion_prompt(
    source: &str,
    file_path: &str,
    context: &ConversionContext,
) -> String {
    let mut prompt = format!(
        "Convert the following Perl file to Rust.\n\nFile: {}\n\n",
        file_path
    );

    if !context.cpan_mappings.is_empty() {
        prompt.push_str("## CPAN Module Mappings\n\n");
        for (perl_mod, rust_crate) in &context.cpan_mappings {
            prompt.push_str(&format!("- `{}` → `{}`\n", perl_mod, rust_crate));
        }
        prompt.push('\n');
    }

    if !context.project_modules.is_empty() {
        prompt.push_str("## Other modules in this project\n\n");
        for module in &context.project_modules {
            prompt.push_str(&format!(
                "- `{}` ({})\n",
                module.package_name,
                module.path.display()
            ));
        }
        prompt.push('\n');
    }

    if let Some(perl_version) = &context.perl_version {
        prompt.push_str(&format!("## Perl Version: {}\n\n", perl_version));
    }

    prompt.push_str("## Perl Source Code\n\n```perl\n");
    prompt.push_str(source);
    prompt.push_str("\n```\n\n");
    prompt.push_str("Convert this to idiomatic Rust. Return only the Rust code.");

    prompt
}

/// Build a prompt for fixing compiler errors.
pub fn fix_prompt(rust_code: &str, errors: &[String]) -> String {
    let mut prompt = String::from(
        "The following Rust code has compiler errors. Fix them and return the corrected code.\n\n",
    );
    prompt.push_str("## Current Rust Code\n\n```rust\n");
    prompt.push_str(rust_code);
    prompt.push_str("\n```\n\n## Compiler Errors\n\n```\n");
    for error in errors {
        prompt.push_str(error);
        prompt.push('\n');
    }
    prompt.push_str("```\n\nReturn ONLY the fixed Rust code.");
    prompt
}

/// Build a prompt for fixing output mismatches.
pub fn output_mismatch_prompt(rust_code: &str, perl_output: &str, rust_output: &str) -> String {
    let mut prompt = String::from(
        "The Rust code compiles but produces different output than the original Perl code. Fix it.\n\n",
    );
    prompt.push_str("## Rust Code\n\n```rust\n");
    prompt.push_str(rust_code);
    prompt.push_str("\n```\n\n## Expected Output (from Perl)\n\n```\n");
    prompt.push_str(perl_output);
    prompt.push_str("\n```\n\n## Actual Output (from Rust)\n\n```\n");
    prompt.push_str(rust_output);
    prompt.push_str("\n```\n\nReturn ONLY the fixed Rust code.");
    prompt
}

/// Context information provided alongside conversion prompts.
pub struct ConversionContext {
    pub cpan_mappings: Vec<(String, String)>,
    pub project_modules: Vec<ModuleSummary>,
    pub perl_version: Option<String>,
}

/// Build a prompt for generating Cargo.toml.
pub fn cargo_toml_prompt(project_name: &str, dependencies: &[CpanDependency]) -> String {
    let mut prompt = format!(
        "Generate a Cargo.toml for a Rust project named '{}' that was converted from Perl.\n\n",
        project_name
    );
    prompt.push_str("## Required dependencies based on CPAN modules used:\n\n");
    for dep in dependencies {
        if let Some(ref equiv) = dep.rust_equivalent {
            if !equiv.crate_name.is_empty() && !equiv.crate_name.starts_with('#') {
                prompt.push_str(&format!(
                    "- {} (from Perl's {})\n",
                    equiv.crate_name, dep.module_name
                ));
            }
        }
    }
    prompt.push_str("\nReturn ONLY the Cargo.toml content.");
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_not_empty() {
        let prompt = system_prompt();
        assert!(prompt.contains("Perl 5 to Rust"));
        assert!(prompt.contains("implicit explicit"));
    }

    #[test]
    fn test_file_conversion_prompt() {
        let context = ConversionContext {
            cpan_mappings: vec![("JSON".to_string(), "serde_json".to_string())],
            project_modules: vec![],
            perl_version: Some("5.32".to_string()),
        };
        let prompt = file_conversion_prompt("print 'hello';", "hello.pl", &context);
        assert!(prompt.contains("print 'hello'"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("5.32"));
    }

    #[test]
    fn test_fix_prompt() {
        let prompt = fix_prompt(
            "fn main() { let x: i32 = \"hello\"; }",
            &["error[E0308]: mismatched types".to_string()],
        );
        assert!(prompt.contains("E0308"));
        assert!(prompt.contains("fn main"));
    }
}
