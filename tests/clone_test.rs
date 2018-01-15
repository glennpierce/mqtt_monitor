#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;

#[macro_use]
extern crate liblasso;
extern crate ctrlc;

#[macro_use]
extern crate serde_json;

use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::pipeline::{Pipeline, PipelineRunType, PipeLineElementIdentifier};
use liblasso::web;

use liblasso::elements::multiply_element::MultiplyElement;
use liblasso::elements::range_clamp_element::RangeClampElement;
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::stats_element::StatsElement;
use liblasso::elements::csv_file_element::CsvFileElement;


fn clone_box_test() -> Result<()> {

    let mut echo_element = EchoSinkElement::new();
    let mut pipeline = Pipeline::new("clone box test pipeline".to_string(), PipelineRunType::MANUAL, false);
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
             
    let pipeline_name = pipeline.get_name().to_string();
    
    println!("Before pipeline clone");
    let mut pipeline2 = pipeline.clone();
    println!("After pipeline clone");

    {
        let mut element_data = pipeline.get_element_data_mut(echo_element_ref);
        element_data.unwrap().store_setting("TestSetting", json!("Cool"));
    }

    {
        let element_data2 = pipeline.get_element_data(echo_element_ref);
        let result = element_data2.unwrap().load_setting("TestSetting");

        assert_eq!(result, Some(json!("Cool")));
    }

    {
        let element_data3 = pipeline2.get_element_data(echo_element_ref);
        let result = element_data3.unwrap().load_setting("TestSetting");

        assert_eq!(result, Some(json!("Cool")));
    }

    {
        let mut pipeline_container = PipelineContainer::new();
        pipeline_container.add_pipeline(pipeline);
        let pipeline_container2 = pipeline_container.clone();

        let element= pipeline_container.get_pipeline_element(&pipeline_name, echo_element_ref);
        let element_data4 = pipeline2.get_element_data(echo_element_ref);
        let result = element_data4.unwrap().load_setting("TestSetting");

        assert_eq!(result, Some(json!("Cool")));

    }


    Ok(())
}


#[test]
fn clone_box_test1() {

    pretty_env_logger::init().unwrap();

    if let Err(ref e) = clone_box_test() {

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