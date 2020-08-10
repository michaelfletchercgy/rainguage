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

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS tip_cnt INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS temperature REAL", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS relative_humidity REAL", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS usb_bytes_read INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS usb_bytes_written INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_error_cnt INTEGER", &[]) {
        Ok(_) => { }, // cool!
        Err(err) => { error!("table create: error {:?}", err);}
    }

    match client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS hardware_error_other_cnt INTEGER", &[]) {
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
    DBError(postgres::Error)
}

impl From<std::io::Error> for ProcessError {
    fn from(_: std::io::Error) -> Self {
        ProcessError::IOError
    }
}

impl From<postgres::Error> for ProcessError {
    fn from(err: postgres::Error) -> Self {
        ProcessError::DBError(err)
    }
}

impl From<rainguage_messages::DeserializeError> for ProcessError {
    fn from(err: rainguage_messages::DeserializeError) -> Self {
        ProcessError::CorruptTelemetry(err)
    }
}
fn process(client: &mut Client, file:&File) -> Result<(),ProcessError> {    
    let mut buf:Vec<u8> = Vec::new();

    let bytes_iter = file.bytes()
        .map(|r| r.unwrap());
    let packet_iter = rainguage_messages::PacketIterator::new(bytes_iter);

    for packet in packet_iter {
        info!("received:{:?}", packet);
        match packet {
            Ok(packet) => {
                let now = chrono::Utc::now();
                
                client.execute("INSERT INTO telemetry (ts, vbat, loop_cnt, lora_rx_bytes, lora_tx_bytes, lora_error_cnt, tip_cnt, temperature, relative_humidity, usb_bytes_read, usb_bytes_written, usb_err_cnt, hardware_error_other_cnt)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)", 
                    &[  &now, 
                        &(packet.vbat as i32),
                        &(packet.loop_cnt as i32),
                        &(packet.lora_rx_bytes as i32),
                        &(packet.lora_tx_bytes as i32),
                        &(packet.lora_error_cnt as i32),
                        &(packet.tip_cnt as i32),
                        &(packet.temperature as f32),
                        &(packet.relative_humidity as f32),
                        &(packet.usb_bytes_read as i32),
                        &(packet.usb_bytes_written as i32),
                        &(packet.usb_error_cnt as i32),
                        &(packet.hardware_err_other_cnt as i32),
                        ])?;
                buf.clear();
            },
            Err(_err) => {
                
            }
        }
    }

   Ok(())
}