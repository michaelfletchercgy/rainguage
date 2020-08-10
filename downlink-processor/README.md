# downlink-processor

Processes raw telemetry from the downlink device and stores it in a postgres database.  The primary feature is adding a
timestamp to the data

The downlink firmware will write the packets to the serial port.  The serial port must be in raw-mode otherwise certain
bytes will be dropped.

## Future

* Use termios (via rust, maybe termion) to put the tty into raw mode instead of the shell script.
* Store records in a persistent queue if either the network or remote postgres database unavailable.
* Send an acknowledgement
* Produce an integration test
    docker-compose -p to create a 'testing' docker-compose project
    will have to use current clock source
    write a file by hand.
* Using a rust downlink-firmware capture the signal strength.