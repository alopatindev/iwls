#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate nalgebra;
extern crate wifiscanner;

use nalgebra::clamp;
use std::{cmp, env};
use std::io::Write;
use std::process::Command;

type ChannelLoad = (ChannelId, f64);
type ChannelsLoad = Vec<ChannelLoad>;
type ChannelsLoadSlice<'a> = &'a [ChannelLoad];

pub type ChannelId = u8;

pub const MIN_CHANNEL: ChannelId = 1;
pub const MAX_CHANNEL: ChannelId = 14;
pub const UNKNOWN_CHANNEL: ChannelId = 0;

pub const MIN_SIGNAL: f64 = -100.0;
pub const MAX_SIGNAL: f64 = -50.0;

const MIN_CHANNELS_DISTANCE: ChannelId = 5;
const MAX_SUGGESTIONS: usize = 5;

const LOW_LOAD: f64 = 0.2;

#[derive(Debug, Clone)]
struct Point {
    ssid: String,
    mac: String,
    quality: f64,
    channel: ChannelId,
}

#[derive(Debug, Default)]
struct Channel {
    number_of_points: usize,
    signal_load: f64,
}

impl Channel {
    fn increment(&mut self, quality: f64) {
        self.number_of_points += 1;
        self.signal_load += quality;
    }
}

macro_rules! println_err(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

pub fn list_access_points(clear_term: bool, suggestions: bool) {
    let points = scan_access_points();
    let current_point = get_current_point(&points);

    if clear_term {
        clear_terminal();
    }

    print_access_points(&points, current_point);

    if suggestions {
        println!("");
        print_suggested_channels(&points, current_point);
    }
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
        Err(e) => {
            println_err!("Error: iwlist {:?}", e);
            vec![]
        }
    }
}

fn print_access_points(points: &[Point], current_point: Option<&Point>) {
    println!("{0:<20} {1:<20} {2:<8} {3:<9} {4}",
             "ESSID",
             "Mac",
             "Quality",
             "Channel",
             "Connected");

    for p in points {
        let connected = if is_current_point(p, current_point) {
            "*"
        } else {
            ""
        };
        println!("{0:<20} {1:<20} {2:<8} {3:<9} {4}",
                 p.ssid,
                 p.mac,
                 to_readable_quality(p.quality),
                 to_readable_channel(p.channel),
                 connected);
    }
}

fn is_current_point(point: &Point, current_point: Option<&Point>) -> bool {
    match current_point {
        Some(p) => p as *const Point == point as *const Point,
        None => false,
    }
}

fn compute_channels_load(points: &[Point]) -> ChannelsLoad {
    // FIXME: probably median finding should be used for better results

    let mut channels = Vec::with_capacity(MAX_CHANNEL as usize);
    for i in 1..(MAX_CHANNEL + 1) {
        let value = (i, Channel::default());
        channels.push(value);
    }

    for p in points.iter().filter(|p| p.channel != UNKNOWN_CHANNEL) {
        let index = p.channel as usize - 1;
        channels[index].1.increment(p.quality);

        let (left, right) = intersected_channels(p.channel);

        let mut quality = p.quality;
        for &i in &left {
            quality *= 0.5;
            channels[i as usize - 1].1.increment(quality);
        }

        let mut quality = p.quality;
        for &i in &right {
            quality *= 0.5;
            channels[i as usize - 1].1.increment(quality);
        }
    }

    channels.into_iter()
        .map(|(id, channel)| {
            let average_load = if channel.number_of_points > 0 {
                channel.signal_load / channel.number_of_points as f64
            } else {
                0.0
            };
            (id, average_load)
        })
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

fn intersected_channels(x: ChannelId) -> (Vec<ChannelId>, Vec<ChannelId>) {
    let limit = 2;

    let mut left = Vec::with_capacity(limit);
    let mut y = x;
    let mut i = 0;
    while y > MIN_CHANNEL && i < limit {
        y -= 1;
        i += 1;
        if channels_intersect(y, x) {
            left.push(y);
        }
    }

    let mut right = Vec::with_capacity(limit);
    let mut y = x;
    let mut i = 0;
    while y < MAX_CHANNEL && i < limit {
        y += 1;
        i += 1;
        if channels_intersect(y, x) {
            right.push(y);
        }
    }

    (left, right)
}

fn least_intersected(id: ChannelId) -> bool {
    for &i in &[1, 6, 11, 14] {
        if i == id {
            return true;
        }
    }

    false
}

fn compute_suggestion(other_points: &[Point]) -> Vec<ChannelId> {
    let mut channels_load = compute_channels_load(other_points);

    channels_load.sort_by(|a, b| {
        if a.1 < LOW_LOAD && least_intersected(a.0) {
            cmp::Ordering::Less
        } else {
            a.1.partial_cmp(&b.1).unwrap_or(cmp::Ordering::Less)
        }
    });

    channels_load.iter()
        .take(MAX_SUGGESTIONS)
        .map(|&(id, _)| id)
        .collect()
}

fn print_suggested_channels(points: &[Point], current_point: Option<&Point>) {
    match current_point {
        Some(point) => {
            let points: Vec<Point> = points.into_iter()
                .filter(|x| x.mac != point.mac)
                .cloned()
                .collect();
            let what = format!("\"{}\"", point.ssid);
            print_suggestion(&points, &what);
        }
        None => {
            println!("Current access point is unknown");
        }
    }

    print_suggestion(points, "a new router");

    let channels_load = compute_channels_load(points);
    let channels_load_readable = to_readable_channels_load(&channels_load[..]);
    println!("Channels load: {}", channels_load_readable);
}

fn print_suggestion(points: &[Point], what: &str) {
    let xs = compute_suggestion(points);

    if !xs.is_empty() {
        let other_channels: String = xs.iter()
            .skip(1)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("The best channel for {} is {} (or maybe {})",
                 what,
                 xs[0],
                 other_channels);
    } else {
        println!("Cannot suggest a good channel for {}", what);
    }
}

fn unit_to_percent(x: f64) -> f64 {
    x * 100.0
}

fn to_readable_channels_load(channels_load: ChannelsLoadSlice) -> String {
    let is_zero = |x| x <= std::f64::EPSILON;

    let alot_of_zeros = channels_load.iter()
        .filter(|x| is_zero(x.1))
        .collect::<Vec<_>>()
        .len() > 1;

    let mut result: Vec<String> = channels_load.iter()
        .filter(|x| !alot_of_zeros || !is_zero(x.1))
        .map(|x| format!("{} is {:.1}%", x.0, unit_to_percent(x.1)))
        .collect();

    if alot_of_zeros {
        result.push("others are 0.0%".to_string());
    }

    result.join(", ")
}

fn to_readable_quality(quality: f64) -> String {
    let result = format!("{:.1}%", unit_to_percent(quality));
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

fn get_current_point(points: &[Point]) -> Option<&Point> {
    let mac = get_current_point_mac();
    mac.and_then(|m| points.iter().find(|p| m == p.mac))
}

fn get_current_point_mac() -> Option<String> {
    const PATH_ENV: &'static str = "PATH";
    let path_system = "/usr/sbin:/sbin";
    let path = env::var_os(PATH_ENV).map_or(path_system.to_string(), |v| {
        format!("{}:{}", v.to_string_lossy().into_owned(), path_system)
    });

    const COMMAND: &'static str = "iwconfig";
    let output = Command::new(COMMAND)
        .env(PATH_ENV, path)
        .output();

    match output {
        Ok(output) => {
            let data = String::from_utf8_lossy(&output.stdout);
            return data.lines()
                .map(|x| x.split(" Access Point: ").collect::<Vec<&str>>())
                .filter(|xs| xs.len() == 2 && !xs[1].is_empty())
                .map(|xs| xs[1].trim_right().to_string())
                .next();
        }
        Err(e) => println_err!("Error: {} {:?}", COMMAND, e),
    }

    None
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

        let assert_compute_suggestion = |expect: &[ChannelId], input: &[Point]| {
            assert_eq!(expect, compute_suggestion(input).as_slice())
        };

        {
            let current = make_point(1.0, 2, "current");
            let mut input = vec![];
            assert_compute_suggestion(&[14, 11, 6, 1, 2], &input[..]);
            input.push(current);
            assert_compute_suggestion(&[14, 11, 6, 5, 7], &input[..]);
        }

        {
            let current = make_point(1.0, 2, "current");
            let a = make_point(0.9, 11, "a");
            let mut input = vec![a];
            assert_compute_suggestion(&[14, 6, 1, 2, 3], &input[..]);
            input.push(current);
            assert_compute_suggestion(&[14, 6, 5, 7, 8], &input[..]);
        }

        {
            let current = make_point(1.0, 2, "current");
            let a = make_point(0.9, 11, "a");
            let b = make_point(0.3, 5, "b");
            let mut input = vec![a, b];
            assert_compute_suggestion(&[14, 6, 1, 2, 8], &input[..]);
            input.push(current);
            assert_compute_suggestion(&[14, 8, 7, 6, 4], &input[..]);
        }

        {
            let a = make_point(0.0, UNKNOWN_CHANNEL, "current");
            assert_compute_suggestion(&[14, 11, 6, 1, 2], &[a]);
        }
    }
}
