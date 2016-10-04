#[macro_use]
extern crate clap;
extern crate iwls;

use clap::App;
use iwls::*;
use std::{thread, time};

const WATCH_INTERVAL_MS: u64 = 1_000;

fn watch() {
    let interval = time::Duration::from_millis(WATCH_INTERVAL_MS);

    loop {
        let now = time::Instant::now();
        list_access_points(true);
        let dt = now.elapsed();
        if dt < interval {
            thread::sleep(interval - dt);
        }
    }
}

fn main() {
    let usage = "-w 'Watch mode'";
    let matches = App::new("iwls").args_from_usage(usage).get_matches();
    if matches.is_present("w") {
        watch();
    } else {
        list_access_points(false);
    }
}
