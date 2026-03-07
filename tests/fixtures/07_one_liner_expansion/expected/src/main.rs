use regex::Regex;

fn main() {
    let lines = vec![
        "INFO: Server started",
        "ERROR: Connection failed",
        "WARN: Timeout detected",
        "error: disk full",
        "INFO: Request processed",
    ];

    // Print lines matching /error/i
    let error_re = Regex::new(r"(?i)error").unwrap();
    for line in &lines {
        if error_re.is_match(line) {
            println!("{}", line);
        }
    }

    // Equivalent of: perl -pe 's/foo/bar/g'
    let mut texts = vec![
        "foo and foo".to_string(),
        "no match".to_string(),
        "foofoo".to_string(),
    ];
    let foo_re = Regex::new(r"foo").unwrap();
    for text in &mut texts {
        *text = foo_re.replace_all(text, "bar").to_string();
        println!("{}", text);
    }

    // Equivalent of: perl -ane 'print "$F[0]\n"'
    let records = vec!["Alice 30 Engineer", "Bob 25 Designer"];
    for record in &records {
        let fields: Vec<&str> = record.split_whitespace().collect();
        println!("{}", fields[0]);
    }
}
