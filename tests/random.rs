#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;

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
use liblasso::elements::random_timestamped_source_element::RandomTimestampedSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;


fn random() -> Result<()> {

    let mut fake_src = RandomTimestampedSourceElement::new();
    fake_src.set_property("minimum", "0.0");
    fake_src.set_property("maximum", "10.0");

    let mut multiply_element : MultiplyElement = MultiplyElement::new();
    multiply_element.set_property("factor", "2.0");

    let mut range_clamp_element : RangeClampElement = RangeClampElement::new();
    range_clamp_element.set_property("minimum", "12.0");
    range_clamp_element.set_property("maximum", "14.0");

    let mut echo_element = EchoSinkElement::new();
    let mut pipeline = Pipeline::new("example pipeline".to_string(), PipelineRunType::MANUAL, false);

    let fake_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(fake_src)).unwrap();
    let multiply_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(multiply_element)).unwrap();
    let range_clamp_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(range_clamp_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    
    pipeline.attach_output_pad_to_input_pad(fake_src_ref,
                                        multiply_element_ref,
                                        "random_source_output1",
                                        "multiply_element_input")
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(multiply_element_ref,
                                                        range_clamp_element_ref,
                                                        ElementChannelType::CHANNEL1,
                                                        ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad(range_clamp_element_ref,
                                        echo_element_ref,
                                        "range_clamp_element_output",
                                        "echo_element_input1")
        .unwrap();


    let mut pipeline_container = PipelineContainer::new();

    let pipeline_name = pipeline.get_name().to_string();
    
    pipeline_container.add_pipeline(pipeline);

    pipeline_container.serialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();

        scope.spawn(move || {
            web::serve(Arc::new(Mutex::new(container)));
        });


        scope.spawn(move || {
            pipeline_container.clone().run(&pipeline_name);
        });

      
        loop {
            thread::sleep(time::Duration::from_millis(1000));
        }
    });

    Ok(())
}



#[test]
fn random1() {

    if let Err(ref e) = random() {

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