#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[macro_use] extern crate log;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;

use dotenv::dotenv;

use rainguage_messages::TelemetryPacket;
use rocket_contrib::json::Json;
use rocket::State;
mod persister;

#[post("/telemetry", format = "json", data = "<packet>")]
fn post(packet:Json<TelemetryPacket>, tx:State<SyncSender<TelemetryPacket>>) {
    tx.send(packet.into_inner()).unwrap();
}

fn main() {
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();
    dotenv().ok();

    let (tx, rx) = sync_channel::<TelemetryPacket>(32);

    persister::start(rx);

    info!("Starting ...");

    rocket::ignite()
        .manage(tx)
        .mount("/", routes![post])
        .launch();
}