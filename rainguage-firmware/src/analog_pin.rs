extern crate feather_m0 as hal;
use crate::hal::pac::ADC;
use crate::hal::clock::GenericClockController;

pub struct AnalogPin {
    adc: ADC
}


impl AnalogPin {
    pub fn new(clocks:&mut GenericClockController, adc:ADC) -> AnalogPin {
        // before enabling the ADC, the asynchronous clock source must be selected and enabled, and the ADC reference must be
        // configured.
        let gclock = clocks.gclk0();
        clocks.adc(&gclock).unwrap();

        let mut result = AnalogPin {
            adc
        };

        result.initialize();

        result
    }

    fn initialize(&mut self) {
        // Read the NVM Software Calibration Data and write it back to the adc CALIB register
        let nvm_software_calib_addr = 0x806020u32 as *const u128;
        let nvm_software_calib: u128 = unsafe { *nvm_software_calib_addr };

        let adc_linearity_calibration = ((nvm_software_calib >> 27) & 0xff) as u8;
        let adc_bias_calibration = ((nvm_software_calib >> 35) & 0x8) as u8;

        unsafe {
            self.adc.calib.write(|w| w.linearity_cal().bits(adc_linearity_calibration)
                                    .bias_cal().bits(adc_bias_calibration));
        }

        // Considder dropping this ...
        self.sync_adc();
        self.adc.ctrlb.write(|w| w.prescaler().div32());
        self.adc.ctrlb.write(|w| w.ressel()._12bit());
        
        unsafe { self.adc.sampctrl.write(|w| w.samplen().bits(5)); }
        
        self.sync_adc();
        self.adc.inputctrl.write(|w| w.muxneg().gnd());
        self.adc.avgctrl.write(|w| w.samplenum()._1());
        unsafe { self.adc.avgctrl.write(|w| w.adjres().bits(0)); }
        // to this

        self.sync_adc();
        self.adc.inputctrl.write(|w| w.gain().div2());
        self.adc.refctrl.write(|w| w.refsel().intvcc1());
    }

    #[inline(always)]
    fn sync_adc(&mut self) {
        while self.adc.status.read().syncbusy().bit_is_set() { }
    }

    /*
     * Read the voltage from a pin.
     */
    pub fn read(&mut self) -> u16 {
        self.adc.inputctrl.write(|w| w.muxpos().pin7());
        self.sync_adc(); 

        // enable the adc
        self.adc.ctrla.write(|w| w.enable().set_bit());
        self.sync_adc(); 

        // start the analog-digital conversion.  The first value must be thrown away
        self.adc.swtrig.write(|w| w.start().set_bit());
        self.sync_adc();

        // clear the data ready flag
        self.adc.intflag.write(|w| w.resrdy().set_bit());

        // start the analog-digital conversion again.
        self.sync_adc();
        self.adc.swtrig.write(|w| w.start().set_bit());

        // read the value back
        while self.adc.intflag.read().resrdy().bit_is_clear() { }

        let result = self.adc.result.read().bits();

        // turn off the adc.
        self.sync_adc();
        self.adc.ctrla.write(|w| w.enable().clear_bit());
        self.sync_adc();

        result
    }
}