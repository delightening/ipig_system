use lettre::message::Mailbox;
fn main() {
    for s in [
        "John Doe <john@example.com>",
        "\"John Doe\" <john@example.com>",
        "Dr. \"Rick\" Chen <john@example.com>",
        "\"Dr. \"Rick\" Chen\" <john@example.com>",
        "王小明 <john@example.com>",
        "\"王小明\" <john@example.com>",
    ] {
        println!("{} => {:?}", s, s.parse::<Mailbox>());
    }
}
