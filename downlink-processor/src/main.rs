use postgres::Config;
use postgres::Client;
use postgres::NoTls;

use std::fs::File;

use std::io::Read;
use chrono::DateTime;
use chrono::Utc;

#[macro_use]
extern crate log;

struct TelemetryPacket {
    ts:chrono::DateTime<chrono::Utc>,
    vbat: u32,
    loop_cnt: u32
}
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
    let file_name = "/dev/ttyACM0";

    let file = File::open(file_name).unwrap();

    loop {
        match process(&mut client, &file) {
            Err(_) => {
                error!("Handled error, resetting.");
            },
            Ok(_) => {
                info!("Processing completed.");
                return;
            }
        }
    }
}
#[derive(Debug)]
enum ProcessError {
    CorruptTelemetry,
    IOError,
    DBError
}

impl From<std::string::FromUtf8Error> for ProcessError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        ProcessError::CorruptTelemetry
    }
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

impl From<std::num::ParseIntError> for ProcessError {
    fn from(_: std::num::ParseIntError) -> Self {
        ProcessError::CorruptTelemetry
    }
}

fn process(client: &mut Client, file:& File) -> Result<(),ProcessError> {
    let newline = 10;
    let mut buf:Vec<u8> = Vec::new();
    for b_res in file.bytes() {
        let b = b_res?;
        if b == newline {
            let string = String::from_utf8(buf.clone())?;
            let now = chrono::Utc::now();

            info!("received telemetry");
            let packet = parse_line(string.as_str(), now)?;
            client.execute("INSERT INTO telemetry (ts, vbat) VALUES ($1, $2)", &[&packet.ts, &(packet.vbat as i32)])?;

            buf.clear();
        } else {
            buf.push(b);
        }
    }

    Ok(())
}
fn parse_line(line: &str, now:DateTime<Utc>) -> Result<TelemetryPacket, ProcessError> {
    let mut vbat = 0;
    let mut loop_cnt = 0;

    for part in line.split(" ") {
        let cleaned_part = if part.ends_with(",") {
            &part[..part.len() - 1]
        } else {
            &part
        };

        let pieces:Vec<&str> = cleaned_part.split("=").collect();
        let name = pieces.get(0).unwrap_or_else(|| &"");
        let value = pieces.get(1).unwrap_or_else(|| &"");

        match name {
            &"vbat" => vbat = value.parse()?,
            &"loop" => loop_cnt = value.parse()?,
            _ => { } // ignore everything else
        }
    }
    Ok(TelemetryPacket {
        ts:now,
        vbat:vbat,
        loop_cnt: loop_cnt
    })
} 

#[cfg(test)]
#[test]
fn it_works() {
    const TEST:&str = "loop=99600 vbat=1833, usb_int_cnt=0, usb_ser_read=0, usb_err_cnt=0, lora_xmit_cnt=0 id0=64947609 id1=504c5435 id2=382e3120 id3=ff0d3521\r";
    let now = chrono::Utc::now();
    let packet = parse_line(TEST, now).unwrap();

    assert_eq!(now, packet.ts);
    assert_eq!(1833, packet.vbat);
    assert_eq!(99600, packet.loop_cnt);
}

