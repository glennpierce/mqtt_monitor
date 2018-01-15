extern crate schedule;
extern crate chrono;
extern crate crossbeam;

use std::{thread, time};
use chrono::*;

fn hello_world_5() {
    crossbeam::scope(|scope| {

        scope.spawn(move || {
            println!("{}: Hello World (every 5 seconds)!", Utc::now());
            thread::sleep(time::Duration::from_millis(10000));
            println!("Done hello_world_5");
        });
    });
}

#[test]
fn schedule_hello() {
    // Create new, empty agenda
    let mut a = schedule::Agenda::new();

    // Add a job from a closure, scheduled to run every 2 seconds
    a.add(|| {

                crossbeam::scope(|scope| {

                    scope.spawn(move || {
                        println!("{}: Hello World (every 2 seconds)!", Utc::now());
                        thread::sleep(time::Duration::from_millis(5000));
                        println!("Done hello_world_2");
                    });
                });


             })
        .schedule("0 * * * * *")
        .unwrap();

    // Add a job from a function and give it a name
    a.add(hello_world_5).with_name("hello-world");

    a.add(|| {
                 println!("{}: Hello World (every 3 seconds)!", Utc::now());
             })
        .schedule("0 * * * * *")
        .unwrap();

    // Schedule that job to run every 5 seconds
    a.get("hello-world")
        .unwrap()
        .schedule("0 * * * * *")
        .unwrap();

    loop {
        // Execute pending jobs
        a.run_pending();

        // Sleep for 500ms
        thread::sleep(time::Duration::from_millis(500));
    }
}