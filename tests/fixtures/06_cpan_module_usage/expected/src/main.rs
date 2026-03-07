use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

fn main() {
    let data = json!({"name": "Alice", "age": 30});
    let json_str = serde_json::to_string(&data).unwrap();
    println!("JSON: {}", json_str);

    let decoded: Value = serde_json::from_str(&json_str).unwrap();
    println!("Name: {}", decoded["name"].as_str().unwrap());

    let encoded = STANDARD.encode("Hello, World!");
    println!("Base64: {}", encoded);
    let original = STANDARD.decode(&encoded).unwrap();
    println!("Decoded: {}", String::from_utf8(original).unwrap());
}
