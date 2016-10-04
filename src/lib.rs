#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate nalgebra;
extern crate wifiscanner;

use nalgebra::clamp;
use std::cmp;
use std::collections::HashMap;
use std::process::Command;

pub type ChannelId = u8;

pub const MAX_CHANNEL: ChannelId = 14;
pub const UNKNOWN_CHANNEL: ChannelId = 0;

pub const MIN_SIGNAL: f64 = -100.0;
pub const MAX_SIGNAL: f64 = -50.0;

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

type ChannelsLoad = HashMap<ChannelId, f64>;

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

fn print_access_points(points: &[AccessPoint]) {
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

fn compute_channels_load(points: &[AccessPoint]) -> ChannelsLoad {
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

    channels.iter()
        .map(|(id, channel)| (*id, channel.signal_load / channel.number_of_points as f64))
        .collect::<ChannelsLoad>()
}

fn print_suggested_channels(points: &[AccessPoint]) {
    let channels_load = compute_channels_load(points);
    println!("{:?}", channels_load);
    // TODO
}

fn to_readable_quality(quality: f64) -> String {
    format!("{}%", quality * 100.0)
}

fn signal_to_quality(signal_level: &str) -> f64 {
    let signal = signal_level.parse().ok().map_or(MIN_SIGNAL, |x| x);
    let signal = clamp(signal, MIN_SIGNAL, MAX_SIGNAL);
    let offset = -MIN_SIGNAL;
    let quality = (signal + offset) / (MAX_SIGNAL + offset);
    clamp(quality, 0.0, 100.0)
}

fn parse_channel(id: &str) -> ChannelId {
    id.parse().unwrap_or(UNKNOWN_CHANNEL)
}

fn to_readable_channel(id: ChannelId) -> String {
    if id == UNKNOWN_CHANNEL || id > MAX_CHANNEL {
        "Unknown".to_string()
    } else {
        id.to_string()
    }
}

fn clear_terminal() {
    // FIXME: use something portable
    if let Ok(output) = Command::new("clear").output() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_readable_channel() {
        use super::to_readable_channel;

        for i in 1..(MAX_CHANNEL + 1) {
            assert_eq!(i.to_string(), to_readable_channel(i));
        }

        assert_eq!("Unknown", to_readable_channel(UNKNOWN_CHANNEL));
        assert_eq!("Unknown", to_readable_channel(MAX_CHANNEL + 1));
    }

    #[test]
    fn test_parse_channel() {
        use super::parse_channel;

        assert_eq!(1, parse_channel("1"));
        assert_eq!(UNKNOWN_CHANNEL, parse_channel("0"));
        assert_eq!(UNKNOWN_CHANNEL, parse_channel("foo"));
    }
}
