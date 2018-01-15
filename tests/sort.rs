#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::io::Read;

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::web;

// Get absolute path to the "target" directory ("build" dir)
fn get_target_dir() -> PathBuf {
    let bin = std::env::current_exe().expect("exe path");
    let mut target_dir = PathBuf::from(bin.parent().expect("bin parent"));
    while target_dir.file_name() != Some(std::ffi::OsStr::new("target")) {
        target_dir.pop();
    }
    target_dir
}

fn get_data_dir() -> PathBuf {
    let target_dir = get_target_dir();
    let path = target_dir.join("../data");
    path
}

#[test]
fn sort() {

    let path = get_data_dir().join("table_names.txt").to_str().unwrap().to_string();
    //let display = path.display();

    let mut file = match File::open(&path)
    {
        Err(why) => panic!("couldn't open {}", path),
        Ok(file) => file,
    };

    let mut buffer = String::new();

    file.read_to_string(&mut buffer).unwrap();

    let mut data: Vec<String> = buffer.lines_any().filter_map(|s| Some(s.to_lowercase().trim().to_string())).collect();

    println!("{:?}", data);
    println!("\n");
    //data.sort();

    data.sort_by(|a, b| b.to_lowercase().cmp(&a.to_lowercase()).reverse());

    println!("{:?}", data);
}

