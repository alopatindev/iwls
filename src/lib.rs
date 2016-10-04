#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate nalgebra;
extern crate wifiscanner;

use nalgebra::clamp;
use std::cmp;
use std::process::Command;

const MIN_SIGNAL: f32 = -100.0;
const MAX_SIGNAL: f32 = -50.0;

const UNKNOWN_CHANNEL: u8 = 0;

struct AccessPoint {
    ssid: String,
    mac: String,
    quality: f32,
    channel: u8,
}

pub fn list_access_points() {
    let points = scan_access_points();
    format_access_points(&points);
}

pub fn clear_terminal_and_list_access_points() {
    let points = scan_access_points();
    clear_terminal();
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
             "ESSID",
             "Mac",
             "Quality",
             "Channel");

    for p in &points[..] {
        println!("{0:<20} {1:<20} {2:<8} {3}",
                 p.ssid,
                 p.mac,
                 to_readable_quality(p.quality),
                 to_readable_channel(p.channel));
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
    channel.parse().unwrap_or(UNKNOWN_CHANNEL)
}

fn to_readable_channel(channel: u8) -> String {
    if channel == UNKNOWN_CHANNEL {
        "Unknown".to_string()
    } else {
        format!("{}", channel)
    }
}

fn clear_terminal() {
    // FIXME: use something portable
    if let Ok(output) = Command::new("clear").output() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
}
