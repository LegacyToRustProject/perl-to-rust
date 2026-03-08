// Converted from: test-projects/oo-sample/lib/Animal.pm
//
// Perl OOP pattern → Rust conversion:
//   package Animal             → struct Animal (mod animal)
//   bless { name, sound, age } → struct fields
//   sub new { bless {}, $class }   → fn new() -> Self
//   sub name  { $_[0]->{name}  }   → fn name(&self) -> &str
//   sub age   { rw accessor    }   → fn age() + fn set_age(&mut self)
//   sub speak { printf ...     }   → fn speak(&self) { println!(...) }
//   use overload '""' => \&str_fn  → impl std::fmt::Display

use std::fmt;

/// Converted from `package Animal` (bless-based OOP).
///
/// Perl fields (`name`, `sound`, `_age`) become public struct fields.
/// Encapsulation is achieved by making them private and exposing methods.
#[derive(Debug, Clone)]
pub struct Animal {
    name: String,
    sound: String,
    age: u32,
}

impl Animal {
    /// Perl: `Animal->new(name => ..., sound => ..., age => ...)`
    /// bless { name => $args{name}, sound => $args{sound}, _age => $args{age} }, $class
    pub fn new(name: impl Into<String>, sound: impl Into<String>, age: u32) -> Self {
        Self {
            name: name.into(),
            sound: sound.into(),
            age,
        }
    }

    /// Perl: `Animal->new(name => ...)` with all optional args defaulted.
    /// `$args{sound} // 'silence'`, `$args{age} // 0`
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, "silence", 0)
    }

    // ── Read-only accessors ────────────────────────────────────────────────────
    // Perl: sub name  { return $_[0]->{name}  }
    pub fn name(&self) -> &str {
        &self.name
    }

    // Perl: sub sound { return $_[0]->{sound} }
    pub fn sound(&self) -> &str {
        &self.sound
    }

    // ── Read-write accessor ────────────────────────────────────────────────────
    // Perl dual-purpose: $obj->age()  OR  $obj->age(5)
    // Rust: separate getter + setter
    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn set_age(&mut self, val: u32) {
        self.age = val;
    }

    // ── Instance methods ───────────────────────────────────────────────────────
    // Perl: sub speak { my ($self) = @_; printf "%s says %s\n", $self->name, $self->sound; }
    pub fn speak(&self) {
        println!("{} says {}", self.name, self.sound);
    }

    // Perl: sub describe { my ($self) = @_; printf "I am %s, age %d\n", $self->name, $self->age; }
    pub fn describe(&self) {
        println!("I am {}, age {}", self.name, self.age);
    }
}

// Perl: use overload '""' => \&to_string;
// sub to_string { sprintf "Animal(%s)", $_[0]->{name} }
impl fmt::Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Animal({})", self.name)
    }
}
