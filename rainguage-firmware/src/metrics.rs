
static mut USB_ERROR_CNT:u32 = 0;

// This is incremented if there is an error transmitting via lora.
static mut LORA_TRANSMIT_ERROR_CNT:u32 = 0;

static mut LORA_TRANSMIT_BYTES:u32 = 0;

pub fn increment_usb_error_cnt() {
    unsafe { USB_ERROR_CNT = USB_ERROR_CNT.wrapping_add(1); }
}

pub fn get_usb_error_cnt() -> u32 {
    unsafe { USB_ERROR_CNT }
}

pub fn increment_lora_transmit_error_cnt() {
    unsafe { LORA_TRANSMIT_ERROR_CNT = LORA_TRANSMIT_ERROR_CNT.wrapping_add(1); }
}

pub fn get_lora_transmit_error_cnt() -> u32 {
    unsafe { LORA_TRANSMIT_ERROR_CNT }
}

pub fn increment_lora_transmit_bytes(bytes:usize) {
    unsafe { LORA_TRANSMIT_BYTES = LORA_TRANSMIT_BYTES.wrapping_add(bytes as u32); }
}

pub fn get_lora_transmit_bytes() -> u32 {
    unsafe { LORA_TRANSMIT_BYTES }
}