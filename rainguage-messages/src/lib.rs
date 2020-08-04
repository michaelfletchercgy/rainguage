#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum SerializeError {
    Internal(postcard::Error)
}

impl From<postcard::Error> for SerializeError {
    fn from(err: postcard::Error) -> Self {
        SerializeError::Internal(err)
    }
    
}

#[derive(Debug)]
pub enum DeserializeError {
    Internal(postcard::Error)
}

impl From<postcard::Error> for DeserializeError {
    fn from(err: postcard::Error) -> Self {
        DeserializeError::Internal(err)
    }
    
}

#[derive(Serialize, Deserialize, Debug)]
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

    pub lora_packets_read: u32, 

    pub lora_packets_written: u32, 

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
            lora_packets_read: 0,
            lora_packets_written: 0,
            lora_error_cnt: 0,
            hardware_err_other_cnt: 0
        }
    }
}

pub fn serialize(telem:&TelemetryPacket, buf:&mut [u8]) -> Result<(), SerializeError> {
    postcard::to_slice(telem, buf)?;

    Ok(())
}

pub fn deserialize(buf:&[u8]) -> Result<TelemetryPacket, DeserializeError> {
    Ok(postcard::from_bytes(buf)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        let mut buf:[u8; 64] = [0; 64];

        let mut packet = super::TelemetryPacket::new();
        packet.loop_cnt = 52;
        packet.vbat = 1200;

        super::serialize(&packet, &mut buf).unwrap();
        let deser_packet = super::deserialize(&buf).unwrap();

        assert_eq!(52, deser_packet.loop_cnt);
        assert_eq!(1200, deser_packet.vbat);
    }
}