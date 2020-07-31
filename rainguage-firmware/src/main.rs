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

mod buffer;
mod analog_pin;

use core::fmt::Write;
use buffer::Buffer;
use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::{interrupt, CorePeripherals, Peripherals};

use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use cortex_m::asm::delay as cycle_delay;
use cortex_m::peripheral::NVIC;
use hal::prelude::*;
use hal::delay::Delay;
use hal::time::MegaHertz;

const FREQUENCY: i64 = 915;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut parts = peripherals.PORT.split();

    let mut red_led = parts.pa17.into_open_drain_output(&mut parts.port);
    parts.pa7.into_function_b(&mut parts.port);

    let mut vbat = analog_pin::AnalogPin::new(&mut clocks, peripherals.ADC);

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

    let usb_dm = parts.pa24;
    let usb_dp = parts.pa25;
    red_led.set_low().unwrap();

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

     let mut lora = sx127x_lora::LoRa::new(
         lora_spi, cs_out, reset_out, FREQUENCY,
         Delay::new(core.SYST, &mut clocks));

    loop {
        cycle_delay(15 * 1024 * 1024);
        red_led.set_high().unwrap();

        unsafe {
            USB_BUS.as_mut().map(|_usb_dev| {
                
                USB_SERIAL.as_mut().map(|serial| {
                    
                    let mut fail = false;
                    let mut lora_init = false;
                    let mut bytes = 0;
                    let vbat_value = vbat.read();
                    let vbat_volt = ((vbat_value as f32 * 2.0) * 3.3) / 4096.0;
                    match lora.as_mut() {
                         Ok(lora_drv) => {
                             lora_drv.set_tx_power(23, 1).unwrap();
                             //lora_drv.set_mode(RadioMode::)
                             lora_drv.set_preamble_length(8);
                             //setModemConfig(Bw125Cr45Sf128); // Radio default
                             //    setModemConfig(Bw125Cr48Sf4096); // slow and reliable?
                               //  setPreambleLength(8); // Default is 8
                             lora_init = true;
                             let mut buf = Buffer::new();

                             let message = "HALLLO\n\r";
                            let mut buffer = [0;255];
                            for (i,c) in message.chars().enumerate() {
                                buffer[i] = c as u8;
                            }
                             write!(buf, "interrupt_count={} vbat={} vbat_volt={}", INTERRUPT_COUNT, vbat_value, vbat_volt);
                             let s = "hello world".as_bytes();
                             
                             match lora_drv.transmit_payload_busy(buffer, message.len()) {
                                 Ok(c) => { 
                                    let mut buf = Buffer::new();
                                    write!(buf, "xmit {}\n", c);
                                    serial.write(buf.as_bytes());
                                 },
                                 Err(err) => {
                                     let mut buf = Buffer::new();
                                     write!(buf, "err={:?}", err);
                                     serial.write(buf.as_bytes());
                                 }
                             }
                             
                            //  match lora_drv.poll_irq(Some(2)) {
                            //      Ok(r) => {
                            //          bytes = r;
                            //      },
                            //      Err(x) => {
                            //          fail = true;
                            //      }
                            //  }
                         },
                         Err(err) => {
                             lora_init = false;
                             let mut buf = Buffer::new();
                             write!(buf, "lora_err={:?}\r\n", err).unwrap();
                             serial.write(buf.as_bytes());

                            }
                    };

                    let vbat_value = vbat.read();
                    let vbat_volt = ((vbat_value as f32 * 2.0) * 3.3) / 1024.0;
                    let mut buf = Buffer::new();
                    write!(buf, "interrupt_count={} USB_SERIAL_BYTES_READ={}, fail={}, bytes={}, lora_init={} vbat={} vbat_volt={}\r\n", 
                        INTERRUPT_COUNT, USB_SERIAL_BYTES_READ, fail, bytes, lora_init, vbat_value, vbat_volt).unwrap();
                    //write!(buf, "hi\n");
                    serial.write(buf.as_bytes());


                });
            });
        }
        red_led.set_low().unwrap();
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

                // it fails if we don't read
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
