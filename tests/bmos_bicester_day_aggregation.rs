#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;
extern crate ctrlc;
extern crate chrono;

use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use chrono::{Utc, Duration, Local, FixedOffset, DateTime, NaiveDateTime, NaiveTime, NaiveDate, Timelike, TimeZone};
use chrono::Datelike;

use liblasso::errors::*;
use liblasso::element::*;
use liblasso::pipeline_container::PipelineContainer;
use liblasso::pipeline::{Pipeline, PipelineRunType, PipeLineElementIdentifier};
use liblasso::web;

use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::postgres_source_element::PostgresSourceElement;
use liblasso::elements::csv_file_element::CsvFileElement;
use liblasso::elements::bmos_table_sink_element::BmosTableSinkElement;
use liblasso::elements::duplicate_element::DuplicateElement;
use liblasso::elements::null_sink_element::NullSinkElement;
use liblasso::elements::data_convert_element::DataConvertElement;
use liblasso::elements::postgres_table_sink_element::PostgresTableSinkElement;

fn bicester_day_aggregation() -> Result<()> {

    let mut db_src = PostgresSourceElement::new();
    db_src.set_property("host", "carneab4.memset.net");
    db_src.set_property("port", "5432");
    db_src.set_property("user", "glenn");
    db_src.set_property("database", "bmos");
    db_src.set_property("password", "2dd~Pabq")?;

    let start : DateTime<Utc> = Utc::now() - Duration::days(7);
    let floored_start = Utc.ymd(start.year(), start.month(), start.day()).and_hms(0, 0, 0);

    db_src.set_property("start_time", &floored_start.to_rfc3339())?;
    db_src.set_property("timestamp_increment_seconds", "86400")?;   // 3600 * 24

    db_src.set_property("select_sql", "SELECT MAX(sensor_id)::text as sensor_id, 
                                              name::text as name,
                                              split_part(name, '_', 2)::text as sensor_type,
                                              date_trunc('day', ts)::date::text as day,
                                              (MAX(value) - MIN(CASE WHEN value = 0 THEN NULL ELSE value END))::text as day_value
                                              FROM sensor_values, sensors
                                              WHERE name ILIKE 'bi1%' AND sensors.id=sensor_values.sensor_id
                                              AND ts >= $1 AND ts < $2
                                              GROUP BY sensor_id, sensors.name, day ORDER BY 2,3")?;

    let mut converter_element = DataConvertElement::new();

    converter_element.set_property("data_types", "i32,string,string,date,f32");

    // Store

    let mut store_element = PostgresTableSinkElement::new();

    store_element.set_property("host", "carneab4.memset.net");
    store_element.set_property("port", "5432");
    store_element.set_property("user", "glenn");
    store_element.set_property("database", "bmos");
    store_element.set_property("password", "2dd~Pabq")?;

    store_element.set_property("table_create_sql", "CREATE TABLE IF NOT EXISTS sensor_values_days
                                                            (sensor_id integer NOT NULL,
                                                             sensor_name varchar(50) NOT NULL,
                                                             sensor_type varchar(50) NOT NULL,
                                                             day date,
                                                             day_value double precision NOT NULL DEFAULT 'NaN'::double precision,
                                                             last_update timestamp with time zone, 
                                                             CONSTRAINT sensor_id_day_index UNIQUE(sensor_id, day))")?;

    store_element.set_property("insert_sql", "INSERT INTO sensor_values_days(sensor_id, sensor_name, sensor_type, day, day_value, last_update) 
                                                       VALUES ($1,
                                                               $2,
                                                               $3,
                                                               CAST ($4 AS DATE),
                                                               CAST ($5 AS REAL),
                                                               now() at time zone 'utc'
                                                             ) ON CONFLICT(sensor_id,day)
                                                        DO UPDATE SET day_value=$5::real, last_update=now() at time zone 'utc'")?;

    let mut echo_element = EchoSinkElement::new();
    let duplicate_element = DuplicateElement::new();
    let mut null_element = NullSinkElement::new();

    let mut pipeline = Pipeline::new("bicester_day_aggregation".to_string(), PipelineRunType::SCHEDULED, true);

    let db_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(db_src)).unwrap();
    let converter_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(converter_element)).unwrap();
    let duplicate_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(duplicate_element)).unwrap();
    //let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    let store_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(store_element)).unwrap();
    let null_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(null_element)).unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(db_src_ref,
                                                       converter_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(converter_element_ref,
                                                       duplicate_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
                                                       store_element_ref,
                                                       ElementChannelType::CHANNEL1,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.attach_output_pad_to_input_pad_by_channel(duplicate_element_ref,
                                                       null_element_ref,
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
fn bicester_day_aggregation1() {

    if let Err(ref e) = bicester_day_aggregation() {

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