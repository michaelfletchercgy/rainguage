
static mut USB_ERROR_CNT:usize = 0;

// This is incremented if there is an error transmitting via lora.
static mut LORA_TRANSMIT_ERROR_CNT:usize = 0;

pub fn increment_usb_error_cnt() {
    unsafe { USB_ERROR_CNT = USB_ERROR_CNT.wrapping_add(1); }
}

pub fn get_usb_error_cnt() -> usize {
    unsafe { USB_ERROR_CNT }
}

pub fn increment_lora_transmit_error_cnt() {
    unsafe { LORA_TRANSMIT_ERROR_CNT = LORA_TRANSMIT_ERROR_CNT.wrapping_add(1); }
}

pub fn get_lora_transmit_error_cnt() -> usize {
    unsafe { LORA_TRANSMIT_ERROR_CNT }
}