use std::fs;
use std::io::{BufRead, BufReader, Write};

fn main() {
    // Write to a file
    let mut file = fs::File::create("/tmp/perl_test.txt").expect("Cannot open");
    writeln!(file, "Line 1").unwrap();
    writeln!(file, "Line 2").unwrap();
    writeln!(file, "Line 3").unwrap();

    // Read from the file
    let file = fs::File::open("/tmp/perl_test.txt").expect("Cannot open");
    let reader = BufReader::new(file);
    let mut line_num = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        line_num += 1;
        println!("[{}] {}", line_num, line);
    }

    // Clean up
    fs::remove_file("/tmp/perl_test.txt").ok();
}
