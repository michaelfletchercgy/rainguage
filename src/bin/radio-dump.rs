use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io::stdout;

pub fn main() {
    let file = File::open("/dev/ttyS0").unwrap();

    println!("Starting ...");

    for byte in file.bytes() {
        let b = byte.unwrap();

        if b >=32 && b <=126 {
            print!("{}", b as char);
            stdout().flush().unwrap();
        }
    } 
}

