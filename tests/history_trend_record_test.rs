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

use liblasso::elements::multiply_element::MultiplyElement;
use liblasso::elements::range_clamp_element::RangeClampElement;
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::history_trend_record_source_element::HistoryTrendRecordSourceElement;
use liblasso::elements::stats_element::StatsElement;
use liblasso::elements::csv_file_element::CsvFileElement;
use liblasso::elements::ftp_sink_element::FtpSinkElement;
use liblasso::elements::duplicate_element::DuplicateElement;

fn history_trend_record() -> Result<()> {

    let mut db_src = HistoryTrendRecordSourceElement::new();
    db_src.set_property("host", "carneab4.memset.net");
    db_src.set_property("port", "3306");
    db_src.set_property("user", "niagara");
    db_src.set_property("database", "carnego");
    db_src.set_property("password", "hv52!522g89H")?;

    let mut csv_file_element = CsvFileElement::new();

    let mut echo_element = EchoSinkElement::new();

    let duplicate_element = DuplicateElement::new();

    let mut ftp_sink_element = FtpSinkElement::new();

    ftp_sink_element.set_property("host", "ftp.c3ntinel.com")?;
    ftp_sink_element.set_property("port", "21")?;
    ftp_sink_element.set_property("user", "FTPTEST")?;
    ftp_sink_element.set_property("password", "P97H2M28757UxoT")?;
    ftp_sink_element.set_property("working_directory", "PlymouthGenie")?;
    ftp_sink_element.set_property("fake", "true")?;

    let mut pipeline = Pipeline::new("history_trend_record pipeline".to_string(), PipelineRunType::MANUAL, false);

    let db_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(db_src)).unwrap();
    let csv_file_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(csv_file_element)).unwrap();
    let duplicate_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(duplicate_element)).unwrap();
    let ftp_sink_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(ftp_sink_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
             
    pipeline.attach_output_pad_to_input_pad_by_channel(db_src_ref,
                                                       csv_file_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(csv_file_element_ref,
                                                       duplicate_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
                                                       echo_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
                                                       ftp_sink_element_ref,
                                                       ElementChannelType::CHANNEL2,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();


    pipeline.set_schedule_time("* */2 * * * * *");

    let mut pipeline_container = PipelineContainer::new();
    let pipeline_name = pipeline.get_name().to_string();

    pipeline_container.add_pipeline(pipeline);
    pipeline_container.serialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();
        let container2 = pipeline_container.clone();
        let container3 = pipeline_container.clone();

        scope.spawn(move || {
            web::serve(Arc::new(Mutex::new(container)));
        });

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
fn history_trend_record1() {

    pretty_env_logger::init().unwrap();

    if let Err(ref e) = history_trend_record() {

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