#![allow(warnings)]

extern crate time;
extern crate chrono;

use chrono::{Utc, Local, FixedOffset, Weekday, DateTime, NaiveDateTime, NaiveTime, NaiveDate, Timelike, TimeZone, Duration};
use chrono::Datelike;

#[derive(Clone, Debug, PartialEq)]
pub enum TimeGranularity {
    ALL,
    SECOND,
    MINUTE,
    FIVEMINUTE,
    TWENTYMINUTE,
    THIRTYMINUTE,
    HOUR,
    DAYQUARTER,
    DAY,
    WEEK,
    MONTH,
    YEAR
}

impl TimeGranularity {

    fn to_duration(&self) -> Result<Duration, &str> {
        match *self {
            TimeGranularity::ALL => Ok(Duration::nanoseconds(0)),
            TimeGranularity::SECOND => Ok(Duration::seconds(0)),
            TimeGranularity::MINUTE => Ok(Duration::minutes(1)),
            TimeGranularity::FIVEMINUTE => Ok(Duration::minutes(5)),
            TimeGranularity::TWENTYMINUTE => Ok(Duration::minutes(20)),
            TimeGranularity::THIRTYMINUTE => Ok(Duration::minutes(30)),
            TimeGranularity::HOUR => Ok(Duration::hours(1)),
            TimeGranularity::DAYQUARTER => Ok(Duration::hours(6)),
            TimeGranularity::DAY => Ok(Duration::days(1)),
            TimeGranularity::WEEK => Ok(Duration::weeks(1)),
            _ => Err("No conversion possible"),
        }
    }
}

fn floor_by_granularity(dt : DateTime<Utc>, granularity : &TimeGranularity) -> DateTime<Utc> {

    match *granularity {
            TimeGranularity::ALL => dt,
            TimeGranularity::SECOND => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms_nano(dt.hour(), dt.minute(), dt.second(), 0),
            TimeGranularity::MINUTE => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), dt.minute(), 0),
            TimeGranularity::FIVEMINUTE => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), (dt.minute() / 5) * 5, 0),
            TimeGranularity::TWENTYMINUTE => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), (dt.minute() / 20) * 20, 0),
            TimeGranularity::THIRTYMINUTE => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), (dt.minute() / 30) * 30, 0),
            TimeGranularity::HOUR => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), 0, 0),
            TimeGranularity::DAYQUARTER => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms((dt.hour() / 6) * 6, 0, 0),
            TimeGranularity::DAY => Utc.ymd(dt.year(), dt.month(), dt.day()).and_hms(0, 0, 0),
            TimeGranularity::WEEK => {

                let week = dt.iso_week().week();
                let ndt = NaiveDate::from_isoywd(dt.year(), week, Weekday::Mon).and_hms(0, 0, 0);
                return DateTime::<Utc>::from_utc(ndt, Utc);
            },  
            TimeGranularity::MONTH => Utc.ymd(dt.year(), dt.month(), 1).and_hms(0, 0, 0),
            TimeGranularity::YEAR => Utc.ymd(dt.year(), 1, 1).and_hms(0, 0, 0),
            _ => dt,
    }
}

fn ceil_by_granularity(dt : DateTime<Utc>, granularity : &TimeGranularity) -> DateTime<Utc> {
    match *granularity {
            TimeGranularity::ALL => dt,
            TimeGranularity::MONTH => {
                // First change day to 25th of month.
                // We are then going to add 10 days and floor
                // Can't just add to month incase we are december
                let towards_end_of_month = Utc.ymd(dt.year(), dt.month(), 25).and_hms(0, 0, 0);
                let next_month_dt = towards_end_of_month + Duration::days(10);
                let next_month_start = floor_by_granularity(next_month_dt, &TimeGranularity::MONTH);
                return next_month_start - Duration::nanoseconds(1);
            },
            TimeGranularity::YEAR => {
                return Utc.ymd(dt.year(), 12, 31).and_hms_milli(23, 59, 59, 999);
            },
            _ => {

                // Possible lead second bug here. But can't imagine it matters for our cases
                let floored = floor_by_granularity(dt, granularity);
                let duration = granularity.to_duration().unwrap();
                let ceil = floored + duration - Duration::nanoseconds(1); // subtract a nano second to push into the last time period
                return ceil;
            }
    }
}

#[test]
fn date_parse() {

    let now = Utc::now();
    let floored = floor_by_granularity(now, &TimeGranularity::YEAR);
    let ceil = ceil_by_granularity(now, &TimeGranularity::YEAR);

    println!("Now: {:?}", now);
    println!("Floored: {:?}", floored);
    println!("Ceil: {:?}", ceil);

    //DateTime::parse_from_str("1/1/06 0:00", "%0m/%e/%y %k:%M").unwrap();
    //DateTime::parse_from_str("1/1/06 0:00", "%D %k:%M").unwrap();

    NaiveTime::parse_from_str("0:00:00", "%H:%M:%S").unwrap();
    NaiveTime::parse_from_str("0:00:00", "%T").unwrap();
    NaiveDate::parse_from_str("1/1/06", "%D").unwrap();
    let dt = NaiveDateTime::parse_from_str("1/1/06 0:00", "%D %H:%M").unwrap();
    
    //let Utc_dt = DateTime::<Utc>::from_Utc(dt, Utc);

    //assert!(!DateTime::parse_from_str("00:00:00", "%H:%M:%S").is_err());

    // assert_eq!(DateTime::parse_from_str("2014-5-7T12:34:56+09:30", "%Y-%m-%dT%H:%M:%S%z"),
    //                Ok(ymdhms(2014, 5, 7, 12, 34, 56, 570*60))); // ignore offset
    // assert!(DateTime::parse_from_str("20140507000000", "%Y%m%d%H%M%S").is_err()); // no offset
    // assert!(DateTime::parse_from_str("Fri, 09 Aug 2013 23:54:35 GMT",
    //                                      "%a, %d %b %Y %H:%M:%S GMT").is_err());
    // assert_eq!(Utc.datetime_from_str("Fri, 09 Aug 2013 23:54:35 GMT",
    //                                      "%a, %d %b %Y %H:%M:%S GMT"),
    //                Ok(Utc.ymd(2013, 8, 9).and_hms(23, 54, 35)));
}