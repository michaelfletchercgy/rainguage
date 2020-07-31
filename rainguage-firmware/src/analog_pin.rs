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
        let linearity = 0;
        let bias = 0;

        // unsafe {
        //     self.adc.calib.write(|w| w.linearity_cal().bits(linearity));
        //     self.adc.calib.write(|w| w.bias_cal().bits(bias));
        // }
        /*
        // ADC Bias Calibration
        uint32_t bias = (*((uint32_t *) ADC_FUSES_BIASCAL_ADDR) & ADC_FUSES_BIASCAL_Msk) >> ADC_FUSES_BIASCAL_Pos;

        // ADC Linearity bits 4:0
        uint32_t linearity = (*((uint32_t *) ADC_FUSES_LINEARITY_0_ADDR) & ADC_FUSES_LINEARITY_0_Msk) >> ADC_FUSES_LINEARITY_0_Pos;

        // ADC Linearity bits 7:5
        linearity |= ((*((uint32_t *) ADC_FUSES_LINEARITY_1_ADDR) & ADC_FUSES_LINEARITY_1_Msk) >> ADC_FUSES_LINEARITY_1_Pos) << 5;

        ADC->CALIB.reg = ADC_CALIB_BIAS_CAL(bias) | ADC_CALIB_LINEARITY_CAL(linearity);
        */
        /*
        if ( g_APinDescription[ulPin].ulPin & 1 ) // is pin odd?
        {
          uint32_t temp ;
  
          // Get whole current setup for both odd and even pins and remove odd one
          temp = (PORT->Group[g_APinDescription[ulPin].ulPort].PMUX[g_APinDescription[ulPin].ulPin >> 1].reg) & PORT_PMUX_PMUXE( 0xF ) ;
          // Set new muxing
          PORT->Group[g_APinDescription[ulPin].ulPort].PMUX[g_APinDescription[ulPin].ulPin >> 1].reg = temp|PORT_PMUX_PMUXO( ulPeripheral ) ;
          // Enable port mux
          PORT->Group[g_APinDescription[ulPin].ulPort].PINCFG[g_APinDescription[ulPin].ulPin].reg |= PORT_PINCFG_PMUXEN | PORT_PINCFG_DRVSTR;
        }
        */
        self.sync_adc();
        self.adc.ctrlb.write(|w| w.prescaler().div32());
        self.adc.ctrlb.write(|w| w.ressel()._12bit());
        unsafe { self.adc.sampctrl.write(|w| w.samplen().bits(5)); }
        
        self.sync_adc();
        self.adc.inputctrl.write(|w| w.muxneg().gnd());
        self.adc.avgctrl.write(|w| w.samplenum()._1());
        unsafe { self.adc.avgctrl.write(|w| w.adjres().bits(0)); }

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
        // pinPeripheral(pin, PIO_ANALOG);

        // Select the pin 
        // ADC->INPUTCTRL.bit.MUXPOS = g_APinDescription[pin].ulADCChannelNumber; // Selection for the positive ADC input
        // TODO I am guess a bit with this pin.
        // Description #1
        // #9 - GPIO #9, also analog input A7. This analog input is connected to a voltage divider for the 
        // lipoly battery so be aware that this pin naturally 'sits' at around 2VDC due to the resistor 
        // divider
        // Description #2
        //    #define VBATPIN A7
        //    float measuredvbat = analogRead(VBATPIN);
        //    I think this is 7?  Could also be 12.
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
              /* The first conversion after the reference is changed must not be used. All other configuration registers must
be stable during the conversion. The source for GCLK_ADC is selected and enabled in the System Controller
(SYSCTRL). Refer to “SYSCTRL – System Controller” on page 148 for more details.
When GCLK_ADC is enabled, the ADC can be enabled by writing a one to the Enable bit in the Control Register A
(CTRLA.ENABLE).*/
        
        /*
        	    syncDAC();
		
		DAC->CTRLA.bit.ENABLE = 0x00; // Disable DAC
		//DAC->CTRLB.bit.EOEN = 0x00; // The DAC output is turned off.
		syncDAC();
        */
        /*
          syncADC();
  ADC->INPUTCTRL.bit.MUXPOS = g_APinDescription[pin].ulADCChannelNumber; // Selection for the positive ADC input
  
  // Control A
  /*
   * Bit 1 ENABLE: Enable
   *   0: The ADC is disabled.
   *   1: The ADC is enabled.
   * Due to synchronization, there is a delay from writing CTRLA.ENABLE until the peripheral is enabled/disabled. The
   * value written to CTRL.ENABLE will read back immediately and the Synchronization Busy bit in the Status register
   * (STATUS.SYNCBUSY) will be set. STATUS.SYNCBUSY will be cleared when the operation is complete.
   *
   * Before enabling the ADC, the asynchronous clock source must be selected and enabled, and the ADC reference must be
   * configured. The first conversion after the reference is changed must not be used.
   */
  syncADC();
  ADC->CTRLA.bit.ENABLE = 0x01;             // Enable ADC

  // Start conversion
  syncADC();
  ADC->SWTRIG.bit.START = 1;

  // Clear the Data Ready flag
  ADC->INTFLAG.reg = ADC_INTFLAG_RESRDY;

  // Start conversion again, since The first conversion after the reference is changed must not be used.
  syncADC();
  ADC->SWTRIG.bit.START = 1;

  // Store the value
  while (ADC->INTFLAG.bit.RESRDY == 0);   // Waiting for conversion to complete
  valueRead = ADC->RESULT.reg;

  syncADC();
  ADC->CTRLA.bit.ENABLE = 0x00;             // Disable ADC
  syncADC();
#endif

  return mapResolution(valueRead, _ADCResolution, _readResolution);

  In the most basic configuration, the ADC sample values from the configured internal or external sources (INPUTCTRL
register). The rate of the conversion is dependent on the combination of the GCLK_ADC frequency and the clock
prescaler.
To convert analog values to digital values, the ADC needs first to be initialized, as described in “Initialization” on page
847. Data conversion can be started either manually, by writing a one to the Start bit in the Software Trigger register
(SWTRIG.START), or automatically, by configuring an automatic trigger to initiate the conversions. A free-running mode
could be used to continuously convert an input channel. There is no need for a trigger to start the conversion. It will start
automatically at the end of previous conversion.
The automatic trigger can be configured to trigger on many different conditions.
The result of the conversion is stored in the Result register (RESULT) as it becomes available, overwriting the result from
the previous conversion.
To avoid data loss if more than one channel is enabled, the conversion result must be read as it becomes available
(INTFLAG.RESRDY). Failing to do so will result in an overrun error condition, indicated by the OVERRUN bit in the
Interrupt Flag Status and Clear register (INTFLAG.OVERRUN).
To use an interrupt handler, the corresponding bit in the Interrupt Enable Set register (INTENSET) must be written to
one.
        */
    }
}