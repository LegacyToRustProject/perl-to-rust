struct Dog {
    name: String,
    breed: String,
}

impl Dog {
    fn new(name: String, breed: Option<String>) -> Self {
        Self {
            name,
            breed: breed.unwrap_or_else(|| "Mixed".to_string()),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn breed(&self) -> &str {
        &self.breed
    }

    fn speak(&self) -> String {
        format!("{} says: Woof!", self.name)
    }
}

fn main() {
    let dog = Dog::new("Rex".to_string(), Some("Labrador".to_string()));
    println!("{}", dog.name());
    println!("{}", dog.breed());
    println!("{}", dog.speak());

    let mutt = Dog::new("Buddy".to_string(), None);
    println!("{}", mutt.breed());
}
