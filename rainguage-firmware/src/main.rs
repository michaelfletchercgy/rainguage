#![no_std]
#![no_main]
#![feature(lang_items)]

extern crate cortex_m;
extern crate feather_m0 as hal;
extern crate panic_halt;
extern crate usb_device;
extern crate usbd_serial;
extern crate embedded_hal;
extern crate sx127x_lora;

mod analog_pin;
mod buffer;
mod metrics;
mod usb_write;

use analog_pin::AnalogPin;
use buffer::Buffer;
use core::fmt::Write;
use cortex_m::asm::delay as cycle_delay;
use cortex_m::peripheral::NVIC;
use embedded_hal::digital::v2::OutputPin;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::time::MegaHertz;
use hal::usb::UsbBus;
use sx127x_lora::LoRa;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usb_write::UsbWrite;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

const FREQUENCY: i64 = 915;

// How frequently should we transmit.  So every TRANSMIT_CYCLE loops we will sent a telemetry packer.
const TRANSMIT_CYCLE: usize = 10;

#[entry]
fn main() -> ! {
    //
    // Phase 1 of Hardware Initializing ... get USB going so we can find out about things.
    //
    let mut usb_write = UsbWrite::new();
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut parts = peripherals.PORT.split();

    let usb_dm = parts.pa24;
    let usb_dp = parts.pa25;

    // Sleep for 5s on the startup to aid resetting to booting mode.
    cycle_delay(60 * 1024 * 1024);

    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(hal::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            usb_dm,
            usb_dp,
            &mut parts.port,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build()
        );
    }

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    //
    // Phase 2 of Hardware Initializing ... everything else
    //
    let mut red_led = parts.pa17.into_open_drain_output(&mut parts.port);
    if let Err(_) = red_led.set_low() {
        write!(usb_write, "Could not set red_led low.  That is weird.").unwrap();
    }

    parts.pa7.into_function_b(&mut parts.port);

    let mut vbat = AnalogPin::new(&mut clocks, peripherals.ADC);

    let sck = parts.pb11.into_floating_input(&mut parts.port);
    let mosi = parts.pb10.into_floating_input(&mut parts.port);
    let miso = parts.pa12.into_floating_input(&mut parts.port);

    let lora_spi = hal::spi_master(&mut clocks, MegaHertz(8), peripherals.SERCOM4,
        &mut peripherals.PM, 
        sck,
        mosi, 
        miso,
        &mut parts.port);

    let cs_out = parts.pa6.into_open_drain_output(&mut parts.port);
    let reset_out = parts.pa8.into_open_drain_output(&mut parts.port);
    let _ = parts.pa9.into_open_drain_output(&mut parts.port); // int_out lora interrupt line

     let mut lora = match LoRa::new(
         lora_spi, cs_out, reset_out, FREQUENCY,
         Delay::new(core.SYST, &mut clocks)) {
            Ok(lora) => lora,
            Err(err) => {
                write!(usb_write, "Failed to initialize lora:{:?}", err).unwrap();

                fatal_error("lora failed to initialize", &mut usb_write)
            }
         };

    let id_word0 = unsafe { *(0x0080A00C as *const u32) };
    let id_word1 = unsafe { *(0x0080A040 as *const u32) };
    let id_word2 = unsafe { *(0x0080A044 as *const u32) };
    let id_word3 = unsafe { *(0x0080A048 as *const u32) };

    let mut loop_cnt: usize = 0;
    let mut transmit_counter = TRANSMIT_CYCLE;
    loop {
        cycle_delay(15 * 1024 * 1024);
        red_led.set_high().unwrap();

        let vbat_value = vbat.read();

        let interrupt_count = unsafe{ INTERRUPT_COUNT };
        let usb_serial_bytes_read = unsafe{ USB_SERIAL_BYTES_READ };

        if transmit_counter >= TRANSMIT_CYCLE {
            transmit_counter = 0;
            let mut buffer = Buffer::new();

            write!(buffer, ">>>>>> loop={} vbat={}, usb_int_cnt={}, usb_ser_read={}, usb_err_cnt={}, lora_xmit_cnt={} calib={:x} id0={:x} id1={:x} id2={:x} id3={:x}\r\n",
                loop_cnt, vbat_value, interrupt_count, usb_serial_bytes_read, metrics::get_usb_error_cnt(), metrics::get_lora_transmit_error_cnt(), calib,
                id_word0, id_word1, id_word2, id_word3).unwrap();
            
            match lora.transmit_payload_busy(buffer.as_bytes(), buffer.size()) {
                Ok(_) => { 
                    //write!(usb_write, "xmit {}\n", c).unwrap();
                },
                Err(_) => {
                    metrics::increment_lora_transmit_error_cnt();
                }
            }
        }
        transmit_counter = transmit_counter + 1;

        // write!(usb_write, "interrupt_count={} usb_serial_bytes_read={}, vbat={} vbat_volt={}\r\n", 
        // interrupt_count, usb_serial_bytes_read, vbat_value, vbat_volt).unwrap();

        red_led.set_low().unwrap();

        loop_cnt = loop_cnt.wrapping_add(1);
     }
}

/// If we fail in a fatal way, the best we can do is print that error
/// over and over
fn fatal_error(msg: &str, usb_output:&mut UsbWrite) -> ! {
    loop {
        match write!(usb_output, "fatal error: {}", msg) {
            Ok(_) => {}, // don't care
            Err(_) => {} // not much we can do now
        }
    }
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
static mut INTERRUPT_COUNT:u64 = 0;
static mut USB_SERIAL_BYTES_READ: usize = 0;

fn poll_usb() {
    unsafe {
        INTERRUPT_COUNT = INTERRUPT_COUNT + 1;
    }
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                usb_dev.poll(&mut [serial]);
                let mut buf = [0u8; 64];

                if let Ok(count) = serial.read(&mut buf) {
                    USB_SERIAL_BYTES_READ = USB_SERIAL_BYTES_READ + count;
                }
            });
        });
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}