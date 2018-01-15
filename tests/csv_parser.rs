#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;

use std::sync::Arc;
use std::sync::Mutex;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::pipeline::{Pipeline, PipelineRunType, PipeLineElementIdentifier};
use liblasso::web;

use liblasso::elements::multiply_element::MultiplyElement;
use liblasso::elements::range_clamp_element::RangeClampElement;
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::file_source_element::FileSourceElement;
use liblasso::elements::csv_reader_element::CsvReaderElement;


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
fn csv_parser() {

    let path = get_data_dir().join("sacramento_crime_data.csv").to_str().unwrap().to_string();

    let mut file_src = FileSourceElement::new();
    file_src.set_property("path", &path);

    let mut echo_element = EchoSinkElement::new();
    let mut csv_reader_element = CsvReaderElement::new();

    let mut pipeline = Pipeline::new("csv reader pipeline".to_string(), PipelineRunType::MANUAL, true);

    let file_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(file_src)).unwrap();
    let csv_reader_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(csv_reader_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();

    pipeline.attach_output_pad_to_input_pad(file_src_ref,
                                            csv_reader_element_ref,
                                            "file_source_output",
                                            "csv_reader_input")
         .unwrap();

    pipeline.attach_output_pad_to_input_pad(csv_reader_element_ref,
                                            echo_element_ref,
                                            "csv_reader_output",
                                            "echo_element_input")
         .unwrap();

    let mut pipeline_container = PipelineContainer::new();
    let pipeline_name = pipeline.get_name().to_string();
    
    pipeline_container.add_pipeline(pipeline);

    crossbeam::scope(|scope| {

         pipeline_container.run(&pipeline_name);

         pipeline_container.wait_for_completion();
    });
}

