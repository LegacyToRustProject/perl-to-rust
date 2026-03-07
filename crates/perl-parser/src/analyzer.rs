use crate::types::*;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Analyzes a Perl project directory and produces a structured report.
pub struct PerlAnalyzer {
    package_re: Regex,
    sub_re: Regex,
    use_re: Regex,
    my_re: Regex,
    our_re: Regex,
    local_re: Regex,
    bless_re: Regex,
    isa_re: Regex,
    version_re: Regex,
}

impl PerlAnalyzer {
    pub fn new() -> Self {
        Self {
            package_re: Regex::new(r#"^\s*package\s+([\w:]+)\s*;"#).unwrap(),
            sub_re: Regex::new(r#"^\s*sub\s+(\w+)\s*\{?"#).unwrap(),
            use_re: Regex::new(
                r#"^\s*use\s+([\w:]+)(?:\s+(?:qw\(([^)]*)\)|'([^']*)'|"([^"]*)"|([^;]+)))?\s*;"#,
            )
            .unwrap(),
            my_re: Regex::new(r#"^\s*my\s+([\$@%])(\w+)"#).unwrap(),
            our_re: Regex::new(r#"^\s*our\s+([\$@%])(\w+)"#).unwrap(),
            local_re: Regex::new(r#"^\s*local\s+([\$@%])(\w+)"#).unwrap(),
            bless_re: Regex::new(r#"\bbless\b"#).unwrap(),
            isa_re: Regex::new(r#"(?:use\s+parent|use\s+base|@ISA\s*=)\s*[^;]*['"]?([\w:]+)"#)
                .unwrap(),
            version_re: Regex::new(r#"use\s+v?(5\.\d+(?:\.\d+)?)"#).unwrap(),
        }
    }

    /// Analyze an entire Perl project directory.
    pub fn analyze_project(&self, root: &Path) -> Result<PerlProject> {
        let mut modules = Vec::new();
        let mut scripts = Vec::new();
        let mut perl_version = None;

        let files = self.find_perl_files(root)?;
        for file in &files {
            let source = std::fs::read_to_string(file)
                .with_context(|| format!("Failed to read {}", file.display()))?;

            // Detect Perl version
            if perl_version.is_none() {
                if let Some(caps) = self.version_re.captures(&source) {
                    perl_version = Some(caps[1].to_string());
                }
            }

            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "pm" => modules.push(self.analyze_module(file, &source)?),
                "pl" | "cgi" => scripts.push(self.analyze_script(file, &source)?),
                _ => {
                    // Check shebang for perl scripts
                    if source.starts_with("#!")
                        && source.lines().next().is_some_and(|l| l.contains("perl"))
                    {
                        scripts.push(self.analyze_script(file, &source)?);
                    }
                }
            }
        }

        // Collect all CPAN dependencies
        let mut cpan_deps: Vec<CpanDependency> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for module in &modules {
            for u in &module.uses {
                if !u.is_pragma && seen.insert(u.module_name.clone()) {
                    cpan_deps.push(CpanDependency {
                        module_name: u.module_name.clone(),
                        version: None,
                        rust_equivalent: None,
                    });
                }
            }
        }
        for script in &scripts {
            for u in &script.uses {
                if !u.is_pragma && seen.insert(u.module_name.clone()) {
                    cpan_deps.push(CpanDependency {
                        module_name: u.module_name.clone(),
                        version: None,
                        rust_equivalent: None,
                    });
                }
            }
        }

        Ok(PerlProject {
            root: root.to_path_buf(),
            modules,
            scripts,
            cpan_dependencies: cpan_deps,
            perl_version,
        })
    }

    /// Generate an analysis report from a parsed project.
    pub fn generate_report(&self, project: &PerlProject) -> AnalysisReport {
        let total_files = project.modules.len() + project.scripts.len();
        let total_lines: usize = project
            .modules
            .iter()
            .map(|m| m.source.lines().count())
            .chain(project.scripts.iter().map(|s| s.source.lines().count()))
            .sum();

        let modules: Vec<ModuleSummary> = project
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

        let scripts: Vec<ScriptSummary> = project
            .scripts
            .iter()
            .map(|s| ScriptSummary {
                path: s.path.clone(),
                subroutine_count: s.subroutines.len(),
                line_count: s.source.lines().count(),
            })
            .collect();

        let regex_count = project
            .modules
            .iter()
            .map(|m| self.count_regexes(&m.source))
            .chain(
                project
                    .scripts
                    .iter()
                    .map(|s| self.count_regexes(&s.source)),
            )
            .sum();

        let oop_modules = project.modules.iter().filter(|m| m.is_oop).count();

        let estimated_complexity = if total_lines > 10000 || oop_modules > 10 {
            Complexity::High
        } else if total_lines > 1000 || oop_modules > 3 {
            Complexity::Medium
        } else {
            Complexity::Low
        };

        AnalysisReport {
            project_root: project.root.clone(),
            total_files,
            total_lines,
            modules,
            scripts,
            cpan_dependencies: project.cpan_dependencies.clone(),
            regex_count,
            oop_modules,
            estimated_complexity,
        }
    }

    fn find_perl_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_dir(root, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn walk_dir(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "blib" && name != "local" {
                    self.walk_dir(&path, files)?;
                }
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "pl" | "pm" | "cgi" | "t") {
                    files.push(path);
                } else if ext.is_empty() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.starts_with("#!/")
                            && content.lines().next().is_some_and(|l| l.contains("perl"))
                        {
                            files.push(path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_module(&self, path: &Path, source: &str) -> Result<PerlModule> {
        let package_name = self
            .package_re
            .captures(source)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| "main".to_string());

        let subroutines = self.extract_subroutines(source);
        let uses = self.extract_uses(source);
        let variables = self.extract_variables(source);
        let is_oop = self.bless_re.is_match(source) || self.detect_oop_patterns(source);
        let parent_classes = self.extract_parent_classes(source);

        debug!("Analyzed module: {} at {}", package_name, path.display());

        Ok(PerlModule {
            path: path.to_path_buf(),
            package_name,
            source: source.to_string(),
            subroutines,
            uses,
            variables,
            is_oop,
            parent_classes,
        })
    }

    fn analyze_script(&self, path: &Path, source: &str) -> Result<PerlScript> {
        let subroutines = self.extract_subroutines(source);
        let uses = self.extract_uses(source);
        let variables = self.extract_variables(source);
        let has_main =
            source.contains("__MAIN__") || !subroutines.is_empty() || source.lines().count() > 1;

        debug!("Analyzed script: {}", path.display());

        Ok(PerlScript {
            path: path.to_path_buf(),
            source: source.to_string(),
            subroutines,
            uses,
            variables,
            has_main,
        })
    }

    fn extract_subroutines(&self, source: &str) -> Vec<Subroutine> {
        let mut subs = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            if let Some(caps) = self.sub_re.captures(lines[i]) {
                let name = caps[1].to_string();
                let line_start = i + 1;
                let mut brace_count =
                    lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                let mut body_lines = vec![lines[i].to_string()];
                let mut j = i + 1;

                if brace_count == 0 {
                    while j < lines.len() {
                        body_lines.push(lines[j].to_string());
                        brace_count += lines[j].matches('{').count() as i32
                            - lines[j].matches('}').count() as i32;
                        j += 1;
                        if brace_count > 0 {
                            break;
                        }
                    }
                }

                while j < lines.len() && brace_count > 0 {
                    body_lines.push(lines[j].to_string());
                    brace_count +=
                        lines[j].matches('{').count() as i32 - lines[j].matches('}').count() as i32;
                    j += 1;
                }

                let body = body_lines.join("\n");
                let is_method = body.contains("my ($self")
                    || body.contains("my $self")
                    || body.contains("shift");
                let parameters = self.extract_parameters(&body);

                subs.push(Subroutine {
                    name,
                    body,
                    line_start,
                    line_end: j,
                    is_method,
                    parameters,
                });
                i = j;
                continue;
            }
            i += 1;
        }

        subs
    }

    fn extract_parameters(&self, body: &str) -> Vec<String> {
        let param_re = Regex::new(r#"my\s*\(([^)]+)\)\s*=\s*@_"#).unwrap();
        if let Some(caps) = param_re.captures(body) {
            return caps[1].split(',').map(|s| s.trim().to_string()).collect();
        }
        let shift_re = Regex::new(r#"my\s+([\$@%]\w+)\s*=\s*shift"#).unwrap();
        shift_re
            .captures_iter(body)
            .map(|c| c[1].to_string())
            .collect()
    }

    fn extract_uses(&self, source: &str) -> Vec<UseStatement> {
        let mut uses = Vec::new();
        let pragmas = [
            "strict", "warnings", "utf8", "feature", "constant", "lib", "vars", "Exporter",
            "parent", "base", "overload", "mro",
        ];

        for (i, line) in source.lines().enumerate() {
            if let Some(caps) = self.use_re.captures(line) {
                let module_name = caps[1].to_string();
                let imports = caps
                    .get(2)
                    .or(caps.get(3))
                    .or(caps.get(4))
                    .or(caps.get(5))
                    .map(|m| {
                        m.as_str()
                            .split_whitespace()
                            .map(|s| {
                                s.trim_matches(|c| c == ',' || c == '\'' || c == '"')
                                    .to_string()
                            })
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                let is_pragma = pragmas.iter().any(|p| *p == module_name)
                    || module_name.chars().next().is_some_and(|c| c.is_lowercase());

                uses.push(UseStatement {
                    module_name,
                    imports,
                    is_pragma,
                    line: i + 1,
                });
            }
        }

        uses
    }

    fn extract_variables(&self, source: &str) -> Vec<Variable> {
        let mut vars = Vec::new();

        for (i, line) in source.lines().enumerate() {
            let (scope, re) = if self.my_re.is_match(line) {
                (Scope::My, &self.my_re)
            } else if self.our_re.is_match(line) {
                (Scope::Our, &self.our_re)
            } else if self.local_re.is_match(line) {
                (Scope::Local, &self.local_re)
            } else {
                continue;
            };

            if let Some(caps) = re.captures(line) {
                let sigil = match &caps[1] {
                    "$" => Sigil::Scalar,
                    "@" => Sigil::Array,
                    "%" => Sigil::Hash,
                    _ => continue,
                };
                let name = caps[2].to_string();
                let inferred_type = self.infer_type(line, sigil);

                vars.push(Variable {
                    name,
                    sigil,
                    scope,
                    inferred_type,
                    line: i + 1,
                });
            }
        }

        vars
    }

    fn infer_type(&self, line: &str, sigil: Sigil) -> Option<InferredType> {
        match sigil {
            Sigil::Array => Some(InferredType::ArrayOf(Box::new(InferredType::Dynamic))),
            Sigil::Hash => Some(InferredType::HashOf(
                Box::new(InferredType::String),
                Box::new(InferredType::Dynamic),
            )),
            Sigil::Scalar => {
                if line.contains("= \"") || line.contains("= '") {
                    Some(InferredType::String)
                } else if Regex::new(r#"=\s*\d+\s*;"#).unwrap().is_match(line) {
                    Some(InferredType::Integer)
                } else if Regex::new(r#"=\s*\d+\.\d+\s*;"#).unwrap().is_match(line) {
                    Some(InferredType::Float)
                } else {
                    None
                }
            }
        }
    }

    fn detect_oop_patterns(&self, source: &str) -> bool {
        source.contains("->new")
            || source.contains("use Moose")
            || source.contains("use Moo;")
            || source.contains("use Mouse")
            || source.contains("use Class::Accessor")
    }

    fn extract_parent_classes(&self, source: &str) -> Vec<String> {
        let mut parents = Vec::new();
        for caps in self.isa_re.captures_iter(source) {
            parents.push(caps[1].to_string());
        }
        parents
    }

    fn count_regexes(&self, source: &str) -> usize {
        let match_re = Regex::new(r#"=~\s*[/sm]"#).unwrap();
        let subst_re = Regex::new(r#"=~\s*s[/|]"#).unwrap();
        match_re.find_iter(source).count() + subst_re.find_iter(source).count()
    }
}

impl Default for PerlAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_simple_module() {
        let analyzer = PerlAnalyzer::new();
        let source = r#"
package MyModule;
use strict;
use warnings;
use JSON;

sub new {
    my ($class, %args) = @_;
    return bless { name => $args{name} }, $class;
}

sub greet {
    my ($self) = @_;
    return "Hello, " . $self->{name};
}

1;
"#;
        let module = analyzer
            .analyze_module(Path::new("MyModule.pm"), source)
            .unwrap();
        assert_eq!(module.package_name, "MyModule");
        assert_eq!(module.subroutines.len(), 2);
        assert!(module.is_oop);
        assert_eq!(module.uses.len(), 3);
        assert!(module
            .uses
            .iter()
            .any(|u| u.module_name == "JSON" && !u.is_pragma));
    }

    #[test]
    fn test_analyze_script() {
        let analyzer = PerlAnalyzer::new();
        let source = r#"#!/usr/bin/perl
use strict;
use warnings;
use LWP::UserAgent;

my $ua = LWP::UserAgent->new;
my $response = $ua->get("http://example.com");
print $response->content;
"#;
        let script = analyzer
            .analyze_script(Path::new("fetch.pl"), source)
            .unwrap();
        assert_eq!(script.uses.len(), 3);
        assert!(script
            .uses
            .iter()
            .any(|u| u.module_name == "LWP::UserAgent"));
    }

    #[test]
    fn test_analyze_project() {
        let tmp = TempDir::new().unwrap();
        let module_path = tmp.path().join("lib");
        std::fs::create_dir_all(&module_path).unwrap();

        std::fs::write(
            module_path.join("Dog.pm"),
            "package Dog;\nuse strict;\nsub new { my ($class, %args) = @_; bless { name => $args{name} }, $class }\nsub speak { return \"Woof!\" }\n1;\n",
        )
        .unwrap();

        std::fs::write(
            tmp.path().join("main.pl"),
            "#!/usr/bin/perl\nuse strict;\nuse lib 'lib';\nuse Dog;\nmy $d = Dog->new(name => \"Rex\");\nprint $d->speak();\n",
        )
        .unwrap();

        let analyzer = PerlAnalyzer::new();
        let project = analyzer.analyze_project(tmp.path()).unwrap();
        assert_eq!(project.modules.len(), 1);
        assert_eq!(project.scripts.len(), 1);

        let report = analyzer.generate_report(&project);
        assert_eq!(report.total_files, 2);
        assert_eq!(report.oop_modules, 1);
    }

    #[test]
    fn test_extract_variables() {
        let analyzer = PerlAnalyzer::new();
        let source = "my $name = \"Alice\";\nmy @items = (1, 2, 3);\nmy %config = (debug => 1);\nour $VERSION = 1.0;\n";
        let vars = analyzer.extract_variables(source);
        assert_eq!(vars.len(), 4);
        assert_eq!(vars[0].sigil, Sigil::Scalar);
        assert_eq!(vars[1].sigil, Sigil::Array);
        assert_eq!(vars[2].sigil, Sigil::Hash);
        assert_eq!(vars[3].scope, Scope::Our);
    }
}
