#![allow(warnings)]

extern crate failure;
extern crate rusqlite;

use failure::{Error, err_msg};

extern crate pretty_env_logger;
extern crate ctrlc;
extern crate chrono;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use std::collections::HashMap;

use std::io;
extern crate fern;
extern crate clap;

extern crate rumqtt;

use std::{thread, time};
use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use rumqtt::{MqttOptions, MqttClient, MqttCallback, Message, QoS};
use rusqlite::Connection;

use clap::{Arg, App};

fn setup_logging(verbosity: u8) -> std::prelude::v1::Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => {
            // Let's say we depend on something which whose "info" level messages are too verbose
            // to include in end-user output. If we don't need them, let's not include them.
            base_config
                .level(log::LevelFilter::Info)
                .level_for("overly-verbose-target", log::LevelFilter::Warn)
        }
        1 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("overly-verbose-target", log::LevelFilter::Info),
        2 => base_config.level(log::LevelFilter::Debug),
        _3_or_more => base_config.level(log::LevelFilter::Trace),
    };

    base_config.chain(io::stdout()).apply()?;

    Ok(())
}

#[derive(Debug)]
struct Reading {
    topic: String,
    datetime: i32,
    value: f64
}

fn run() -> Result<(), Error> {

    let matches = App::new("Mqtt-Monitor")
                          .version("0.1")
                          .author("Glenn Pierce <glennpierce@gmail.com>")
                          .about("Mqtt-Monitor")

                           .arg(
                                Arg::with_name("verbose")
                                    .short("v")
                                    .long("verbose")
                                    .multiple(true)
                                    .help("Increases logging verbosity each use for up to 3 times"),
                            )


                          .get_matches();

    let verbosity: u8 = matches.occurrences_of("verbose");

    setup_logging(verbosity).expect("failed to initialize logging.");

    info!("Mqtt-Monitor v0.0.1 starting up!");
    debug!("DEBUG output enabled.");
    trace!("TRACE output enabled.");
    info!(target: "overly-verbose-target", "hey, another library here, we're starting.");

    let conn = Connection::open_in_memory().unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS topics(id INTEGER PRIMARY KEY AUTOINCREMENT,
                    topic TEXT NOT NULL,
                    CONSTRAINT unique_topic UNIQUE (topic)
                   )", &[]).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS reading (
                  topic           INTEGER PRIMARY KEY,
                  datetime        INTEGER,
                  value           REAL,
                  FOREIGN KEY(topic) REFERENCES topics(id)
                  )", &[]).unwrap();

    conn.close();

    let client_options = MqttOptions::new()
                                        .set_keep_alive(5)
                                        .set_reconnect(3)
                                        .set_client_id("pierce_house-mqtt")
                                        .set_broker("192.168.1.7:1883");

    let msg_current_callback_fn = move |message : Message| {

        let topic = message.topic.to_string();
        let payload = Arc::try_unwrap(message.payload).unwrap();
        let s = String::from_utf8_lossy(&payload);

        //print!("{}: ", topic);
        //println!("{}", s.to_string());

        // let reading = Reading {
        //     topic: 0,
        //     datetime: "Steven".to_string(),
        //     value: time::get_time()
        // };

        let conn = Connection::open_in_memory().unwrap();

        conn.execute("INSERT OR IGNORE INTO topics(topic) VALUES('?1')", &[&topic]).unwrap();

        //SELECT id FROM "Values" WHERE data = 'SOME_DATA';

        // conn.execute("INSERT INTO person (name, time_created, data)
        //             VALUES (?1, ?2, ?3)",
        //             &[&me.name, &me.time_created, &me.data]).unwrap();

        // let current: f32 = s.parse().unwrap();
        // let power = current * 248.0;
        // let watt_payload = format!("{}", power);
        // request.publish("test/basic", QoS::Level0, watt_payload.into_bytes()).expect("Publish failure");

    };

    let msg_callback = MqttCallback::new().on_message(msg_current_callback_fn);

    let mut request = MqttClient::start(client_options, Some(msg_callback)).expect("Coudn't start");

    let topics = vec![("wemos/+/current/status", QoS::Level0)];
    request.subscribe(topics).expect("Subcription failure");

    while true {
        thread::sleep(time::Duration::from_millis(100));
    }

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {

        println!("{}", e.backtrace());

        std::process::exit(1);
    }

   
}