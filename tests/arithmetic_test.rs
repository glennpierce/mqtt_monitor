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
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::arithmetic_element::ArithmeticElement;


fn arithmetic_test() -> Result<()> {

    let mut fake_src = RandomSourceElement::new();
    fake_src.set_property("minimum", "0.0");
    fake_src.set_property("maximum", "10.0");

    let mut arithmetic_element : ArithmeticElement = ArithmeticElement::new();
    arithmetic_element.set_property("eval_string", "$1,$1/2,$1*2 + 10.5");

    let mut echo_element = EchoSinkElement::new();
    let mut pipeline = Pipeline::new("arithmetic pipeline".to_string(), PipelineRunType::MANUAL, false);

    let fake_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(fake_src)).unwrap();
    let arithmetic_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(arithmetic_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    
    pipeline.attach_output_pad_to_input_pad_by_channel(fake_src_ref,
                                                       arithmetic_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(arithmetic_element_ref,
                                                       echo_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    let mut pipeline_container = PipelineContainer::new();

    let pipeline_name = pipeline.get_name().to_string();
    
    pipeline_container.add_pipeline(pipeline);

    //pipeline_container.serialize().unwrap();

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
fn arithmetic_test1() {

    if let Err(ref e) = arithmetic_test() {

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