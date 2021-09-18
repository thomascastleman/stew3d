use std::fs::File;
use std::io::{self, Read};

mod instr;

fn main() -> io::Result<()> {
    let filename = "print-mem.3000.b";

    let mut f = File::open(filename)?;
    let mut buffer = Vec::new();
    let file_size = f.read_to_end(&mut buffer)?;

    println!("file is {} bytes", file_size);
    for b in buffer {
        println!("{:02x}", b);
    }

    Ok(())
}
