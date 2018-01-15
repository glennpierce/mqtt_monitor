#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;
extern crate ctrlc;

use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::pipeline::{Pipeline, PipelineRunType, PipeLineElementIdentifier};
use liblasso::web;

use liblasso::elements::counter_source_element::CounterSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;

fn new_schedule_pipeline(pipeline_name : &str) -> Pipeline {

    let mut src = CounterSourceElement::new();
    src.set_property("maximum", "30");

    let mut echo_element = EchoSinkElement::new();

    let mut pipeline = Pipeline::new(pipeline_name.to_string(), PipelineRunType::SCHEDULED, true);

    let src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(src)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
             
    pipeline.attach_output_pad_to_input_pad_by_channel(src_ref,
                                                       echo_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.set_schedule_time("0 * * * * *");

    pipeline
}

fn run_pipelines() -> Result<()> {

    let mut pipeline_container = PipelineContainer::new();

    pipeline_container.add_pipeline(new_schedule_pipeline("schedule_test_pipeline1"));
    pipeline_container.add_pipeline(new_schedule_pipeline("schedule_test_pipeline2"));
    pipeline_container.add_pipeline(new_schedule_pipeline("schedule_test_pipeline3"));
    pipeline_container.add_pipeline(new_schedule_pipeline("schedule_test_pipeline4"));
    pipeline_container.add_pipeline(new_schedule_pipeline("schedule_test_pipeline5"));
    pipeline_container.serialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();
        let container2 = pipeline_container.clone();
        let container3 = pipeline_container.clone();

        ctrlc::set_handler(move || {
            
            container2.stop_pipelines();
            std::process::exit(0);

        }).expect("Error setting Ctrl-C handler");
        
        pipeline_container.run_pipelines(true);
    });

    Ok(())
}


#[test]
fn schedule_test1() {

    pretty_env_logger::init().unwrap();

    if let Err(ref e) = run_pipelines() {

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