# downlink-processor

Processes raw telemetry from the downlink device and stores it in a postgres database.  The primary feature is adding a
timestamp to the data


## Future

* Store records in a persistent queue if either the network or remote postgres database unavailable.
* Send an acknowle