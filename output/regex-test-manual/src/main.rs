// Converted from /tmp/regex-test.pl
// Demonstrates Perl regex → Rust regex conversion

use regex::Regex;

fn main() {
    // キャプチャグループ
    // Perl: if ($date =~ /^(\d{4})-(\d{2})-(\d{2})$/) { print "year=$1, month=$2, day=$3\n"; }
    let date = "2024-03-08";
    let date_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap();
    if let Some(caps) = date_re.captures(date) {
        println!("year={}, month={}, day={}", &caps[1], &caps[2], &caps[3]);
    }

    // 置換
    // Perl: (my $result = $text) =~ s/Hello/Goodbye/g;
    // Note: Perl's copy-then-substitute idiom → Rust: clone + replace_all
    let text = "Hello World Hello";
    let hello_re = Regex::new(r"Hello").unwrap();
    let result = hello_re.replace_all(text, "Goodbye");
    println!("{}", result);

    // 名前付きキャプチャ
    // Perl: /\[(?P<level>\w+)\] (?P<message>.+) at line (?P<line>\d+)/
    // Perl's (?P<name>...) is the same as Rust's regex (?P<name>...)
    let log = "[ERROR] Connection failed at line 42";
    let log_re =
        Regex::new(r"\[(?P<level>\w+)\] (?P<message>.+) at line (?P<line>\d+)").unwrap();
    if let Some(caps) = log_re.captures(log) {
        println!("level={}, line={}", &caps["level"], &caps["line"]);
        println!("message={}", &caps["message"]);
    }
}
