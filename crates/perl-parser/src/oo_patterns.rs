//! Perl OO → Rust struct/impl/trait conversion patterns.
//!
//! Covers the full spectrum of Perl OOP styles:
//! 1. `bless`-based (classic, most common)
//! 2. `Moose` / `Moo` / `Mouse` (attribute-centric)
//! 3. `Class::Accessor` / `Class::Tiny` (minimal)
//! 4. Role-based (`Moose::Role`, `Role::Tiny`)

use regex::Regex;

// ── Pattern catalog ───────────────────────────────────────────────────────────

/// A recognized Perl OO pattern with its idiomatic Rust equivalent.
#[derive(Debug, Clone, PartialEq)]
pub struct OoPattern {
    /// Short identifier.
    pub name: &'static str,
    /// Perl example code.
    pub perl_example: &'static str,
    /// Idiomatic Rust equivalent.
    pub rust_equivalent: &'static str,
    /// Notes on conversion caveats.
    pub notes: &'static str,
    /// Pattern category.
    pub category: OoCategory,
}

/// Top-level category for OO conversion patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OoCategory {
    /// `bless { ... }` constructor → `fn new() -> Self`
    Constructor,
    /// `use parent` / `@ISA` → trait implementation
    Inheritance,
    /// `$self->SUPER::method()` → delegate call
    SuperCall,
    /// `sub method { my ($self) = @_ }` → `fn method(&self)`
    Method,
    /// `has field => (is => 'rw', isa => 'Int')` → struct field
    Accessor,
    /// `use Moose::Role` / `with 'MyRole'` → Rust trait
    Role,
    /// `use overload '""' => \&to_string` → `impl Display`
    Overload,
    /// `$obj->isa('Foo')` / `$obj->can('bar')` → trait objects
    Introspection,
    /// `DESTROY` method → `impl Drop`
    Destructor,
}

/// Detected OO style of a Perl module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OoStyle {
    /// Classic `bless`-based OOP (Perl 5 default)
    Bless,
    /// `use Moose` (full MOP framework)
    Moose,
    /// `use Moo` (lighter Moose subset)
    Moo,
    /// `use Mouse` (faster Moose subset)
    Mouse,
    /// `use Class::Accessor` or `use Class::Accessor::Fast`
    ClassAccessor,
    /// `use Class::Tiny` (minimal)
    ClassTiny,
    /// Not detected as OO
    None,
}

/// A field extracted from Moose/Moo `has` declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct OoField {
    /// Field name.
    pub name: String,
    /// Whether the field is read-write (`is => 'rw'`) vs read-only (`is => 'ro'`).
    pub is_rw: bool,
    /// Perl type annotation (e.g. `'Str'`, `'Int'`, `'ArrayRef'`).
    pub perl_type: Option<String>,
    /// Inferred Rust type.
    pub rust_type: String,
    /// Default value expression (Perl source).
    pub default: Option<String>,
    /// Whether the field is required (`required => 1`).
    pub required: bool,
}

/// A method (sub) recognized as OO (first arg is `$self` or `$class`).
#[derive(Debug, Clone, PartialEq)]
pub struct OoMethod {
    /// Method name.
    pub name: String,
    /// Whether this is a constructor (`new` / `create` / `build`).
    pub is_constructor: bool,
    /// Whether this is a destructor (`DESTROY`).
    pub is_destructor: bool,
    /// Whether this method calls `SUPER::`.
    pub calls_super: bool,
    /// Detected parameter names (from `my ($self, ...) = @_`).
    pub parameters: Vec<String>,
}

/// Full OO class information extracted from a Perl source file.
#[derive(Debug, Clone)]
pub struct OoClassInfo {
    /// Package name (from `package Foo;`).
    pub class_name: String,
    /// Detected OO style.
    pub style: OoStyle,
    /// Parent class names (from `use parent`, `use base`, `@ISA`).
    pub parent_classes: Vec<String>,
    /// Consumed roles (from `with 'MyRole'`).
    pub roles: Vec<String>,
    /// Attributes / fields (Moose/Moo `has` declarations).
    pub fields: Vec<OoField>,
    /// Methods sorted: constructor first, then alphabetical.
    pub methods: Vec<OoMethod>,
    /// Whether any method overloads an operator (`use overload`).
    pub has_overloading: bool,
    /// Whether a `DESTROY` method exists.
    pub has_destructor: bool,
}

// ── Detector ──────────────────────────────────────────────────────────────────

/// Detects Perl OO patterns and extracts class metadata.
pub struct OoDetector {
    package_re: Regex,
    bless_re: Regex,
    parent_re: Regex,
    isa_re: Regex,
    super_re: Regex,
    moose_re: Regex,
    moo_re: Regex,
    mouse_re: Regex,
    accessor_re: Regex,
    tiny_re: Regex,
    has_re: Regex,
    role_with_re: Regex,
    overload_re: Regex,
    method_re: Regex,
    self_arg_re: Regex,
    constructor_names_re: Regex,
    parent_class_re: Regex,
    isa_list_re: Regex,
    role_name_re: Regex,
    has_field_re: Regex,
    has_type_re: Regex,
    has_default_re: Regex,
}

impl OoDetector {
    /// Create a new detector. Compiles all regexes once.
    pub fn new() -> Self {
        Self {
            package_re: Regex::new(r"(?m)^\s*package\s+([\w:]+)\s*;").unwrap(),
            bless_re: Regex::new(r"\bbless\s*\{").unwrap(),
            parent_re: Regex::new(r"\buse\s+(?:parent|base)\b").unwrap(),
            isa_re: Regex::new(r"@ISA\s*=").unwrap(),
            super_re: Regex::new(r"\bSUPER::").unwrap(),
            moose_re: Regex::new(r"\buse\s+Moose\b").unwrap(),
            moo_re: Regex::new(r"\buse\s+Moo\b").unwrap(), // \bMoo\b won't match "Moose"
            mouse_re: Regex::new(r"\buse\s+Mouse\b").unwrap(),
            accessor_re: Regex::new(r"\buse\s+Class::Accessor").unwrap(),
            tiny_re: Regex::new(r"\buse\s+Class::Tiny\b").unwrap(),
            has_re: Regex::new(r#"(?m)^\s*has\s+['"]?(\w+)['"]?\s*=>"#).unwrap(),
            role_with_re: Regex::new(r#"(?m)^\s*with\s+['"](\w[\w:]*)['"]\s*"#).unwrap(),
            overload_re: Regex::new(r"\buse\s+overload\b").unwrap(),
            method_re: Regex::new(r"(?m)^\s*sub\s+(\w+)\s*\{?").unwrap(),
            self_arg_re: Regex::new(r"my\s*\(\s*([\$\w,\s]+)\)\s*=\s*@_").unwrap(),
            constructor_names_re: Regex::new(r"^(?:new|create|build|make|instance)$").unwrap(),
            parent_class_re: Regex::new(
                r#"\buse\s+(?:parent|base)\s+(?:qw\(([^)]*)\)|['"]?([\w:]+)['"]?)"#,
            )
            .unwrap(),
            isa_list_re: Regex::new(r#"@ISA\s*=\s*\(([^)]*)\)"#).unwrap(),
            role_name_re: Regex::new(r#"(?m)^\s*with\s+['"]?([\w:]+)['"]?"#).unwrap(),
            has_field_re: Regex::new(
                r#"(?m)^\s*has\s+['"]?(\w+)['"]?\s*=>\s*\(([^)]*)\)"#,
            )
            .unwrap(),
            has_type_re: Regex::new(r#"isa\s*=>\s*['"]([^'"]+)['"]"#).unwrap(),
            has_default_re: Regex::new(
                r#"default\s*=>\s*(?:sub\s*\{[^}]*\}|'([^']*)'|"([^"]*)"|\d+)"#,
            )
            .unwrap(),
        }
    }

    /// Count OO-related pattern occurrences in source (for heuristics).
    pub fn counts(&self, source: &str) -> OoCounts {
        OoCounts {
            bless: self.bless_re.find_iter(source).count(),
            parent: self.parent_re.find_iter(source).count()
                + self.isa_re.find_iter(source).count(),
            super_call: self.super_re.find_iter(source).count(),
            moose: self.moose_re.find_iter(source).count(),
            moo: self.moo_re.find_iter(source).count(),
            mouse: self.mouse_re.find_iter(source).count(),
            accessor: self.accessor_re.find_iter(source).count(),
            has_attr: self.has_re.captures_iter(source).count(),
            role_with: self.role_with_re.captures_iter(source).count(),
            overload: self.overload_re.find_iter(source).count(),
        }
    }

    /// Detect the primary OO style used in the source.
    pub fn detect_style(&self, source: &str) -> OoStyle {
        if self.moose_re.is_match(source) {
            return OoStyle::Moose;
        }
        if self.moo_re.is_match(source) {
            return OoStyle::Moo;
        }
        if self.mouse_re.is_match(source) {
            return OoStyle::Mouse;
        }
        if self.accessor_re.is_match(source) {
            return OoStyle::ClassAccessor;
        }
        if self.tiny_re.is_match(source) {
            return OoStyle::ClassTiny;
        }
        if self.bless_re.is_match(source) {
            return OoStyle::Bless;
        }
        OoStyle::None
    }

    /// Extract complete OO class information from a single-package source file.
    pub fn extract_class_info(&self, source: &str) -> Option<OoClassInfo> {
        let class_name = self
            .package_re
            .captures(source)
            .map(|c| c[1].to_string())?;

        let style = self.detect_style(source);
        let parent_classes = self.extract_parents(source);
        let roles = self.extract_roles(source);
        let fields = self.extract_fields(source);
        let methods = self.extract_methods(source);
        let has_overloading = self.overload_re.is_match(source);
        let has_destructor = methods.iter().any(|m| m.is_destructor);

        Some(OoClassInfo {
            class_name,
            style,
            parent_classes,
            roles,
            fields,
            methods,
            has_overloading,
            has_destructor,
        })
    }

    fn extract_parents(&self, source: &str) -> Vec<String> {
        let mut parents = Vec::new();

        // use parent qw(Foo Bar) or use parent 'Foo'
        for caps in self.parent_class_re.captures_iter(source) {
            if let Some(qw) = caps.get(1) {
                for p in qw.as_str().split_whitespace() {
                    let cleaned = p.trim_matches(|c| c == '\'' || c == '"');
                    if !cleaned.is_empty() {
                        parents.push(cleaned.to_string());
                    }
                }
            } else if let Some(single) = caps.get(2) {
                parents.push(single.as_str().to_string());
            }
        }

        // @ISA = ('Foo', 'Bar')
        for caps in self.isa_list_re.captures_iter(source) {
            for p in caps[1].split(',') {
                let cleaned = p.trim().trim_matches(|c| c == '\'' || c == '"');
                if !cleaned.is_empty() && !parents.contains(&cleaned.to_string()) {
                    parents.push(cleaned.to_string());
                }
            }
        }

        parents
    }

    fn extract_roles(&self, source: &str) -> Vec<String> {
        let mut roles = Vec::new();
        for caps in self.role_name_re.captures_iter(source) {
            roles.push(caps[1].to_string());
        }
        roles
    }

    fn extract_fields(&self, source: &str) -> Vec<OoField> {
        let mut fields = Vec::new();
        for caps in self.has_field_re.captures_iter(source) {
            let name = caps[1].to_string();
            let attrs = &caps[2];
            let is_rw = attrs.contains("'rw'") || attrs.contains("\"rw\"");
            let required = attrs.contains("required") && attrs.contains('1');

            let perl_type = self
                .has_type_re
                .captures(attrs)
                .map(|c| c[1].to_string());

            let rust_type = perl_type_to_rust(perl_type.as_deref());

            let default = self.has_default_re.captures(attrs).map(|c| {
                c.get(1)
                    .or(c.get(2))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "sub { ... }".to_string())
            });

            fields.push(OoField {
                name,
                is_rw,
                perl_type,
                rust_type,
                default,
                required,
            });
        }
        fields
    }

    fn extract_methods(&self, source: &str) -> Vec<OoMethod> {
        let mut methods = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let Some(caps) = self.method_re.captures(line) else {
                continue;
            };
            let name = caps[1].to_string();

            // Collect body (next ~20 lines heuristic, or until balanced brace)
            let body_end = (i + 1 + 20).min(lines.len());
            let body: String = lines[i..body_end].join("\n");

            let is_constructor = self.constructor_names_re.is_match(&name);
            let is_destructor = name == "DESTROY";
            let calls_super = self.super_re.is_match(&body);

            let parameters = if let Some(pcaps) = self.self_arg_re.captures(&body) {
                pcaps[1]
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .collect()
            } else if body.contains("my $self = shift") || body.contains("my ($self)") {
                vec!["$self".to_string()]
            } else {
                vec![]
            };

            methods.push(OoMethod {
                name,
                is_constructor,
                is_destructor,
                calls_super,
                parameters,
            });
        }

        methods
    }
}

impl Default for OoDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Counts of OO pattern occurrences in a source file.
#[derive(Debug, Clone, Default)]
pub struct OoCounts {
    pub bless: usize,
    pub parent: usize,
    pub super_call: usize,
    pub moose: usize,
    pub moo: usize,
    pub mouse: usize,
    pub accessor: usize,
    pub has_attr: usize,
    pub role_with: usize,
    pub overload: usize,
}

impl OoCounts {
    /// Returns true if any OO usage is detected.
    pub fn is_oop(&self) -> bool {
        self.bless > 0
            || self.parent > 0
            || self.moose > 0
            || self.moo > 0
            || self.mouse > 0
            || self.accessor > 0
    }

    /// Returns a complexity score (0–100) for OO conversion difficulty.
    pub fn complexity_score(&self) -> u32 {
        let mut score = 0u32;
        score += (self.super_call * 5) as u32;
        score += (self.moose * 10) as u32;
        score += (self.moo * 8) as u32;
        score += (self.has_attr * 3) as u32;
        score += (self.role_with * 7) as u32;
        score += (self.overload * 8) as u32;
        score.min(100)
    }
}

// ── Perl type → Rust type mapping ─────────────────────────────────────────────

/// Convert a Perl/Moose type name to an approximate Rust type.
pub fn perl_type_to_rust(perl_type: Option<&str>) -> String {
    match perl_type {
        None => "Option<String>".to_string(),
        Some(t) => match t {
            "Str" | "Scalar" => "String".to_string(),
            "Int" | "Integer" => "i64".to_string(),
            "Num" | "Float" => "f64".to_string(),
            "Bool" => "bool".to_string(),
            "ArrayRef" => "Vec<String>".to_string(),
            "HashRef" => "HashMap<String, String>".to_string(),
            "CodeRef" => "Box<dyn Fn()>".to_string(),
            "Maybe[Str]" => "Option<String>".to_string(),
            "Maybe[Int]" => "Option<i64>".to_string(),
            "PositiveInt" | "PositiveNum" => "u64".to_string(),
            _ if t.starts_with("ArrayRef[") => {
                let inner = &t[9..t.len() - 1];
                format!("Vec<{}>", perl_type_to_rust(Some(inner)))
            }
            _ if t.starts_with("HashRef[") => {
                let inner = &t[8..t.len() - 1];
                format!("HashMap<String, {}>", perl_type_to_rust(Some(inner)))
            }
            _ if t.starts_with("Maybe[") => {
                let inner = &t[6..t.len() - 1];
                format!("Option<{}>", perl_type_to_rust(Some(inner)))
            }
            // Assume it's a custom class name → Box<dyn Trait> or concrete struct
            other => format!("Box<{}>", other),
        },
    }
}

// ── Pattern catalog ────────────────────────────────────────────────────────────

/// Returns the full OO conversion pattern library.
pub fn all_patterns() -> Vec<OoPattern> {
    vec![
        // ── CONSTRUCTOR ──────────────────────────────────────────────
        OoPattern {
            name: "bless_constructor",
            perl_example: r#"sub new {
    my ($class, %args) = @_;
    return bless { name => $args{name} }, $class;
}"#,
            rust_equivalent: r#"pub fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
}"#,
            notes: "bless { ... } → struct literal. $class implicit via Self.",
            category: OoCategory::Constructor,
        },
        OoPattern {
            name: "bless_with_defaults",
            perl_example: r#"sub new {
    my ($class, %args) = @_;
    return bless {
        name  => $args{name} // 'default',
        count => $args{count} // 0,
    }, $class;
}"#,
            rust_equivalent: r#"pub fn new(name: Option<String>, count: Option<i64>) -> Self {
    Self {
        name:  name.unwrap_or_else(|| "default".to_string()),
        count: count.unwrap_or(0),
    }
}"#,
            notes: "`$args{key} // default` → `Option::unwrap_or` or builder pattern.",
            category: OoCategory::Constructor,
        },
        OoPattern {
            name: "moose_constructor",
            perl_example: r#"use Moose;
has name  => (is => 'ro', isa => 'Str', required => 1);
has count => (is => 'rw', isa => 'Int', default  => 0);"#,
            rust_equivalent: r#"#[derive(Debug)]
pub struct MyClass {
    pub name: String,   // ro: no setter
    pub count: i64,     // rw: pub field or getter+setter
}

impl MyClass {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), count: 0 }
    }
    pub fn count(&self) -> i64 { self.count }
    pub fn set_count(&mut self, v: i64) { self.count = v; }
}"#,
            notes: "Moose `has` → struct fields. `is => 'ro'` → no setter. `is => 'rw'` → getter + setter.",
            category: OoCategory::Accessor,
        },
        // ── INHERITANCE ──────────────────────────────────────────────
        OoPattern {
            name: "use_parent",
            perl_example: r#"use parent 'Animal';  # Dog ISA Animal"#,
            rust_equivalent: r#"// Option A: composition (preferred for single inheritance)
pub struct Dog {
    animal: Animal,   // embed parent struct
    tricks: Vec<String>,
}

// Option B: trait (preferred for polymorphism)
pub trait Animal {
    fn name(&self) -> &str;
    fn speak(&self);
}
impl Animal for Dog { ... }"#,
            notes: "Perl inheritance → Rust composition (embed parent) or trait polymorphism. There is no direct inheritance in Rust.",
            category: OoCategory::Inheritance,
        },
        OoPattern {
            name: "isa_array",
            perl_example: r#"our @ISA = ('Base', 'Mixin');"#,
            rust_equivalent: r#"// Multiple inheritance → multiple trait impls
pub trait Base { fn base_method(&self); }
pub trait Mixin { fn mixin_method(&self); }
impl Base for MyStruct { fn base_method(&self) { ... } }
impl Mixin for MyStruct { fn mixin_method(&self) { ... } }"#,
            notes: "@ISA with multiple parents → each parent becomes a trait. Shared state → explicit fields.",
            category: OoCategory::Inheritance,
        },
        OoPattern {
            name: "role_with",
            perl_example: r#"use Moose;
with 'Printable', 'Serializable';"#,
            rust_equivalent: r#"pub trait Printable { fn print(&self); }
pub trait Serializable { fn serialize(&self) -> String; }

pub struct MyClass { ... }
impl Printable for MyClass { fn print(&self) { println!("{:?}", self); } }
impl Serializable for MyClass { fn serialize(&self) -> String { ... } }"#,
            notes: "Moose Role → Rust trait. `with` → `impl Trait for Struct`.",
            category: OoCategory::Role,
        },
        // ── SUPER:: ──────────────────────────────────────────────────
        OoPattern {
            name: "super_new",
            perl_example: r#"sub new {
    my ($class, %args) = @_;
    my $self = $class->SUPER::new(%args);
    $self->{extra} = 'value';
    return $self;
}"#,
            rust_equivalent: r#"pub fn new(/* args */) -> Self {
    // With composition: initialize parent struct directly
    let animal = Animal::new(/* args */);
    Self { animal, extra: "value".to_string() }
    // With trait: no SUPER::, just call helper directly
}"#,
            notes: "SUPER::new → initialize embedded parent struct or call parent fn directly. No super() in Rust.",
            category: OoCategory::SuperCall,
        },
        OoPattern {
            name: "super_method_override",
            perl_example: r#"sub describe {
    my ($self) = @_;
    $self->SUPER::describe();    # call parent first
    printf "Extra: %s\n", $self->{extra};
}"#,
            rust_equivalent: r#"pub fn describe(&self) {
    self.animal.describe();       // call parent struct's method
    println!("Extra: {}", self.extra);
}"#,
            notes: "SUPER::method → call the method on the embedded parent struct field.",
            category: OoCategory::SuperCall,
        },
        // ── METHODS ──────────────────────────────────────────────────
        OoPattern {
            name: "instance_method",
            perl_example: r#"sub speak {
    my ($self) = @_;
    printf "%s says %s\n", $self->name, $self->sound;
}"#,
            rust_equivalent: r#"pub fn speak(&self) {
    println!("{} says {}", self.name, self.sound);
}"#,
            notes: "`my ($self) = @_` → `&self`. Dereference `$self->{field}` → `self.field`.",
            category: OoCategory::Method,
        },
        OoPattern {
            name: "mutating_method",
            perl_example: r#"sub learn_trick {
    my ($self, $trick) = @_;
    push @{$self->{tricks}}, $trick;
}"#,
            rust_equivalent: r#"pub fn learn_trick(&mut self, trick: impl Into<String>) {
    self.tricks.push(trick.into());
}"#,
            notes: "Methods that modify `$self` → `&mut self`.",
            category: OoCategory::Method,
        },
        OoPattern {
            name: "class_method",
            perl_example: r#"sub class_name {
    my ($class) = @_;
    return ref($class) || $class;
}"#,
            rust_equivalent: r#"pub fn class_name() -> &'static str {
    std::any::type_name::<Self>()
}"#,
            notes: "`my ($class) = @_` class methods → associated functions `fn foo()` with no `self`.",
            category: OoCategory::Method,
        },
        OoPattern {
            name: "read_write_accessor",
            perl_example: r#"sub age {
    my ($self, $val) = @_;
    $self->{_age} = $val if @_ > 1;
    return $self->{_age};
}"#,
            rust_equivalent: r#"pub fn age(&self) -> u32 { self.age }
pub fn set_age(&mut self, val: u32) { self.age = val; }"#,
            notes: "Perl dual-purpose accessor → separate Rust getter + setter for clarity.",
            category: OoCategory::Accessor,
        },
        // ── OVERLOADING ──────────────────────────────────────────────
        OoPattern {
            name: "overload_stringify",
            perl_example: r#"use overload '""' => \&to_string;
sub to_string { return "MyClass(" . $_[0]->{name} . ")" }"#,
            rust_equivalent: r#"use std::fmt;
impl fmt::Display for MyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyClass({})", self.name)
    }
}"#,
            notes: "`use overload '\"\"'` → `impl std::fmt::Display`.",
            category: OoCategory::Overload,
        },
        OoPattern {
            name: "overload_arithmetic",
            perl_example: r#"use overload '+' => \&add, '-' => \&subtract;"#,
            rust_equivalent: r#"use std::ops::{Add, Sub};
impl Add for MyClass {
    type Output = MyClass;
    fn add(self, rhs: MyClass) -> MyClass { ... }
}
impl Sub for MyClass { ... }"#,
            notes: "`use overload '+'` → `impl std::ops::Add`.",
            category: OoCategory::Overload,
        },
        OoPattern {
            name: "overload_comparison",
            perl_example: r#"use overload '==' => \&equal, '<=>' => \&compare;"#,
            rust_equivalent: r#"impl PartialEq for MyClass {
    fn eq(&self, other: &Self) -> bool { ... }
}
impl PartialOrd for MyClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { ... }
}"#,
            notes: "`==` → `PartialEq`, `<=>` → `PartialOrd`.",
            category: OoCategory::Overload,
        },
        // ── DESTRUCTOR ───────────────────────────────────────────────
        OoPattern {
            name: "destroy_method",
            perl_example: r#"sub DESTROY {
    my ($self) = @_;
    # cleanup: close file handle, release resource
    $self->{fh}->close if $self->{fh};
}"#,
            rust_equivalent: r#"impl Drop for MyClass {
    fn drop(&mut self) {
        // RAII: cleanup runs automatically when value goes out of scope
        // File handles etc. implement Drop themselves — often no code needed
        if let Some(ref mut fh) = self.fh {
            let _ = fh.flush();
        }
    }
}"#,
            notes: "Perl `DESTROY` → Rust `impl Drop`. Most resource cleanup is automatic in Rust via RAII.",
            category: OoCategory::Destructor,
        },
        // ── INTROSPECTION ────────────────────────────────────────────
        OoPattern {
            name: "isa_check",
            perl_example: r#"if ($obj->isa('Animal')) { ... }"#,
            rust_equivalent: r#"// With trait objects:
if let Some(animal) = obj.as_any().downcast_ref::<Animal>() { ... }
// Or use is<T>() from the 'downcast' crate
// Or redesign using enum variants for closed type sets"#,
            notes: "`isa()` runtime type check → Rust `Any::downcast_ref` or enum matching.",
            category: OoCategory::Introspection,
        },
        OoPattern {
            name: "can_check",
            perl_example: r#"if ($obj->can('speak')) { $obj->speak(); }"#,
            rust_equivalent: r#"// Use trait bounds: if T: Speaks, the method is guaranteed to exist
fn call_if_speaks<T: Speaks>(obj: &T) { obj.speak(); }
// Or trait objects: Box<dyn Speaks>"#,
            notes: "`can()` → Rust trait bounds guarantee method existence at compile time.",
            category: OoCategory::Introspection,
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn detector() -> OoDetector {
        OoDetector::new()
    }

    // ── OoStyle detection ─────────────────────────────────────────────────────

    #[test]
    fn test_detect_bless_style() {
        let src = "package Foo;\nsub new { bless { x => 1 }, shift }\n1;";
        assert_eq!(detector().detect_style(src), OoStyle::Bless);
    }

    #[test]
    fn test_detect_moose_style() {
        let src = "package Foo;\nuse Moose;\nhas name => (is => 'ro', isa => 'Str');\n1;";
        assert_eq!(detector().detect_style(src), OoStyle::Moose);
    }

    #[test]
    fn test_detect_moo_style() {
        let src = "package Foo;\nuse Moo;\nhas count => (is => 'rw', default => 0);\n1;";
        assert_eq!(detector().detect_style(src), OoStyle::Moo);
    }

    #[test]
    fn test_detect_mouse_style() {
        let src = "package Foo;\nuse Mouse;\nhas x => (is => 'ro');\n1;";
        assert_eq!(detector().detect_style(src), OoStyle::Mouse);
    }

    #[test]
    fn test_detect_class_accessor() {
        let src = "package Foo;\nuse Class::Accessor::Fast;\nFoo->mk_accessors(qw(name age));\n1;";
        assert_eq!(detector().detect_style(src), OoStyle::ClassAccessor);
    }

    #[test]
    fn test_detect_none_style() {
        let src = "#!/usr/bin/perl\nuse strict;\nprint \"hello\\n\";\n";
        assert_eq!(detector().detect_style(src), OoStyle::None);
    }

    // ── counts ────────────────────────────────────────────────────────────────

    #[test]
    fn test_counts_bless() {
        let src = "bless { a => 1 }, $class;\nbless { b => 2 }, $class;";
        let c = detector().counts(src);
        assert_eq!(c.bless, 2);
        assert!(c.is_oop());
    }

    #[test]
    fn test_counts_super() {
        let src = "$self->SUPER::new();\n$self->SUPER::describe();";
        let c = detector().counts(src);
        assert_eq!(c.super_call, 2);
    }

    #[test]
    fn test_counts_has_attrs() {
        let src =
            "has name => (is => 'ro', isa => 'Str');\nhas age => (is => 'rw', default => 0);\n";
        let c = detector().counts(src);
        assert_eq!(c.has_attr, 2);
    }

    #[test]
    fn test_counts_not_oop() {
        let src = "my $x = 1;\nprint $x;\n";
        assert!(!detector().counts(src).is_oop());
    }

    // ── complexity_score ──────────────────────────────────────────────────────

    #[test]
    fn test_complexity_score_simple() {
        let src = "bless { x => 1 }, $class;\n";
        let score = detector().counts(src).complexity_score();
        assert_eq!(score, 0); // bless alone contributes 0 extra score
    }

    #[test]
    fn test_complexity_score_moose_heavy() {
        // use Moose + 5 has + 2 with + 1 overload + 3 SUPER
        let src = "use Moose;\nhas a => (is=>'ro');\nhas b => (is=>'ro');\nhas c => (is=>'ro');\nhas d=>(is=>'ro');\nhas e=>(is=>'ro');\nwith 'RoleA';\nwith 'RoleB';\nuse overload '\"\"' => \\&str;\nSUPER::x;\nSUPER::y;\nSUPER::z;\n";
        let score = detector().counts(src).complexity_score();
        // moose=10 + 5*has=15 + 2*role=14 + overload=8 + 3*super=15 = 62
        assert!(score >= 50, "score={score}");
    }

    #[test]
    fn test_complexity_capped_at_100() {
        // Extreme case
        let mut src = String::new();
        for _ in 0..20 {
            src.push_str("use Moose;\nwith 'Role';\nuse overload '+' => \\&add;\nSUPER::x;\nhas f => (is=>'rw');\n");
        }
        let score = detector().counts(&src).complexity_score();
        assert_eq!(score, 100);
    }

    // ── extract_class_info ────────────────────────────────────────────────────

    #[test]
    fn test_extract_basic_class() {
        let src = r#"
package Animal;
use strict;
sub new {
    my ($class, %args) = @_;
    return bless { name => $args{name}, sound => $args{sound} // 'silence' }, $class;
}
sub name  { return $_[0]->{name}  }
sub speak {
    my ($self) = @_;
    printf "%s says %s\n", $self->name, $self->sound;
}
1;
"#;
        let info = detector().extract_class_info(src).unwrap();
        assert_eq!(info.class_name, "Animal");
        assert_eq!(info.style, OoStyle::Bless);
        assert!(info.parent_classes.is_empty());
        assert!(info.methods.iter().any(|m| m.name == "speak"));
        assert!(info.methods.iter().any(|m| m.name == "new" && m.is_constructor));
    }

    #[test]
    fn test_extract_parent_use_parent() {
        let src = "package Dog;\nuse parent 'Animal';\nsub new { bless {}, shift }\n1;";
        let info = detector().extract_class_info(src).unwrap();
        assert_eq!(info.parent_classes, vec!["Animal"]);
    }

    #[test]
    fn test_extract_parent_isa() {
        let src = "package Dog;\nour @ISA = ('Animal', 'Pet');\nsub bark { 1 }\n1;";
        let info = detector().extract_class_info(src).unwrap();
        assert!(info.parent_classes.contains(&"Animal".to_string()));
        assert!(info.parent_classes.contains(&"Pet".to_string()));
    }

    #[test]
    fn test_extract_super_call() {
        let src = r#"
package Dog;
use parent 'Animal';
sub new {
    my ($class, %args) = @_;
    my $self = $class->SUPER::new(%args);
    return $self;
}
1;
"#;
        let info = detector().extract_class_info(src).unwrap();
        let new_method = info.methods.iter().find(|m| m.name == "new").unwrap();
        assert!(new_method.calls_super);
    }

    #[test]
    fn test_extract_moose_fields() {
        let src = r#"
package Person;
use Moose;
has name  => (is => 'ro', isa => 'Str', required => 1);
has age   => (is => 'rw', isa => 'Int', default  => 0);
has email => (is => 'rw', isa => 'Maybe[Str]');
1;
"#;
        let info = detector().extract_class_info(src).unwrap();
        assert_eq!(info.style, OoStyle::Moose);
        assert_eq!(info.fields.len(), 3);

        let name_field = info.fields.iter().find(|f| f.name == "name").unwrap();
        assert!(!name_field.is_rw);
        assert!(name_field.required);
        assert_eq!(name_field.rust_type, "String");

        let age_field = info.fields.iter().find(|f| f.name == "age").unwrap();
        assert!(age_field.is_rw);
        assert_eq!(age_field.rust_type, "i64");
    }

    #[test]
    fn test_extract_roles() {
        let src =
            "package Foo;\nuse Moose;\nwith 'Printable';\nwith 'Serializable';\nhas x => (is=>'ro');\n1;";
        let info = detector().extract_class_info(src).unwrap();
        assert!(info.roles.contains(&"Printable".to_string()));
        assert!(info.roles.contains(&"Serializable".to_string()));
    }

    #[test]
    fn test_extract_overload() {
        let src = "package Foo;\nuse overload '\"\"' => \\&to_string;\nsub to_string { \"Foo\" }\n1;";
        let info = detector().extract_class_info(src).unwrap();
        assert!(info.has_overloading);
    }

    #[test]
    fn test_extract_destructor() {
        let src = "package Foo;\nsub new { bless {}, shift }\nsub DESTROY { print \"bye\\n\" }\n1;";
        let info = detector().extract_class_info(src).unwrap();
        assert!(info.has_destructor);
        assert!(info.methods.iter().any(|m| m.is_destructor));
    }

    #[test]
    fn test_no_package_returns_none() {
        let src = "# no package statement\nmy $x = 1;\n";
        assert!(detector().extract_class_info(src).is_none());
    }

    // ── perl_type_to_rust ─────────────────────────────────────────────────────

    #[test]
    fn test_type_str() {
        assert_eq!(perl_type_to_rust(Some("Str")), "String");
    }

    #[test]
    fn test_type_int() {
        assert_eq!(perl_type_to_rust(Some("Int")), "i64");
    }

    #[test]
    fn test_type_bool() {
        assert_eq!(perl_type_to_rust(Some("Bool")), "bool");
    }

    #[test]
    fn test_type_arrayref() {
        assert_eq!(perl_type_to_rust(Some("ArrayRef")), "Vec<String>");
    }

    #[test]
    fn test_type_arrayref_parameterized() {
        assert_eq!(perl_type_to_rust(Some("ArrayRef[Int]")), "Vec<i64>");
    }

    #[test]
    fn test_type_maybe() {
        assert_eq!(perl_type_to_rust(Some("Maybe[Str]")), "Option<String>");
    }

    #[test]
    fn test_type_none() {
        assert_eq!(perl_type_to_rust(None), "Option<String>");
    }

    #[test]
    fn test_type_custom_class() {
        assert_eq!(perl_type_to_rust(Some("Animal")), "Box<Animal>");
    }

    // ── pattern catalog ───────────────────────────────────────────────────────

    #[test]
    fn test_all_patterns_nonempty() {
        let patterns = all_patterns();
        assert!(!patterns.is_empty(), "Pattern catalog must not be empty");
        assert!(patterns.len() >= 15, "Expected at least 15 patterns, got {}", patterns.len());
    }

    #[test]
    fn test_patterns_have_all_categories() {
        let patterns = all_patterns();
        let categories = [
            OoCategory::Constructor,
            OoCategory::Inheritance,
            OoCategory::SuperCall,
            OoCategory::Method,
            OoCategory::Accessor,
            OoCategory::Role,
            OoCategory::Overload,
            OoCategory::Destructor,
            OoCategory::Introspection,
        ];
        for cat in &categories {
            assert!(
                patterns.iter().any(|p| &p.category == cat),
                "Missing category: {:?}",
                cat
            );
        }
    }

    #[test]
    fn test_pattern_names_unique() {
        let patterns = all_patterns();
        let mut names = std::collections::HashSet::new();
        for p in &patterns {
            assert!(names.insert(p.name), "Duplicate pattern name: {}", p.name);
        }
    }
}
