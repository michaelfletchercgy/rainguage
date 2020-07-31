use core::fmt::Write;
use core::fmt::Result;

pub struct Buffer {
    data: [u8; 255],
    pos: usize
}

impl Buffer {
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[0..self.pos]
    }

    pub fn as_all_bytes(&self) -> [u8; 255] {
        self.data
    }

    pub fn size(&self) -> &usize {
        &self.pos
    }

    pub const fn new() -> Buffer {
        Buffer {
            data: [0; 255],
            pos: 0
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}
impl Write for Buffer {
    fn write_str(&mut self, s: &str) -> Result {
        let bytes = s.as_bytes();

        for x in 0..bytes.len() {
            self.data[self.pos] = bytes[x];
            self.pos = self.pos + 1;
        }

        Ok(())
    }
}