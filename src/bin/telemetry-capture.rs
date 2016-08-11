use std::fs::File;
use std::io::Read;

pub fn main() {
    let mut file = File::open("/dev/ttyS0").unwrap();
    println!("Starting ...");
    read_telem(&mut file, |x| {
        println!("something happened{}", x);
    });
}

fn read_telem<F, R: Read>(r:&mut R,mut f:F)
    where F : FnMut(u32) {

    let pattern = "TIPS=";
    let mut pat_match = 0;
    let mut nums = String::new();

    for byte_io in r.bytes() {
        let b = byte_io.unwrap();

        if pat_match == pattern.len() {
            let c = b as char;
            if c.is_numeric() {
                nums.push(c);
            } else {
                let tips:u32 = nums.parse().unwrap();
                f(tips);
                pat_match = 0;
                nums.clear();
            }
        } else {
            if b == pattern.as_bytes()[pat_match] {
                pat_match = pat_match + 1;
            } else {
                pat_match = 0;
                nums.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_telem;
    use std::io::Read;
    use std::io::Result;

    struct MemBuf<'a> {
        data: &'a [u8],
        pos: usize
    }

    impl <'a> MemBuf<'a> {
        pub fn new(data: &'a [u8]) -> MemBuf {
            MemBuf {
                data:data,
                pos: 0
            }
        }
    }

    impl <'a> Read for MemBuf<'a> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.pos = self.pos + 1;

            if self.pos >= self.data.len() {
                Ok(0)
            } else {
                buf[0] = self.data[self.pos];
                Ok(1)
            }
        }
    } 

    #[test]
    fn it_works() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPS=4vvv");
       read_telem(&mut data, |x| cnt = x);
       assert!(cnt == 4);
    }
}
