#![feature(proc_macro_hygiene)]

use std::cmp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time, time::SystemTime};

use chrono::prelude::*;
use huelib::resource::{light, Light, Modifier, ModifierType};
use huelib::{bridge::discover as discover_bridge, Bridge};
use sunrise;

mod conf;
use render::{html, raw};

// TODO: allow to pass this one via command line argument
const TEST_LIGHT_ID: &str = "5";
// TODO: move this to configuration, e.g. env vars
const WAKE_UP_TIME: (u32, u32) = (7, 30);
const BED_TIME: (u32, u32) = (22, 00);

fn set_ctrlc_handler(running: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        println!("Ctrl-C received, terminating gracefully");
        running.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// Calculate sunrise/sunset times for today
fn sunrise_sunset(lat: f64, lng: f64) -> (i64, i64) {
    let today = Local::today();

    sunrise::sunrise_sunset(lat, lng, today.year(), today.month(), today.day())
}

fn get_configured_timestamp(configured_time: (u32, u32)) -> i64 {
    let (hour, minute) = configured_time;

    Local::now()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
        .with_hour(hour)
        .unwrap()
        .with_minute(minute)
        .unwrap()
        .timestamp()
}

fn get_color_temperature(sod: i64, eod: i64, now: i64) -> f32 {
    let sunrise_duration = 60 * 60; // Transition time set to 1 hour
    let sunset_duration = 60 * 60 * 2; // Transition time set to 2 hour

    if now < sod || now > eod {
        // Before start or after end - warmest
        0.
    } else if now > sod + sunrise_duration && now < eod - sunset_duration {
        // 1 hour after start or 1 hour before end - coldest
        1.
    } else if now < sod + sunrise_duration {
        // Sunrise, growing from warmest (0) to coldest (1) in 1 hour
        (now - sod) as f32 / sunrise_duration as f32
    } else {
        // Sunset, decreasing from coldest (1) to warmest (0)
        (eod - now) as f32 / sunset_duration as f32
    }
}

fn easing(x: f32) -> f32 {
    1. - (1. - x).powf(3.)
}

fn get_brightness(sod: i64, eod: i64, now: i64) -> f32 {
    let baseline = 0.4;
    let fade_time = 60 * 60 * 3; // 3 hours of fade in/out time
    let halfday = (eod - sod) / 2;
    let midday = sod + halfday;

    if now < sod - fade_time || now > eod + fade_time {
        // Night
        0.
    } else if now < sod {
        // Fade in
        ((sod - now) as f32 / fade_time as f32) * baseline
    } else if now > eod {
        // Fade out
        (1. - (now - eod) as f32 / fade_time as f32) * baseline
    } else if now <= midday {
        // Morning
        easing((now - sod) as f32 / halfday as f32) * (1. - baseline) + baseline
    } else {
        // Afternoon
        easing((eod - now) as f32 / halfday as f32) * (1. - baseline) + baseline
    }
}

fn get_brightness_modifier(bri: f32) -> light::StateModifier {
    light::StateModifier::new().brightness(ModifierType::Override, (bri * 254.) as u8)
}

fn main2() {
    let running = Arc::new(AtomicBool::new(true));
    let is_running = || running.load(Ordering::SeqCst);
    let username = conf::get_username();
    let (sunrise, sunset) = sunrise_sunset(conf::get_lat(), conf::get_lng());
    // Wake up and bed time are offset by 30 minutes to allow for some slack
    let wake_up_time = get_configured_timestamp(WAKE_UP_TIME) - (60 * 30);
    let bed_time = get_configured_timestamp(BED_TIME) + (60 * 30);
    let start_of_day = cmp::min(sunrise, wake_up_time);
    let end_of_day = cmp::max(sunset, bed_time);

    set_ctrlc_handler(running.clone());

    let ip_address = discover_bridge()
        .expect("Failed to discover bridges")
        .pop()
        .expect("No bridges found in the local network");

    let bridge = Bridge::new(ip_address, username);
    println!("{:?}", bridge);

    let sod_time = Local.timestamp(start_of_day, 0);
    let eod_time = Local.timestamp(end_of_day, 0);
    println!("sod: {}, eod: {}", sod_time, eod_time);

    while is_running() {
        let light: Light = bridge
            .get_light(TEST_LIGHT_ID)
            .expect("Failed to get test light");

        let timestamp = get_timestamp();
        let local_now = Local::now();
        println!("{}", local_now);

        let bri = get_brightness(start_of_day, end_of_day, timestamp);
        println!("brightness: {:.5}", bri);

        let modifier = match light.capabilities.control.color_temperature {
            Some(bounds) => {
                let ct = get_color_temperature(start_of_day, end_of_day, timestamp);
                println!("color temperature: {:.5}", ct);

                get_brightness_modifier(bri).color_temperature(
                    ModifierType::Override,
                    ((1. - ct) * (bounds.max - bounds.min) as f32) as u16 + bounds.min as u16,
                )
            }
            None => get_brightness_modifier(bri),
        };

        match bridge.set_light_state(TEST_LIGHT_ID, &modifier) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to modify the light state: {}", e),
        };

        thread::sleep(time::Duration::from_millis(1500))
    }
}

fn main() {
    let (sunrise, sunset) = sunrise_sunset(conf::get_lat(), conf::get_lng());
    // Wake up and bed time are offset by 30 minutes to allow for some slack
    let wake_up_time = get_configured_timestamp(WAKE_UP_TIME) - (60 * 30);
    let bed_time = get_configured_timestamp(BED_TIME) + (60 * 30);
    let start_of_day = cmp::min(sunrise, wake_up_time);
    let end_of_day = cmp::max(sunset, bed_time);

    let tree = html! {
        <div>
            <p>{"<Hello />"}</p>
            <p>{raw!("<Hello />")}</p>
        </div>
    };

    println!("{}", tree);
    println!(
        "{:?}",
        (0..24)
            .flat_map(|h| (0..4).map(move |i| (h, i * 15)))
            .map(|(h, m)| {
                let timestamp = Local::now()
                    .with_hour(h)
                    .and_then(move |t| t.with_minute(m))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap()
                    .timestamp();
                let bri = get_brightness(start_of_day, end_of_day, timestamp);
                let ct = get_color_temperature(start_of_day, end_of_day, timestamp);

                (h, m, bri, ct)
            })
            .collect::<Vec<_>>()
    );
}
