// Converted from: test-projects/oo-sample/lib/Dog.pm
//
// Perl OOP pattern → Rust conversion:
//   package Dog; use parent 'Animal'   → struct Dog { animal: Animal, ... }
//                                         (composition replaces inheritance)
//   $class->SUPER::new(%args)          → Animal::new(...)  (direct call)
//   $self->SUPER::describe()           → self.animal.describe()
//   push @{$self->{tricks}}, $trick    → self.tricks.push(trick)
//   scalar @{$self->{tricks}}          → self.tricks.len()
//   join(', ', @{$self->{tricks}})     → self.tricks.join(", ")
//
// Design note: Perl's `use parent` (single inheritance) is modelled here with
// *composition*: Dog embeds an Animal struct rather than using a trait.
// This is idiomatic Rust for a small, concrete hierarchy.
// For polymorphism (Box<dyn Animal>) use a trait instead — see OoPattern catalog.

use crate::animal::Animal;

/// Converted from `package Dog` (inherits Animal via `use parent`).
///
/// Rust uses *composition*: `Dog` embeds an `Animal` value.
/// Delegation methods (`name`, `sound`, `age`, `speak`) forward to `self.animal`.
/// SUPER:: calls become `self.animal.method()`.
#[derive(Debug, Clone)]
pub struct Dog {
    /// Embedded parent — replaces Perl's `use parent 'Animal'`.
    animal: Animal,
    /// `$self->{tricks}` → `Vec<String>`.
    tricks: Vec<String>,
}

impl Dog {
    /// Perl: `Dog->new(name => ..., age => ...)`
    /// `$args{sound} //= 'Woof';  my $self = $class->SUPER::new(%args);`
    pub fn new(name: impl Into<String>, age: u32) -> Self {
        // SUPER::new → construct the parent struct directly
        let animal = Animal::new(name, "Woof", age);
        Self {
            animal,
            tricks: Vec::new(),
        }
    }

    // ── Delegated accessors (forward to embedded Animal) ──────────────────────
    // These replace Perl's method dispatch which walked up @ISA automatically.
    pub fn name(&self) -> &str {
        self.animal.name()
    }

    pub fn sound(&self) -> &str {
        self.animal.sound()
    }

    pub fn age(&self) -> u32 {
        self.animal.age()
    }

    // ── Delegated methods ─────────────────────────────────────────────────────
    // Perl: $dog->speak → walks @ISA, finds speak in Animal
    pub fn speak(&self) {
        self.animal.speak()
    }

    // ── New methods specific to Dog ───────────────────────────────────────────
    // Perl: sub learn_trick { my ($self, $trick) = @_; push @{$self->{tricks}}, $trick; }
    pub fn learn_trick(&mut self, trick: impl Into<String>) {
        self.tricks.push(trick.into());
    }

    // Perl: sub perform { ... if @{$self->{tricks}} ... join(', ', ...) }
    pub fn perform(&self) {
        if self.tricks.is_empty() {
            println!("{} knows no tricks yet", self.name());
        } else {
            println!("{} performs: {}", self.name(), self.tricks.join(", "));
        }
    }

    // Perl: sub trick_count { return scalar @{$_[0]->{tricks}} }
    pub fn trick_count(&self) -> usize {
        self.tricks.len()
    }

    // ── Overridden method (calls SUPER) ───────────────────────────────────────
    // Perl: sub describe { $self->SUPER::describe(); printf "Tricks: %d\n", ... }
    pub fn describe(&self) {
        self.animal.describe(); // SUPER::describe() → forward to parent struct
        println!("Tricks learned: {}", self.tricks.len());
    }
}
