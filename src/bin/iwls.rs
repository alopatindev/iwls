#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate iwls;

use clap::App;
use iwls::*;
use std::{thread, time};

const WATCH_INTERVAL_MS: u64 = 1_000;

fn run_watch(suggestions: bool) {
    let interval = time::Duration::from_millis(WATCH_INTERVAL_MS);
    let clear_term = true;

    loop {
        let now = time::Instant::now();

        list_access_points(clear_term, suggestions);
        check_current_user();

        let dt = now.elapsed();
        if dt < interval {
            thread::sleep(interval - dt);
        }
    }
}

fn run_single(suggestions: bool) {
    let clear_term = false;
    list_access_points(clear_term, suggestions);
    check_current_user();
}

fn main() {
    let usage = "-w 'Watch mode'
                 -s 'Suggest channels'";
    let matches = App::new("iwls").args_from_usage(usage).get_matches();
    let suggestions = matches.is_present("s");

    if matches.is_present("w") {
        run_watch(suggestions);
    } else {
        run_single(suggestions);
    }
}
