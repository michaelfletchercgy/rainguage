#![no_std]

use serde::{Serialize, Deserialize};
use crc::{crc32, Hasher32};
use core::iter::Iterator;

use byteorder::ByteOrder;
use byteorder::NetworkEndian;

const MAGIC:[u8;3] = [125, 8, 141];

#[derive(Debug)]
pub enum SerializeError {
    Internal(postcard::Error)
}

impl From<postcard::Error> for SerializeError {
    fn from(err: postcard::Error) -> Self {
        SerializeError::Internal(err)
    }
    
}

#[derive(Debug, PartialEq)]
pub enum DeserializeError {
    SerializeError(postcard::Error),
    InvalidLength,
    InvalidChecksum{
        crc32_buf: [u8;4],
        msg_buf: [u8; 64],
        msg_len: u8
    }
}

impl From<postcard::Error> for DeserializeError {
    fn from(err: postcard::Error) -> Self {
        DeserializeError::SerializeError(err)
    }
    
}

//
// ReadingMagic -> ReadingLength -> ReadingBytes -> ReadingChecksum
//
#[derive(Debug)]
enum IteratorState {
    ReadingMagic{
        bytes_read:u8
    },
    ReadingLength,
    ReadingBytes{
        msg_len: u8,
        num_read: usize,
        msg_buf: [u8; 64]
    },
    ReadingChecksum {
        num_read: usize,
        crc32_buf: [u8;4],
        msg_buf: [u8; 64],
        msg_len: u8
    }
}

// Wraps an iterator of bytes
pub struct PacketIterator <I:Iterator<Item=u8>> {
    byte_iter:I,
    state:IteratorState
}

impl <I:Iterator<Item=u8>> PacketIterator<I> {
    pub fn new(byte_iter:I) -> PacketIterator<I> {
        PacketIterator {
            byte_iter,
            state:IteratorState::ReadingMagic {
                bytes_read: 0
            }
        }
    }
}


impl <'a, I:Iterator<Item=u8>> Iterator for PacketIterator<I> {
    type Item = Result<TelemetryPacket, DeserializeError>;

    fn next(&mut self) -> Option<Result<TelemetryPacket, DeserializeError>> {
        loop {
            match self.byte_iter.next() {
                Some(byte) => {
                    //println!("byte={}", byte);
                    match self.state {
                        IteratorState::ReadingMagic {ref mut bytes_read } => {
                            if *bytes_read > 3 {
                                *bytes_read = 0;
                            } else if byte == MAGIC[*bytes_read as usize] {
                                *bytes_read = *bytes_read + 1;
                            }
    
                            if *bytes_read == 3 {
                                self.state = IteratorState::ReadingLength;
                            }
                        },
                        IteratorState::ReadingLength => {
                            if byte > 64 {
                                self.state = IteratorState::ReadingMagic{ bytes_read:0 };
                                return Some(Result::Err(DeserializeError::InvalidLength));
                            }

                            self.state = IteratorState::ReadingBytes {
                                msg_len:byte,
                                num_read:0,
                                msg_buf: [0u8; 64]
                            };
                        },
                        IteratorState::ReadingBytes{msg_len, ref mut num_read, ref mut msg_buf} => {
                            msg_buf[*num_read] = byte;
                            *num_read += 1;

                            if *num_read >= msg_len.into() {                                
                                self.state = IteratorState::ReadingChecksum {
                                    num_read:0,
                                    msg_buf:*msg_buf,
                                    msg_len,
                                    crc32_buf: [0u8; 4]
                                };
                            }
                        },
                        IteratorState::ReadingChecksum{ref mut num_read, msg_buf, msg_len, ref mut crc32_buf} => {
                            crc32_buf[*num_read] = byte;
                            *num_read += 1;

                            if *num_read >= 4 {
                                //println!("num_read={}", *num_read);
                                //println!("msg_len={}", msg_len);
                                //println!("bytes={:?}", &msg_buf[0..msg_len as usize]);
                                let mut digest = crc32::Digest::new_custom(crc32::IEEE,  0u32,  0u32, crc::CalcType::Normal);
                                digest.write(&msg_buf[0..msg_len as usize]);
                                let calculated_sum = digest.sum32();
                                // let mut calculated_sum:u32 = 0;
                                // for b in &msg_buf[0..msg_len as usize] {
                                //     calculated_sum = calculated_sum.wrapping_add((*b) as u32);
                                // }
                                //println!("calc_sum={}", calculated_sum);
                                //let mut print_buf = [0u8;4];
                                //NetworkEndian::write_u32(&mut print_buf, calculated_sum);
                                //println!("calc_sum as bytes={:?}", &print_buf);
                                
                                let provided_sum = NetworkEndian::read_u32(crc32_buf);
                                //println!(" msg_sum={}", provided_sum);
                                let clone_buf = crc32_buf.clone();
                                self.state = IteratorState::ReadingMagic { bytes_read:0 };

                                if calculated_sum == provided_sum {
                                    match postcard::from_bytes(&msg_buf[0..msg_len as usize]) {
                                        Ok(packet) => {
                                            return Some(Result::Ok(packet));
                                        },
                                        Err(err) => {
                                            return Some(Result::Err(DeserializeError::SerializeError(err)));
                                        }
                                    }
                                } else {
                                    return Some(Result::Err(DeserializeError::InvalidChecksum{
                                        msg_buf,
                                        msg_len,
                                        crc32_buf:clone_buf
                                    }));
                                }
                            }
                        }
                    }
                },
                None => {
                    return Option::None;
                }
            };
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
/// TelemetryPacket is sent from the rainguage.
pub struct TelemetryPacket {
    /// The hardware identifier 
    pub device_id: [u8; 16],

    /// The number of loops that have been run.  The device has no clock so this is an approximation of time.  It
    /// will wrap back to 0.
    pub loop_cnt: u32,

    /// The number of times the rainguage has tipped over.
    pub tip_cnt: u32,

    /// The last voltage recorded by the battery.
    pub vbat: u32,

    pub temperature: f32,

    pub relative_humidity: f32,

    pub usb_bytes_read: u32,

    pub usb_bytes_written: u32,

    pub usb_error_cnt: u32,

    pub lora_rx_bytes: u32, 

    pub lora_tx_bytes: u32, 

    pub lora_error_cnt: u32,

    /// The number of other hardware errors that occured.  This is an error outside of a more specific hardware error.  Things like flashing
    /// leds.
    pub hardware_err_other_cnt: u32
}

impl TelemetryPacket {
    pub fn new() -> TelemetryPacket {
        TelemetryPacket {
            device_id: [0; 16],
            loop_cnt: 0,
            tip_cnt: 0,
            vbat: 0,
            temperature: 0.0,
            relative_humidity: 0.0,
            usb_bytes_read: 0,
            usb_bytes_written: 0,
            usb_error_cnt: 0,
            lora_rx_bytes: 0,
            lora_tx_bytes: 0,
            lora_error_cnt: 0,
            hardware_err_other_cnt: 0
        }
    }
}

// Serialize a telemetry packet into a byte buffer returning the length of the written bytes.
//
// The packet is written including a magic value, bytes and a checksum.  The format is
//
//   magic      3 bytes (125, 8, 141)
//   len        1 byte (length of bytes packet)
//   bytes      `len` bytes 
//   checksum   a crc32 checksum of bytes
pub fn serialize(telem:&TelemetryPacket, buf:&mut [u8]) -> Result<usize, SerializeError> {
    // Write magic into the first three bytes
    buf[0] = MAGIC[0];
    buf[1] = MAGIC[1];
    buf[2] = MAGIC[2];

    // Serialize the telemetry packet
    let result = postcard::to_slice(telem, &mut buf[4..])?;
    let len = result.len();

    // Calculate the crc32 checksum
    let mut digest = crc32::Digest::new(crc32::IEEE);
    digest.write(result);
    let checksum = digest.sum32();
    //let checksum:u32 = 5;
    // Write the length into the buffer
    buf[3] = result.len() as u8;

    // Write the checksum into the buffer.
    NetworkEndian::write_u32(&mut buf[len + 4..len+4+4], checksum);

    // write the sum in
    Ok(len + 3 + 1 + 4)
}

pub fn deserialize(buf:&[u8]) -> Result<TelemetryPacket, DeserializeError> {
    // read len
    // read the checksum
    // calc checksum
    // compare checksum
    Ok(postcard::from_bytes(buf)?)
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    #[test]
    fn largest_packet() {
        let mut packet = super::TelemetryPacket::new();
        packet.device_id = [255; 16];
        packet.loop_cnt = u32::MAX;
        packet.tip_cnt = u32::MAX;
        packet.vbat = u32::MAX;
        packet.temperature = 0.0;
        packet.relative_humidity = 0.0;
        packet.usb_bytes_read = u32::MAX;
        packet.usb_bytes_written = u32::MAX;
        packet.usb_error_cnt = u32::MAX;
        packet.lora_rx_bytes = u32::MAX;
        packet.lora_tx_bytes = u32::MAX;
        packet.lora_error_cnt = u32::MAX;
        packet.hardware_err_other_cnt = u32::MAX;

        let mut buf:[u8; 127] = [0; 127];
        
        let cnt = super::serialize(&packet, &mut buf).unwrap();
        println!("{:?}", buf);
        assert_eq!(72, cnt);
    }

    #[test]
    fn test_serialize_deserialize_zero() {
        let mut buf:[u8; 255] = [0; 255];

        let packet = super::TelemetryPacket::new();
        super::serialize(&packet, &mut buf).unwrap();
        println!("{:?}", buf);
        let bytes = buf.iter()
        .map(|byte| *byte);

        let mut iter = super::PacketIterator::new(bytes);
        assert_eq!(Some(Ok(packet)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_serialize_deserialize_full() {
        let mut buf:[u8; 255] = [0; 255];

        let mut packet = super::TelemetryPacket::new();
        packet.device_id = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 120, 130, 140, 150, 160, 170];
        packet.loop_cnt = 180;
        packet.vbat = 190 as u32;
        packet.usb_bytes_read = 200;
        packet.usb_error_cnt = 210;
        packet.lora_error_cnt = 220;
        packet.lora_tx_bytes = 230;

        super::serialize(&packet, &mut buf).unwrap();

        let bytes = buf.iter()
        .map(|byte| *byte);

        let mut iter = super::PacketIterator::new(bytes);
        let first_packet = iter.next().unwrap().unwrap();
        assert_eq!(packet, first_packet);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_bad_checksum() {
        let mut buf:[u8; 128] = [0; 128];

        let packet = super::TelemetryPacket::new();
        super::serialize(&packet, &mut buf).unwrap();
        // Random Change
        buf[28] = 23;
        let bytes = buf.iter()
            .map(|byte| *byte);

        let mut iter = super::PacketIterator::new(bytes);
        if let Some(Err(_)) = iter.next() {

        } else {
            panic!("expected to get an error");
        }
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_read() {
        use crate::std::io::Read;
        let file = std::fs::File::open("baddata2").unwrap();
        let mut packet_iter = super::PacketIterator::new(file.bytes().map(|r| r.unwrap()));
        println!("{:?}", packet_iter.next());
    }
    #[test]
    fn test_failing_packet() {
        //let mut buf: [u8; 68] = [125, 8, 141, 64, 100, 148, 118, 9, 80, 76, 84, 53, 56, 46, 49, 32, 255, 10, 53, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        //let result: Result<super::TelemetryPacket, postcard::Error> = postcard::from_bytes(&buf);
        //println!("{:?}", result);
        // 02:04:22 [INFO] received:Err(InvalidChecksum { crc32_buf: [99, 69, 122, 178], msg_buf: [100, 148, 118, 9, 80, 76, 84, 53, 56, 46, 49, 32, 255, 10, 53, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], msg_len: 64 })
        let mut buf:[u8; 90] = [0; 90];

        let mut packet = super::TelemetryPacket::new();
        packet.device_id = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 120, 130, 140, 150, 160, 170];
        packet.loop_cnt = 180;
        packet.vbat = 190 as u32;
        packet.usb_bytes_read = 200;
        packet.usb_error_cnt = 210;
        packet.lora_error_cnt = 220;
        packet.lora_tx_bytes = 230;

        println!("b4 :{:?}", buf);
        let result = postcard::to_slice(&packet, &mut buf[4..]);
        println!("res:{:?}", &result);
        println!("af :{:?}", &buf);

    }
}
