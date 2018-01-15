#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;

use std::sync::Arc;
use std::sync::Mutex;

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::web;

#[test]
fn deserialize() {

    pretty_env_logger::init().unwrap();

    let mut pipeline_container = PipelineContainer::new();
    pipeline_container.deserialize().unwrap();

    crossbeam::scope(|scope| {

        let mut container = pipeline_container.clone();
        let container2 = pipeline_container.clone();

        container.run_pipelines(true);

        scope.spawn(move || {
            web::serve(Arc::new(Mutex::new(container2)));
        });

        pipeline_container.wait_for_completion();
    });
}

