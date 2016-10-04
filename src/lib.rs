#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate nalgebra;
extern crate wifiscanner;

use nalgebra::clamp;
use std::cmp;
use std::collections::HashMap;
use std::process::Command;

type ChannelId = u8;

const MAX_CHANNEL: ChannelId = 14;
const UNKNOWN_CHANNEL: ChannelId = 0;

const MIN_SIGNAL: f64 = -100.0;
const MAX_SIGNAL: f64 = -50.0;

struct AccessPoint {
    ssid: String,
    mac: String,
    quality: f64,
    channel: ChannelId,
}

#[derive(Debug)]
struct Channel {
    number_of_points: usize,
    signal_load: f64,
}

pub fn list_access_points() {
    let points = scan_access_points();
    print_access_points(&points);
    print_suggested_channels(&points);
}

pub fn clear_terminal_and_list_access_points() {
    let points = scan_access_points();
    clear_terminal();
    print_access_points(&points);
    print_suggested_channels(&points);
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

fn print_access_points(points: &Vec<AccessPoint>) {
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

fn compute_channels_load(points: &Vec<AccessPoint>) -> HashMap<ChannelId, f64> {
    // FIXME: probably median finding should be used for better results
    let mut channels = HashMap::with_capacity(MAX_CHANNEL as usize);

    for p in &points[..] {
        let mut channel = channels.entry(p.channel)
            .or_insert_with(|| {
                Channel {
                    number_of_points: 0,
                    signal_load: 0.0,
                }
            });
        channel.number_of_points += 1;
        channel.signal_load += p.quality;
    }

    let channels_load: HashMap<ChannelId, f64> = channels.iter()
        .map(|(ch, channel)| (*ch, channel.signal_load / channel.number_of_points as f64))
        .collect();

    channels_load
}

fn print_suggested_channels(points: &Vec<AccessPoint>) {
    let channels_load = compute_channels_load(points);
    println!("{:?}", channels_load);
    // TODO
}

fn to_readable_quality(quality: f64) -> String {
    format!("{}%", quality * 100.0)
}

fn signal_to_quality(signal_level: &String) -> f64 {
    let signal = signal_level.parse().ok().map_or(MIN_SIGNAL, |x| x);
    let signal = clamp(signal, MIN_SIGNAL, MAX_SIGNAL);
    let offset = -MIN_SIGNAL;
    let quality = (signal + offset) / (MAX_SIGNAL + offset);
    clamp(quality, 0.0, 100.0)
}

fn parse_channel(channel: &String) -> ChannelId {
    channel.parse().unwrap_or(UNKNOWN_CHANNEL)
}

fn to_readable_channel(channel: ChannelId) -> String {
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
