#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;
extern crate ctrlc;

use std::sync::Arc;
use std::sync::Mutex;

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::pipeline::{Pipeline, PipelineRunType, PipeLineElementIdentifier};
use liblasso::web;

use liblasso::elements::multiply_element::MultiplyElement;
use liblasso::elements::range_clamp_element::RangeClampElement;
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;


fn two_source() -> Result<()> {

    let mut fake_src1 = RandomSourceElement::new();
    fake_src1.set_property("minimum", "0.0");
    fake_src1.set_property("maximum", "10.0");

    let mut fake_src2 = RandomSourceElement::new();
    fake_src2.set_property("minimum", "0.0");
    fake_src2.set_property("maximum", "10.0");

    let mut echo_element = EchoSinkElement::new();
    let mut pipeline = Pipeline::new("two source pipeline".to_string(), PipelineRunType::SCHEDULED, false);

    let fake_src1_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(fake_src1)).unwrap();
    let fake_src2_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(fake_src2)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    
    pipeline.attach_output_pad_to_input_pad(fake_src1_ref,
                                        echo_element_ref,
                                        "random_source_output1",
                                        "echo_element_input1")
        .unwrap();

    pipeline.attach_output_pad_to_input_pad(fake_src2_ref,
                                        echo_element_ref,
                                        "random_source_output2",
                                        "echo_element_input2")
        .unwrap();



    pipeline.set_schedule_time("2 * * * * * *");

    let mut pipeline_container = PipelineContainer::new();

    let pipeline_name = pipeline.get_name().to_string();

    pipeline_container.add_pipeline(pipeline);

    pipeline_container.serialize().unwrap();

    //pipeline_container.deserialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();
        let container2 = pipeline_container.clone();
        let container3 = pipeline_container.clone();

        // scope.spawn(move || {
        //     web::serve(Arc::new(Mutex::new(container)));
        // });

        scope.spawn(move || {
            container2.clone().run(&pipeline_name);
        });

        ctrlc::set_handler(move || {
            
            container3.stop_pipelines();
            std::process::exit(0);

        }).expect("Error setting Ctrl-C handler");
        
        pipeline_container.wait_for_completion();
    });

    Ok(())


}


#[test]
fn two_source_main() {

    if let Err(ref e) = two_source() {

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