#![allow(warnings)]
// error_chain! can recurse deeply
#![recursion_limit = "1024"]

extern crate pretty_env_logger;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate bitflags;

#[macro_use(c)]
extern crate cute;

extern crate ctrlc;
extern crate time;
extern crate chrono;
extern crate crossbeam;
extern crate itertools;
extern crate futures;
extern crate hyper;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate params;
extern crate bodyparser;
extern crate router;
extern crate url;
extern crate rand;
extern crate app_dirs;
extern crate ws;
extern crate spmc;
extern crate csv;
extern crate base64;
extern crate toml;
extern crate postgres;
extern crate cron;
extern crate schedule;
extern crate ftp;
extern crate ssh2;
extern crate backtrace;
extern crate meval;
extern crate rumqtt;

#[macro_use]
extern crate indoc;

//#[macro_use]
//extern crate runtime_fmt;

#[macro_use]
extern crate mysql;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::rc::Weak;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod errors;

#[macro_use]
pub mod element;

mod thread_control;
mod channel_manager;

pub mod element_factory;
pub mod elements;
pub mod pipeline;
pub mod pipeline_container;
pub mod web;

use element::*;
use errors::*;

extern crate crossbeam;
extern crate liblasso;
extern crate fern;
extern crate clap;

#[macro_use]
extern crate log;

use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::web;

use clap::{Arg, App};

fn setup_logging(verbosity: u8) -> std::prelude::v1::Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => {
            // Let's say we depend on something which whose "info" level messages are too verbose
            // to include in end-user output. If we don't need them, let's not include them.
            base_config
                .level(log::LogLevelFilter::Info)
                .level_for("overly-verbose-target", log::LogLevelFilter::Warn)
        }
        1 => base_config
            .level(log::LogLevelFilter::Debug)
            .level_for("overly-verbose-target", log::LogLevelFilter::Info),
        2 => base_config.level(log::LogLevelFilter::Debug),
        _3_or_more => base_config.level(log::LogLevelFilter::Trace),
    };

    // Separate file config so we can include year, month and day in file logs
    // let file_config = fern::Dispatch::new()
    //     .format(|out, message, record| {
    //         out.finish(format_args!(
    //             "{}[{}][{}] {}",
    //             chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
    //             record.target(),
    //             record.level(),
    //             message
    //         ))
    //     })
    //     .chain(fern::log_file("program.log")?);

    // let stdout_config = fern::Dispatch::new()
    //     .format(|out, message, record| {
    //         // special format for debug messages coming from our own crate.
    //         if record.level() > log::LogLevelFilter::Info && record.target() == "cmd_program" {
    //             out.finish(format_args!("---\nDEBUG: {}: {}\n---",
    //                                     chrono::Local::now().format("%H:%M:%S"),
    //                                     message))
    //         } else {
    //             out.finish(format_args!("[{}][{}][{}] {}",
    //                                     chrono::Local::now().format("%H:%M"),
    //                                     record.target(),
    //                                     record.level(),
    //                                     message))
    //         }
    //     })
    //     .chain(io::stdout());

    //base_config.chain(file_config).chain(stdout_config).apply()?;

    base_config.chain(io::stdout()).apply()?;

    Ok(())
}

fn run() -> Result<()> {

    let matches = App::new("lasso")
                          .version("0.1")
                          .author("Glenn Pierce <glennpierce@gmail.com>")
                          .about("Lasso data pipeline processor")
                        //   .arg(Arg::with_name("slave")
                        //        .short("s")
                        //        .long("slave")
                        //        //.index(1)
                        //        .help("Is this server a slave ?")
                        //        .takes_value(false))
                        //        //.required(true))


                          .arg(Arg::with_name("CONFIG")
                               .short("c")
                               .long("config")
                               .help("Path to the config.toml")
                               .takes_value(true))

                           .arg(Arg::with_name("sync")
                               .short("s")
                               .long("synchronous")
                               .help("Run multiple pipelines in a synchronous fashion")
                               .takes_value(false))

                            .arg(Arg::with_name("pipeline")
                               .short("p")
                               .long("pipeline")
                               .help("Run pipeline with specified name")
                               .takes_value(true))

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

    let is_sync: bool = if matches.is_present("sync") { true } else { false };

    let mut pipeline_name : Option<String> = None;
    if matches.is_present("pipeline") {
        pipeline_name = Some(matches.value_of("pipeline").unwrap().to_string());
    }

    info!("Lasso v0.0.1 starting up!");
    debug!("DEBUG output enabled.");
    trace!("TRACE output enabled.");
    info!(target: "overly-verbose-target", "hey, another library here, we're starting.");

    let mut pipeline_container = PipelineContainer::new();
    pipeline_container.deserialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();

        scope.spawn(move || {
            web::serve(Arc::new(Mutex::new(container)));
        });

        println!("Starting lasso");

        if pipeline_name.is_some() {
            pipeline_container.run(&pipeline_name.unwrap());
        }
        else {
            if is_sync {
                info!("running pipelines in synchronous mode");
            }
            else {
                info!("running pipelines in asynchronous mode");
            }

            pipeline_container.run_pipelines(is_sync);
        }
    });

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}


