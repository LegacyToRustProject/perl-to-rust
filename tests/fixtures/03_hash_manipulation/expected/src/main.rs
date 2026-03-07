use std::collections::HashMap;

fn main() {
    let mut colors: HashMap<&str, &str> = HashMap::new();
    colors.insert("red", "#FF0000");
    colors.insert("green", "#00FF00");
    colors.insert("blue", "#0000FF");

    let mut keys: Vec<&&str> = colors.keys().collect();
    keys.sort();
    for name in keys {
        println!("{}: {}", name, colors[*name]);
    }

    let count = colors.len();
    println!("Total colors: {}", count);

    if colors.contains_key("red") {
        println!("Red exists!");
    }

    colors.remove("green");
    println!("After delete: {} colors", colors.len());
}
