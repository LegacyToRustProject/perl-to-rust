use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an entire Perl project analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlProject {
    pub root: PathBuf,
    pub modules: Vec<PerlModule>,
    pub scripts: Vec<PerlScript>,
    pub cpan_dependencies: Vec<CpanDependency>,
    pub perl_version: Option<String>,
}

/// A Perl module (.pm file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlModule {
    pub path: PathBuf,
    pub package_name: String,
    pub source: String,
    pub subroutines: Vec<Subroutine>,
    pub uses: Vec<UseStatement>,
    pub variables: Vec<Variable>,
    pub is_oop: bool,
    pub parent_classes: Vec<String>,
}

/// A Perl script (.pl file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlScript {
    pub path: PathBuf,
    pub source: String,
    pub subroutines: Vec<Subroutine>,
    pub uses: Vec<UseStatement>,
    pub variables: Vec<Variable>,
    pub has_main: bool,
}

/// A subroutine definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subroutine {
    pub name: String,
    pub body: String,
    pub line_start: usize,
    pub line_end: usize,
    pub is_method: bool,
    pub parameters: Vec<String>,
}

/// A `use` or `require` statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStatement {
    pub module_name: String,
    pub imports: Vec<String>,
    pub is_pragma: bool,
    pub line: usize,
}

/// A variable declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub sigil: Sigil,
    pub scope: Scope,
    pub inferred_type: Option<InferredType>,
    pub line: usize,
}

/// Perl variable sigils.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sigil {
    Scalar, // $
    Array,  // @
    Hash,   // %
}

/// Variable scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    My,
    Our,
    Local,
    Global,
}

/// Type inference result for a Perl variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferredType {
    String,
    Integer,
    Float,
    Bool,
    ArrayOf(Box<InferredType>),
    HashOf(Box<InferredType>, Box<InferredType>),
    Reference(Box<InferredType>),
    Object(String),
    Dynamic,
}

/// CPAN module dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpanDependency {
    pub module_name: String,
    pub version: Option<String>,
    pub rust_equivalent: Option<RustEquivalent>,
}

/// Rust equivalent of a CPAN module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustEquivalent {
    pub crate_name: String,
    pub notes: Option<String>,
}

/// A Perl regex pattern with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlRegex {
    pub pattern: String,
    pub modifiers: String,
    pub operation: RegexOperation,
    pub needs_fancy_regex: bool,
    pub line: usize,
}

/// Type of regex operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegexOperation {
    Match,
    Substitute { replacement: String },
    Transliterate { replacement: String },
}

/// Analysis report for a Perl project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub project_root: PathBuf,
    pub total_files: usize,
    pub total_lines: usize,
    pub modules: Vec<ModuleSummary>,
    pub scripts: Vec<ScriptSummary>,
    pub cpan_dependencies: Vec<CpanDependency>,
    pub regex_count: usize,
    pub oop_modules: usize,
    pub estimated_complexity: Complexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub path: PathBuf,
    pub package_name: String,
    pub subroutine_count: usize,
    pub is_oop: bool,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSummary {
    pub path: PathBuf,
    pub subroutine_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Complexity {
    Low,
    Medium,
    High,
}

/// CPAN to Rust crate mapping table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpanMappings {
    pub modules: HashMap<String, String>,
}

/// Generated Rust project structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustProject {
    pub files: Vec<GeneratedFile>,
    pub cargo_toml: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub cargo_check_passed: bool,
    pub compiler_errors: Vec<String>,
    pub output_match: Option<OutputComparison>,
    pub fix_attempts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputComparison {
    pub perl_output: String,
    pub rust_output: String,
    pub matches: bool,
    pub diff: Option<String>,
}
