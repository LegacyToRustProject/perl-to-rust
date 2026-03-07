use regex::Regex;

fn main() {
    let items = vec!["apple", "banana", "cherry", "date"];

    // $_ in for loop — explicit variable in Rust
    let re = Regex::new(r"^[abc]").unwrap();
    for item in &items {
        if re.is_match(item) {
            println!("{}", item);
        }
    }

    // $_ in map — explicit closure parameter
    let upper: Vec<String> = items.iter().map(|item| item.to_uppercase()).collect();
    println!("{}", upper.join(", "));

    // $_ in grep — explicit closure parameter
    let long: Vec<&&str> = items.iter().filter(|item| item.len() > 5).collect();
    println!(
        "Long items: {}",
        long.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
