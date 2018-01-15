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
use liblasso::elements::sftp_sink_element::SFtpSinkElement;
use liblasso::elements::duplicate_element::DuplicateElement;

fn engie_victoria_ftp_test() -> Result<()> {

    let mut db_src = HistoryTrendRecordSourceElement::new();
    db_src.set_property("host", "127.0.0.1");
    db_src.set_property("port", "3306");
    db_src.set_property("user", "niagara");
    db_src.set_property("database", "engie");
    db_src.set_property("password", "fhd732n4")?;
    db_src.set_property("tables", "HISTORYNUMERICTRENDRECORD,HISTORYBOOLEANTRENDRECORD")?;
    db_src.set_property("filter", "/CARNEGO_VICTORIA%")?;

    let mut csv_file_element = CsvFileElement::new();
    csv_file_element.set_property("datetime_format", "%d/%m/%Y  %H:%M:%S")?;
    csv_file_element.set_property("filename_prefix", "CARNEGO_VICTORIA_")?;

    let mut echo_element = EchoSinkElement::new();

    let duplicate_element = DuplicateElement::new();

    let mut sftp_sink_element = SFtpSinkElement::new();

    // sftp -vv -i .ssh/id1_ftp_rsa GENIETest1@10.200.9.10
    sftp_sink_element.set_property("host", "10.200.9.10")?;
    sftp_sink_element.set_property("port", "22")?;
    sftp_sink_element.set_property("user", "GENIETest2")?;
    //sftp_sink_element.set_property("password", "P97H2M28757UxoT")?;
    sftp_sink_element.set_property("key", "id2_ftp_rsa")?;
    sftp_sink_element.set_property("working_directory", "PlymouthGenie")?;
    sftp_sink_element.set_property("fake", "false")?;

    let mut pipeline = Pipeline::new("engie_victoria".to_string(), PipelineRunType::SCHEDULED, true);

    let db_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(db_src)).unwrap();
    let csv_file_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(csv_file_element)).unwrap();
    let duplicate_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(duplicate_element)).unwrap();
    //let ftp_sink_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(ftp_sink_element)).unwrap();
    let sftp_sink_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(sftp_sink_element)).unwrap();
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
                                                       sftp_sink_element_ref,
                                                       ElementChannelType::CHANNEL2,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();


    pipeline.set_schedule_time("0 * * * * *");

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

        ctrlc::set_handler(move || {
            
            container3.stop_pipelines();
            std::process::exit(0);

        }).expect("Error setting Ctrl-C handler");
        
        pipeline_container.run_pipelines(true);
    });

    Ok(())
}



#[test]
fn engie_victoria_ftp_test1() {

    pretty_env_logger::init().unwrap();

    println!("running engie victoria ftp");
    if let Err(ref e) = engie_victoria_ftp_test() {

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
