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
use liblasso::elements::bmos_postgres_source_element::BmosPostgresSourceElement;
use liblasso::elements::stats_element::StatsElement;
use liblasso::elements::csv_file_element::CsvFileElement;
use liblasso::elements::postgres_table_sink_element::PostgresTableSinkElement;
use liblasso::elements::duplicate_element::DuplicateElement;

fn bmos_table_test() -> Result<()> {

    let mut db_src = BmosPostgresSourceElement::new();
    db_src.set_property("host", "carneab4.memset.net");
    db_src.set_property("port", "5432");
    db_src.set_property("user", "glenn");
    db_src.set_property("database", "bmos");
    db_src.set_property("password", "2dd~Pabq")?;
    db_src.set_property("start_time", "2017-06-30T14:00:00+00:00")?;
    db_src.set_property("aggregate_values", "AVG, SUM, MAX, MIN, MIN_MAX_DIFF")?;
    db_src.set_property("time_granularity", "DAY")?;
    db_src.set_property("sensor_name_like", "bi1%")?;
    db_src.set_property("sensor_namespace_like", "elmsbrook_000.bi%")?;

    let mut echo_element = EchoSinkElement::new();
    let duplicate_element = DuplicateElement::new();

    // let mut bmos_table_sink_element = PostgresTableSinkElement::new();

    // bmos_table_sink_element.set_property("host", "carneab4.memset.net");
    // bmos_table_sink_element.set_property("port", "5432");
    // bmos_table_sink_element.set_property("user", "glenn");
    // bmos_table_sink_element.set_property("database", "bmos");
    // bmos_table_sink_element.set_property("password", "2dd~Pabq")?;
    // bmos_table_sink_element.set_property("table", "sensor_values_days")?;
    // bmos_table_sink_element.set_property("columns", "ts, sensor_id, min, max, sum, avg, used")?;

    // bmos_table_sink_element.set_property("table_create_sql",
    //                         "CREATE TABLE IF NOT EXISTS sensor_values_days 
    //                           (ts timestamp with time zone NOT NULL, 
    //                           min double precision NOT NULL DEFAULT 'NaN'::real,
    //                           max double precision NOT NULL DEFAULT 'NaN'::real,
    //                           sum double precision NOT NULL DEFAULT 'NaN'::real,
    //                           avg double precision NOT NULL DEFAULT 'NaN'::real,
    //                           used double precision NOT NULL DEFAULT 'NaN'::real,
    //                           sensor_id integer NOT NULL, 
    //                           FOREIGN KEY (sensor_id) REFERENCES sensors(id),
    //                           CONSTRAINT timestamp_id_index UNIQUE(ts, sensor_id)")?;

    let mut pipeline = Pipeline::new("bmos_table_test".to_string(), PipelineRunType::SCHEDULED, true);

    let db_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(db_src)).unwrap();
    let duplicate_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(duplicate_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    //let bmos_table_sink_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(bmos_table_sink_element)).unwrap();
             

    pipeline.attach_output_pad_to_input_pad_by_channel(db_src_ref,
                                                       duplicate_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
                                                       echo_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    // pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
    //                                                    bmos_table_sink_element_ref,
    //                                                    ElementChannelType::CHANNEL2,
    //                                                    ElementChannelType::CHANNEL1)
    //     .unwrap();


    pipeline.set_schedule_time("* * * * * *");

    let mut pipeline_container = PipelineContainer::new();
    let pipeline_name = pipeline.get_name().to_string();

    pipeline_container.add_pipeline(pipeline);
    pipeline_container.serialize().unwrap();


    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();
        let container2 = pipeline_container.clone();
        let container3 = pipeline_container.clone();

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
fn bmos_table_test1() {

    if let Err(ref e) = bmos_table_test() {

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