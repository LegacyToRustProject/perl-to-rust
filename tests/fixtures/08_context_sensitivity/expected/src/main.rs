fn main() {
    let arr = vec![1, 2, 3, 4, 5];

    // Scalar context: count elements
    let count = arr.len();
    println!("Count: {}", count);

    // List context: copy
    let mut copy = arr.clone();
    copy.push(6);
    println!(
        "Original: {}",
        arr.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!(
        "Copy: {}",
        copy.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );

    // Scalar context in string interpolation
    println!("Array has {} elements", arr.len());

    // Wantarray equivalent: return different types based on context
    // In Rust, we use separate functions or an enum
    let list_result = context_test_list();
    let scalar_result = context_test_scalar();
    println!(
        "List: {}",
        list_result
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("Scalar: {}", scalar_result);
}

fn context_test_list() -> Vec<i32> {
    vec![1, 2, 3]
}

fn context_test_scalar() -> &'static str {
    "scalar"
}
