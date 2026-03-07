mod my_module;

use my_module::MyModule;

fn main() {
    let mut list = MyModule::new();
    list.add_item("alpha".to_string());
    list.add_item("beta".to_string());
    list.add_item("gamma".to_string());

    println!("Count: {}", list.count());
    println!("Items: {}", list.to_string());

    for item in list.all_items() {
        println!("  - {}", item);
    }
}
