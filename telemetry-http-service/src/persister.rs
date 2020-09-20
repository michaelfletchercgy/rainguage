use std::sync::mpsc::Receiver;
use std::thread;

use dotenv::var;

use postgres::Config;
use postgres::NoTls;

use rainguage_messages::TelemetryPacket;

#[derive(Debug)]
enum WriterError {
    PostgresError(postgres::Error)
}

impl From<postgres::Error> for WriterError {
    fn from(err: postgres::Error) -> Self {
        WriterError::PostgresError(err)
    }
}

pub fn start(rx:Receiver<TelemetryPacket>) {
    let config = config();
    init_database(&config).expect("Failed to initialize database.");

    thread::spawn(move|| {
        loop {
            match rx.recv() {
                Ok(packet) => {
                    write_to_database_reliably(&packet, &config);
                },
                Err(err) => {
                    error!("Error receiving messages:{:?}", err);
                }
            }
        }
    });
}

fn config() -> Config {
    let mut pg_config = Config::new();
    pg_config.host(&var("POSTGRES_HOST").unwrap());
    pg_config.user(&var("POSTGRES_USER").unwrap());
    pg_config.password(&var("POSTGRES_PASSWORD").unwrap());
    pg_config.port(var("POSTGRES_PORT").unwrap().parse::<u16>().unwrap());

    pg_config
}

fn init_database(config:&Config) -> Result<(), WriterError> {
    let mut client = config.connect(NoTls)?;

    client.execute("
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
    ", &[])?;

    client.execute("ALTER TABLE telemetry DROP COLUMN IF EXISTS lora_xmit_cnt", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_rx_bytes INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_tx_bytes INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS corrupt BOOL DEFAULT false", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS tip_cnt INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS temperature REAL", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS relative_humidity REAL", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS usb_bytes_read INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS usb_bytes_written INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS lora_error_cnt INTEGER", &[])?;
    client.execute("ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS hardware_error_other_cnt INTEGER", &[])?;

    Ok(())
}

fn write_to_database_reliably(packet:&TelemetryPacket, config:&Config) {
    loop {
        match write_to_database(packet, config) {
            Ok(_) => {
                return;
            },
            Err(err) => {
                error!("Could not write, will retry: {:?}", err);
            }
        }
    }
}


fn write_to_database(packet:&TelemetryPacket, config:&Config) -> Result<(),WriterError> {
    let mut client = config.connect(NoTls).unwrap();
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

    Ok(())
}