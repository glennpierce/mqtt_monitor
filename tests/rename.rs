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


#[test]
fn rename() {

    let mut fake_src = RandomSourceElement::new();
    fake_src.set_property("minimum", "0.0");
    fake_src.set_property("maximum", "10.0");

    let mut multiply_element : MultiplyElement = MultiplyElement::new();
    multiply_element.set_property("factor", "2.0");

    let mut range_clamp_element : RangeClampElement = RangeClampElement::new();
    range_clamp_element.set_property("minimum", "12.0");
    range_clamp_element.set_property("maximum", "14.0");

    let mut echo_element = EchoSinkElement::new();
    let mut pipeline = Pipeline::new("example pipeline_copy1".to_string(), PipelineRunType::MANUAL, false);

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
  
    pipeline_container.copy_pipeline(&pipeline_name);
}

