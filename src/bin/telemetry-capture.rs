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
        let c = b as char;

        if pat_match == pattern.len() {
            if !c.is_numeric() {
                if nums.len() > 0 {
                    let tips:u32 = nums.parse().unwrap();
                    f(tips);
                }

                pat_match = 0;
                nums.clear();
            }
        } else if b != pattern.as_bytes()[pat_match] {
            pat_match = 0;
            nums.clear();
        }

        if pat_match == pattern.len() && c.is_numeric() {
            nums.push(c);
        } else if b == pattern.as_bytes()[pat_match] {
            pat_match = pat_match + 1;
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
    fn simple() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPS=4vvv");
       read_telem(&mut data, |x| cnt = x);
       assert!(cnt == 4);
    }

    #[test]
    fn nothing() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxvv");
       read_telem(&mut data, |_| cnt=1);
       assert!(cnt == 0);
    }


    #[test]
    fn no_number() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPS=z");
       read_telem(&mut data, |_| cnt=1);
       assert!(cnt == 0);
    }

    #[test]
    fn partial_pattern() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPSz");
       read_telem(&mut data, |_| cnt=1);
       assert!(cnt == 0);
    }

    #[test]
    fn repeating_values() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPS=4TIPS=99vvv");
       read_telem(&mut data, |x| cnt=x);
       assert!(cnt == 99);
    }

    #[test]
    fn reset_after_number() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTIPS=TIPS=99vvv");
       read_telem(&mut data, |x| cnt=x);
       assert!(cnt == 99);
    }

    #[test]
    fn reset_after_pattern() {
       let mut cnt = 0;
       let mut data = MemBuf::new(b"xxxxTITIPS=99vvv");
       read_telem(&mut data, |x| cnt=x);
       assert!(cnt == 99);
    }
}
