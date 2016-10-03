#[macro_use]
extern crate clap;
extern crate nalgebra;
extern crate wifiscanner;

use clap::App;
use nalgebra::clamp;
use std::{cmp, thread, time};
use std::process::Command;

const WATCH_INTERVAL_MS: u64 = 1_000;

const MIN_SIGNAL: f32 = -100.0;
const MAX_SIGNAL: f32 = -50.0;

struct AccessPoint {
    ssid: String,
    mac: String,
    quality: f32,
    channel: u8,
}

fn list_access_points(clear: bool) {
    let points = scan_access_points();

    if clear {
        clear_terminal();
    }

    format_access_points(&points);
}

fn scan_access_points() -> Vec<AccessPoint> {
    match wifiscanner::scan() {
        Ok(points) => {
            let mut result: Vec<AccessPoint> = points.into_iter()
                .map(|p| {
                    AccessPoint {
                        ssid: p.ssid,
                        mac: p.mac,
                        quality: signal_to_quality(&p.signal_level),
                        channel: parse_channel(&p.channel),
                    }
                })
                .collect();

            result.sort_by(|a, b| {
                b.quality
                    .partial_cmp(&a.quality)
                    .unwrap_or(cmp::Ordering::Less)
            });

            result
        }
        Err(_) => vec![],
    }
}

fn format_access_points(points: &Vec<AccessPoint>) {
    println!("{0:<20} {1:<20} {2:<8} {3}",
             "SSID",
             "Mac",
             "Quality",
             "Channel");

    for p in &points[..] {
        println!("{0:<20} {1:<20} {2:<8} {3}",
                 p.ssid,
                 p.mac,
                 to_readable_quality(p.quality),
                 p.channel);
    }
}

fn to_readable_quality(quality: f32) -> String {
    format!("{}%", quality * 100.0)
}

fn signal_to_quality(signal_level: &String) -> f32 {
    let signal = signal_level.parse().ok().map_or(MIN_SIGNAL, |x| x);
    let offset = -MIN_SIGNAL;
    let quality = (signal + offset) / (MAX_SIGNAL + offset);
    clamp(quality, 0.0, 100.0)
}

fn parse_channel(channel: &String) -> u8 {
    channel.parse().unwrap()
}

fn clear_terminal() {
    // FIXME: use something portable
    if let Ok(output) = Command::new("clear").output() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

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
