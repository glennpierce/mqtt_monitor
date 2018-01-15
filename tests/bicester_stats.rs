#![allow(warnings)]

extern crate pretty_env_logger;

extern crate crossbeam;
extern crate liblasso;
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

use liblasso::elements::multiply_element::MultiplyElement;
use liblasso::elements::range_clamp_element::RangeClampElement;
use liblasso::elements::random_source_element::RandomSourceElement;
use liblasso::elements::echo_sink_element::EchoSinkElement;
use liblasso::elements::postgres_source_element::PostgresSourceElement;
use liblasso::elements::postgres_simple_source_element::PostgresSimpleSourceElement;
use liblasso::elements::postgres_table_sink_element::PostgresTableSinkElement;
use liblasso::elements::stats_element::StatsElement;
use liblasso::elements::csv_file_element::CsvFileElement;
use liblasso::elements::duplicate_element::DuplicateElement;
use liblasso::elements::data_convert_element::DataConvertElement;

fn bicester_stats() -> Result<()> {

    let mut db_src = PostgresSimpleSourceElement::new();
    db_src.set_property("host", "carneab4.memset.net");
    db_src.set_property("port", "5432");
    db_src.set_property("user", "glenn");
    db_src.set_property("database", "bmos");
    db_src.set_property("password", "2dd~Pabq")?;

    let start : DateTime<Utc> = Utc::now() - Duration::days(7);

    db_src.set_property("start_time", &start.to_rfc3339())?;
    db_src.set_property("table_name", "sensor_values_days")?;
    db_src.set_property("timestamp_column_name", "ts")?;

    db_src.set_property("select_sql", "SELECT MAX(sensor_id),
                                                     ts,
                                                     split_part(name, '_', 2) as sensor_type,
                                                     MAX(value)::text,
                                                     MIN(value)::text,
                                                     AVG(value)::text,
                                                     SUM(value)::text 
                                                      FROM sensor_values_days, sensors
                                                      WHERE sensors.id=sensor_values_days.sensor_id AND ts >= $1
                                                         GROUP BY ts, sensor_type ORDER BY ts")?;

    let mut converter_element = DataConvertElement::new();

    converter_element.set_property("data_types", "i32,utc,string,f32,f32,f32,f32");

    let mut store_element = PostgresTableSinkElement::new();

    store_element.set_property("host", "carneab4.memset.net");
    store_element.set_property("port", "5432");
    store_element.set_property("user", "glenn");
    store_element.set_property("database", "bmos");
    store_element.set_property("password", "2dd~Pabq")?;

    store_element.set_property("table_create_sql", "CREATE TABLE IF NOT EXISTS bicester_daily_stats
                                                            (sensor_id integer NOT NULL,
                                                             ts timestamp with time zone NOT NULL, 
                                                             sensor_type varchar(50) NOT NULL,
                                                             max double precision NOT NULL DEFAULT 'NaN'::double precision,
                                                             min double precision NOT NULL DEFAULT 'NaN'::double precision,
                                                             avg double precision NOT NULL DEFAULT 'NaN'::double precision,
                                                             sum double precision NOT NULL DEFAULT 'NaN'::double precision,
                                                             CONSTRAINT timestamp_type_index UNIQUE(sensor_type, ts))")?;

    store_element.set_property("insert_sql", "INSERT INTO bicester_daily_stats(sensor_id, ts, sensor_type, max, min, avg, sum) 
                                                       VALUES ($1,
                                                               $2,
                                                               $3,
                                                               CAST ($4 AS REAL),
                                                               CAST ($5 AS REAL),
                                                               CAST ($6 AS REAL),
                                                               CAST ($7 AS REAL)) ON CONFLICT(sensor_type,ts)
                                                        DO UPDATE SET max=$4::real,
                                                                      min=$5::real,
                                                                      avg=$6::real,
                                                                      sum=$7::real RETURNING ts")?;

    let duplicate_element = DuplicateElement::new();
    let mut echo_element = EchoSinkElement::new();

    let mut pipeline = Pipeline::new("bicester stat pipeline".to_string(), PipelineRunType::SCHEDULED, false);

    let db_src_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(db_src)).unwrap();
    let duplicate_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(duplicate_element)).unwrap();
    let converter_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(converter_element)).unwrap();
    let echo_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(echo_element)).unwrap();
    let store_element_ref : PipeLineElementIdentifier = pipeline.add_element(Box::new(store_element)).unwrap();

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
                                                       echo_element_ref,
                                                       ElementChannelType::CHANNEL2,
                                                       ElementChannelType::CHANNEL1)
        .unwrap();

    pipeline.set_schedule_time("* 1 * * * *");

    let mut pipeline_container = PipelineContainer::new();

    let pipeline_name = pipeline.get_name().to_string();

    pipeline_container.add_pipeline(pipeline);
    pipeline_container.serialize().unwrap();

    crossbeam::scope(|scope| {

        let container = pipeline_container.clone();
        let container2 = pipeline_container.clone();
        let mut container3 = pipeline_container.clone();

        scope.spawn(move || {
            container2.clone().run(&pipeline_name);
        });
        
        container3.wait_for_completion();
    });

    Ok(())
}

#[test]
fn bicester_stats1() {

    pretty_env_logger::init().unwrap();

    if let Err(ref e) = bicester_stats() {

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
