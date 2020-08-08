use postgres::Config;
use postgres::Client;
use postgres::NoTls;

use std::fs::File;

use std::io::Read;

#[macro_use]
extern crate log;

fn main() {
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default());

    info!("Starting ...");
    let mut pg_config = Config::new();
    pg_config.host("localhost");
    pg_config.user("postgres");
    pg_config.password("supersecret99");
    pg_config.port(15432);
    let mut client = pg_config.connect(NoTls).unwrap();

    match client.execute("
        CREATE TABLE IF NOT EXISTS telemetry (
            id SERIAL NOT NULL,
            PRIMARY KEY (id),
            ts TIMESTAMP WITH TIME ZONE NOT NULL,
            loop_cnt INTEGER,
            vbat INTEGER,
            usb_int_cnt INTEGER,
            usb_ser_read INTEGER,
            usb_err_cnt INTEGER,
            lora_xmit_cnt INTEGER,
            device_id BYTEA
        );
    ", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry DROP COLUMN IF EXISTS lora_xmit_cnt", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_rx_bytes INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_tx_bytes INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }
    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS corrupt BOOL DEFAULT false", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }
    let file_name = "/dev/ttyACM0";

    loop {
        let file = File::open(file_name).unwrap();

        info!("starting loop");

        match process(&mut client, &file) {
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
    DBError
}

impl From<std::io::Error> for ProcessError {
    fn from(_: std::io::Error) -> Self {
        ProcessError::IOError
    }
}

impl From<postgres::Error> for ProcessError {
    fn from(_: postgres::Error) -> Self {
        ProcessError::DBError
    }
}

impl From<rainguage_messages::DeserializeError> for ProcessError {
    fn from(err: rainguage_messages::DeserializeError) -> Self {
        ProcessError::CorruptTelemetry(err)
    }
}
fn process(client: &mut Client, file:&File) -> Result<(),ProcessError> {    
    let mut x_count = 0;
    let mut len = 0;
    let mut buf:Vec<u8> = Vec::new();

    let bytes_iter = file.bytes()
        .map(|r| r.unwrap());
    let packet_iter = rainguage_messages::PacketIterator::new(bytes_iter);

    for packet in packet_iter {
        info!("received:{:?}", packet);
        match packet {
            Ok(packet) => {
                let now = chrono::Utc::now();
                
                client.execute("INSERT INTO telemetry (ts, vbat, loop_cnt, lora_tx_bytes) VALUES ($1, $2, $3, $4)", 
                    &[&now, &(packet.vbat as i32), &(packet.loop_cnt as i32), &(packet.lora_tx_bytes as i32)])?;
                buf.clear();
            },
            Err(err) => {
                
            }
        }
    }

   Ok(())
}

//#[cfg(test)]
// #[test]
// fn it_works() {
//     const TEST:&str = "loop=99600 vbat=1833, usb_int_cnt=0, usb_ser_read=0, usb_err_cnt=0, lora_xmit_cnt=0 id0=64947609 id1=504c5435 id2=382e3120 id3=ff0d3521\r";
//     let now = chrono::Utc::now();
//     let packet = parse_line(TEST, now).unwrap();

//     assert_eq!(now, packet.ts);
//     assert_eq!(1833, packet.vbat);
//     assert_eq!(99600, packet.loop_cnt);
// }

