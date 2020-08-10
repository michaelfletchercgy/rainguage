# rainguage-firmware

rainguage-firmware is responsible for reading rain and sending telemetry.

## Future

* Capture values from the gpio port.  Use interrupts.

* Properly buffer and send data over usb.
* Include more metrics.
* Could do with a code cleanup.
* Create a buffer of packets and explictly acknowledge them.
* Log / record why the machine may have reset (ResetCause)
* Use provided serial_number fn.
* Produce a global error handler.
* Sleep mode (obviously needs interrupts).