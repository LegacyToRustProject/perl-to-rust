// Converted from: test-projects/oo-sample/ (Animal.pm + Dog.pm + main.pl)
// Original Perl: bless-based OOP with use parent + SUPER:: dispatch
//
// Conversion map:
//   package Animal            → mod animal / struct Animal
//   package Dog; use parent   → struct Dog { animal: Animal, ... }  (composition)
//   sub new { bless {} }      → fn new() -> Self
//   $self->SUPER::new(%args)  → Animal::new(...)  (direct call, no super keyword)
//   $self->SUPER::describe()  → self.animal.describe()
//   sub speak { my ($self) }  → fn speak(&self)
//   sub learn_trick { push }  → fn learn_trick(&mut self) / Vec::push
//   @{$self->{tricks}}        → self.tricks: Vec<String>
//   $obj->isa('Animal')       → compile-time via trait bounds
//   use overload '""'         → impl Display

mod animal;
mod dog;

use animal::Animal;
use dog::Dog;

fn main() {
    // Perl: my $cat = Animal->new(name => 'Whiskers', sound => 'Meow', age => 3);
    let cat = Animal::new("Whiskers", "Meow", 3);
    cat.speak();        // Perl: $cat->speak;
    cat.describe();     // Perl: $cat->describe;
    println!("{cat}");  // Perl: print $cat->to_string, "\n"; (use overload '""')

    println!();

    // Perl: my $dog = Dog->new(name => 'Rex', age => 2);
    let mut dog = Dog::new("Rex", 2);
    dog.speak();    // Perl: $dog->speak;
    dog.describe(); // Perl: $dog->describe;

    // Perl: $dog->learn_trick('sit'); etc.
    dog.learn_trick("sit");
    dog.learn_trick("fetch");
    dog.learn_trick("roll over");
    dog.perform();  // Perl: $dog->perform;
    dog.describe(); // Perl: $dog->describe;

    println!("Rex's trick count: {}", dog.trick_count()); // Perl: $dog->trick_count

    // Perl: my @animals = ($cat, $dog);  for my $animal (@animals) { $animal->speak; }
    // With composition the animals share no common trait here — use explicit calls.
    // For polymorphism, see the trait-based approach in the OoPattern catalog.
    println!("\nAll animals:");
    cat.speak();
    dog.speak();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animal_speak_output() {
        let a = Animal::new("Cat", "Meow", 3);
        assert_eq!(a.name(), "Cat");
        assert_eq!(a.sound(), "Meow");
        assert_eq!(a.age(), 3);
    }

    #[test]
    fn test_animal_display() {
        let a = Animal::new("Tiger", "Roar", 5);
        assert_eq!(format!("{a}"), "Animal(Tiger)");
    }

    #[test]
    fn test_dog_inherits_animal_fields() {
        let d = Dog::new("Buddy", 1);
        assert_eq!(d.name(), "Buddy");
        assert_eq!(d.sound(), "Woof"); // default sound set by Dog::new
        assert_eq!(d.age(), 1);
    }

    #[test]
    fn test_dog_learn_trick() {
        let mut d = Dog::new("Rex", 2);
        assert_eq!(d.trick_count(), 0);
        d.learn_trick("sit");
        d.learn_trick("fetch");
        assert_eq!(d.trick_count(), 2);
    }

    #[test]
    fn test_dog_default_sound() {
        // Perl: $dog = Dog->new(name => 'Rex');  # sound defaults to 'Woof'
        let d = Dog::new("Rex", 0);
        assert_eq!(d.sound(), "Woof");
    }

    #[test]
    fn test_animal_age_default() {
        let a = Animal::with_defaults("Anon");
        assert_eq!(a.age(), 0);
        assert_eq!(a.sound(), "silence");
    }
}
