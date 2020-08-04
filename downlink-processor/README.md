# downlink-processor

Processes raw telemetry from the downlink device and stores it in a postgres database.  The primary feature is adding a
timestamp to the data

There is a small serial protocol.

First, four X's
Then, the length of a packet (in u8)
Then the packet

## Future

* Store records in a persistent queue if either the network or remote postgres database unavailable.
* Send an acknowledgement
* Produce an integration test
    docker-compose -p to create a 'testing' docker-compose project
    will have to use current clock source
    write a file by hand.
* Using a rust downlink-firmware capture the signal strength.