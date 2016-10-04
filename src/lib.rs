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

const MIN_CHANNELS_DISTANCE: ChannelId = 5;

#[derive(Debug)]
struct Point {
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

struct Suggestion {
    current_point: Vec<ChannelId>,
    new_point: Vec<ChannelId>,
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

fn scan_access_points() -> Vec<Point> {
    match wifiscanner::scan() {
        Ok(points) => {
            let mut result: Vec<Point> = points.into_iter()
                .map(|p| {
                    Point {
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

fn print_access_points(points: &[Point]) {
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

fn compute_channels_load(points: &[Point]) -> ChannelsLoad {
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

fn channels_intersect(a: ChannelId, b: ChannelId) -> bool {
    // see https://en.wikipedia.org/wiki/List_of_WLAN_channels
    let shift_max = |x| if x == MAX_CHANNEL { x + 2 } else { x };
    let a = shift_max(a);
    let b = shift_max(b);
    let distance = cmp::max(a, b) - cmp::min(a, b);
    distance < MIN_CHANNELS_DISTANCE
}

fn compute_suggestion(current: Option<&Point>, other_points: &[Point]) -> Suggestion {
    // let channels_load = compute_channels_load(current, other_points);
    unimplemented!()
}

fn print_suggested_channels(points: &[Point]) {
    // TODO
}

fn to_readable_quality(quality: f64) -> String {
    let result = format!("{:.1}%", quality * 100.0);
    println!("result = {}", result);
    result
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

    #[test]
    fn test_to_readable_quality() {
        use super::to_readable_quality;

        assert_eq!("12.3%", to_readable_quality(0.1234));
    }

    #[allow(float_cmp)]
    #[test]
    fn test_signal_to_quality() {
        use super::signal_to_quality;

        assert_eq!(0.0, signal_to_quality("-100"));
        assert_eq!(1.0, signal_to_quality("-50"));
        assert_eq!(1.0, signal_to_quality("0"));
        assert_eq!(0.0, signal_to_quality("-200"));
    }

    #[test]
    fn test_channels_intersect() {
        use super::channels_intersect;

        assert!(channels_intersect(1, 1));
        assert!(channels_intersect(1, 2));
        assert!(channels_intersect(1, 3));
        assert!(channels_intersect(1, 4));
        assert!(channels_intersect(1, 5));
        assert!(!channels_intersect(1, 6));
        assert!(!channels_intersect(1, 7));
        assert!(!channels_intersect(1, 13));
        assert!(!channels_intersect(1, 14));
        assert!(channels_intersect(2, 1));
        assert!(channels_intersect(2, 2));
        assert!(channels_intersect(2, 3));
        assert!(channels_intersect(2, 4));
        assert!(channels_intersect(2, 6));
        assert!(!channels_intersect(2, 7));
        assert!(channels_intersect(14, 13));
        assert!(channels_intersect(14, 12));
        assert!(!channels_intersect(14, 11));
    }

    #[test]
    fn test_compute_suggestion() {
        use super::{compute_suggestion, Point};

        let make_point = |quality, id, ssid: &str| {
            Point {
                ssid: ssid.to_string(),
                mac: "".to_string(),
                quality: quality,
                channel: id,
            }
        };

        {
            let y = compute_suggestion(None, &[]);
            assert!(y.current_point.is_empty());
            assert_eq!(&[1, 6, 11, 14, 2], &y.new_point[..]);
        }

        {
            let current = make_point(1.0, 2, "current");
            let y = compute_suggestion(Some(&current), &[]);
            assert_eq!(&[1, 6, 11, 14, 2], &y.current_point[..]);
            assert_eq!(&[1, 6, 11, 14, 2], &y.new_point[..]);
        }

        {
            let current = make_point(1.0, 2, "current");
            let xs = &[make_point(1.0, 11, "a")];
            let y = compute_suggestion(Some(&current), xs);
            assert_eq!(&[1, 14, 6, 2, 3], &y.current_point[..]);
            assert_eq!(&[14, 6, 2, 3, 4], &y.new_point[..]);
        }

        {
            let current = make_point(1.0, 2, "current");
            let xs = &[make_point(1.0, 11, "a"), make_point(0.3, 5, "b")];
            let y = compute_suggestion(Some(&current), xs);
            assert_eq!(&[1, 14, 6, 2, 3], &y.current_point[..]);
            assert_eq!(&[14, 6, 5, 7], &y.new_point[..]);
        }
    }
}
