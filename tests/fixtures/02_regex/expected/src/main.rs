use regex::Regex;

fn main() {
    let line = "2024-03-15";

    let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap();
    if let Some(caps) = re.captures(line) {
        let year = &caps[1];
        let month = &caps[2];
        let day = &caps[3];
        println!("Year: {}, Month: {}, Day: {}", year, month, day);
    }

    let text = "foo bar foo baz foo";
    let re2 = Regex::new(r"foo").unwrap();
    let text = re2.replace_all(text, "qux").to_string();
    println!("{}", text);
}
