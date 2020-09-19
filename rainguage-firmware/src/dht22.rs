use core::prelude::v1::Result;
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[derive(Debug)]
pub enum DhtError {
    Checksum,
    IO,
    Timeout
}

/// A temperature and humidity reading from the DHT22.
#[derive(Debug)]
pub struct Reading {
    pub temperature: f32,
    pub humidity: f32
}

const MAX_COUNT:usize = 2000000;
const DHT_PULSES:usize = 41;

fn decode(arr:[usize; DHT_PULSES*2]) -> Result<Reading, DhtError> {
    let mut threshold:usize = 0;

    let mut i = 2;
    while i < DHT_PULSES * 2 {
        threshold += arr[i];

        i += 2;
    }

    threshold /= DHT_PULSES - 1;

    let mut data = [0 as u8; 5];
    let mut i = 3;
    while i < DHT_PULSES * 2 {
        let index = (i-3) / 16;
        data[index] <<= 1;
        if arr[i] >= threshold {
            data[index] |= 1;
        } else {
            // else zero bit for short pulse
        }

        i += 2;
    }

    if data[4] != (data[0].wrapping_add(data[1]).wrapping_add(data[2]).wrapping_add(data[3]) & 0xFF) {
        return Result::Err(DhtError::Checksum);
    }

    let h_dec = data[0] as u16 * 256 + data[1] as u16;
    let h = h_dec as f32 / 10.0f32;

    let t_dec = (data[2] & 0x7f) as u16 * 256 + data[3] as u16;
    let mut t = t_dec as f32 / 10.0f32;
    if (data[2] & 0x80) != 0 {
        t *= -1.0f32;
    }

    Result::Ok(Reading {
        temperature: t,
        humidity: h
    })
}

pub fn init<Error>(
    output_pin: &mut dyn OutputPin<Error = Error>,
    delay_us: &mut dyn FnMut(u32) -> (),
) -> Result<(), DhtError> {
    output_pin.set_high().map_err(|_| DhtError::IO)?;
    delay_us(500_000);
    // Voltage  level  from  high to  low.
    // This process must take at least 18ms to ensure DHT’s detection of MCU's signal.
    output_pin.set_low().map_err(|_| DhtError::IO)?;
    delay_us(20_000);
    Ok(())
}


pub fn read<Error>(
    input_pin: &mut dyn InputPin<Error = Error>
) -> Result<Reading, DhtError> {

    let mut pulse_counts: [usize; DHT_PULSES*2] = [0; DHT_PULSES * 2];

    let mut count:usize = 0;

    while input_pin.is_high().unwrap_or_default() {
        count = count + 1;

        if count > MAX_COUNT {
            return Err(DhtError::Timeout);
        }
    }

    for c in 0..DHT_PULSES {
        let i = c * 2;


        while input_pin.is_high().unwrap_or_default() == false {
            pulse_counts[i] = pulse_counts[i] + 1;

            if pulse_counts[i] > MAX_COUNT {
                return Err(DhtError::Timeout);
            }
        }

        while input_pin.is_high().unwrap_or_default() == true {
            pulse_counts[i + 1] = pulse_counts[i + 1] + 1;

            if pulse_counts[i + 1] > MAX_COUNT {
                return Err(DhtError::Timeout);
            }
        }
    }

    decode(pulse_counts)
}

