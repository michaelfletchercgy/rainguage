use core::fmt::Write;
use core::fmt::Result;

pub struct UsbWrite {

}

impl UsbWrite {
    pub const fn new() -> UsbWrite {
        UsbWrite {
            
        }
    }
}

impl Write for UsbWrite {
    fn write_str(&mut self, s: &str) -> Result {
        let bytes = s.as_bytes();
        let mut written = 0;
        let mut stop = false;
        while !stop  {
            unsafe {
                super::USB_BUS.as_mut().map(|_usb_dev| {
                    super::USB_SERIAL.as_mut().map(|serial| {
                        match serial.write(&bytes[written..]) {
                            Ok(bytes_written) => {
                                written += bytes_written;

                                super::metrics::increment_usb_bytes_written(written as u32);
                                if written == bytes.len() {
                                    stop = true;
                                }
                            },
                            Err(_) => {
                                super::metrics::increment_usb_error_cnt();
                                // wouldblock is stalling for some reason.
                                stop = true;
                            }
                        }
                    });
                });
            };
        }

        Ok(())
    }
}