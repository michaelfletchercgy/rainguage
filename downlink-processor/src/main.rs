use dotenv::dotenv;
use dotenv::var;

use reqwest::blocking::Client;
use std::fs::File;

use std::io::Read;

#[macro_use]
extern crate log;

fn main() {
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();
    dotenv().ok();
    
    let file_name = &var("SERIAL_PORT").unwrap();
    let url =  &var("HTTP_UPLINK_URL").unwrap();

    loop {
        info!("Opening {}.  Hopefully you remembered to put it into raw mode.", file_name);
        let file = File::open(file_name).unwrap();

        info!("starting loop");

        let client = reqwest::blocking::Client::new();

        match process(&client, url, &file) {
            Err(err) => {
                error!("Handled error, resetting:{:?}", err);
            },
            Ok(_) => {
                info!("Processing completed.");
            }
        }
        info!("looping");
    }
}
#[derive(Debug)]
enum ProcessError {
    CorruptTelemetry(rainguage_messages::DeserializeError),
    IOError,
    HttpError(reqwest::Error)
}

impl From<std::io::Error> for ProcessError {
    fn from(_: std::io::Error) -> Self {
        ProcessError::IOError
    }
}

impl From<reqwest::Error> for ProcessError {
    fn from(err: reqwest::Error) -> Self {
        ProcessError::HttpError(err)
    }
}


impl From<rainguage_messages::DeserializeError> for ProcessError {
    fn from(err: rainguage_messages::DeserializeError) -> Self {
        ProcessError::CorruptTelemetry(err)
    }
}
fn process(client: &Client, url:&str, file:&File) -> Result<(),ProcessError> {    
    let bytes_iter = file.bytes()
        .map(|r| r.unwrap());
    let packet_iter = rainguage_messages::PacketIterator::new(bytes_iter);

    for packet in packet_iter {
        info!("received:{:?}, posting to {}", packet, url);
        match packet {
            Ok(packet) => {

                let res = client.post(url)
                    .json(&packet)
                    .send()?;
                
                    info!("response: {}", res.status());
                
            },
            Err(err) => {
                error!("Error receiving packet: {:?}", err)
            }
        }
    }

   Ok(())
}