use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::process::Command;
use regex::Regex;

extern crate time as time_date;
extern crate regex;

fn battery(fg: &str, ac: &str) -> String {
    let is_charging = fs::read_dir("/sys/class/power_supply")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|file| file.path().to_str().unwrap().contains("AC"))
        .any(|ac_supply| {
            let mut ac = ac_supply.path();
            ac.push("online");
            let mut f = File::open(ac).expect("file not found");

            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .expect("something went wrong reading the ac status");
            contents.contains("1")
        });
    let icon = if is_charging {
        ""
    } else {
        ""
    };

    let batteries = fs::read_dir("/sys/class/power_supply")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|file| file.path().to_str().unwrap().contains("BAT"))
        .map(|battery| {
            let mut bat = battery.path();
            bat.push("capacity");
            let mut f = File::open(bat).expect("file not found");

            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .expect("something went wrong reading the battery capacity");
            let capacity = contents.trim().to_string();
            format!("{}% ", capacity)
        })
        .fold("".to_string(), |mut values, cap| {values.push_str(&cap); values});
    let battery_cap = format!("{}", batteries);

    format!("{}{}{}{}", ac, icon, fg, battery_cap)
}

fn nowplaying(fg: &str, ac: &str) -> String {
    let replacements = ["YouTube", " (Official Video)", " (HQ)", " (Official Video)", " [FULL MUSIC VIDEO]", " (320kbps)", " (with lyrics)", " - Full album", " [Official Music Video]", "(Official Music Video)", "(Lyric Video)", "(lyric video)", "[HQ]", "High Quality Sound", "HD 720p", "[Lyrics]", "(MUSIC VIDEO)"];
    let re = Regex::new(r"\(\d+\) ").unwrap();
    let process = Command::new("xdotool")
                                    .args(&["search", "--classname", "youtube.com"])
                                    .output()
                                    .unwrap();
    let window_ids = String::from_utf8(process.stdout)
                                    .expect("Failed to read");
    if window_ids.split_whitespace().next().is_some() {
        let window_id = window_ids.split_whitespace().next().unwrap();

        let process = Command::new("xdotool")
                                        .args(&["getwindowname", window_id])
                                        .output()
                                        .expect("Failed to execute");
        let out = std::string::String::from_utf8(process.stdout)
                                        .expect("Failed to read");
        let mut playing = out.trim().replace(" - YouTube", "");
        playing = re.replace_all(&playing, "").into_owned();
        for replacement in replacements.iter() {
            playing = playing.replace(replacement, "");
        }
        format!("{}{}{}", ac, fg, playing)
    } else {
        format!("")
    }
}

fn telegram_unread(fg: &str, ac: &str) -> String {
    let re = Regex::new(r"\((\d+)\)").unwrap();
    let mut output = String::new();
    let process = Command::new("xdotool")
                                    .args(&["search", "--name", r"Telegram \("])
                                    .output()
                                    .unwrap();
    let telegram_ids = String::from_utf8(process.stdout)
                                    .expect("Failed to read");
    if telegram_ids.split_whitespace().next().is_some() {
        let telegram_id = telegram_ids.split_whitespace().next().unwrap();

        let process = Command::new("xdotool")
                                        .args(&["getwindowname", telegram_id])
                                        .output()
                                        .expect("Failed to execute");
        let out = std::string::String::from_utf8(process.stdout)
                                        .expect("Failed to read");
        let caps = re.captures(out.as_str()).unwrap();
        let unread = caps.get(1).map_or("", |m| m.as_str());
        if unread != "" {
            output = format!("{}{}{} ", ac, fg, unread);
        }
    }
    output
}

fn wifi_bssid(fg: &str, ac: &str) -> String {
    let process = Command::new("get_wifi_bssid")
                                    .output()
                                    .expect("Failed to execute");
    let bssid = std::string::String::from_utf8(process.stdout)
                                    .expect("Failed to read");
    format!("{}{}{} ", ac, fg, bssid.trim())
}

fn volume(fg: &str, ac: &str) -> String {
    let process = Command::new("pamixer")
                                    .arg("--get-volume")
                                    .output()
                                    .expect("Failed to execute");
    let volume = std::string::String::from_utf8(process.stdout)
                                    .expect("Failed to read");

    let mute_process = Command::new("pamixer")
                                    .arg("--get-mute")
                                    .output()
                                    .expect("Failed to execute");
    let muted = std::string::String::from_utf8(mute_process.stdout)
                                    .expect("Failed to read");
    let icon = if muted.trim() == "true" {
        ""
    } else {
        ""
    };


    format!("{}{}{}{}% ", ac, icon, fg, volume.trim())
}

fn window_title() -> String {
    let process = Command::new("xdotool")
                                    .args(&["getwindowfocus", "getwindowname"])
                                    .output()
                                    .expect("Failed to execute");
    let out = std::string::String::from_utf8(process.stdout)
                                    .expect("Failed to read");
    format!("{}", out.trim())
}

fn clock(fg: &str, ac: &str) -> String {
    let time = time_date::strftime("%Y-%m-%d %R", &time_date::now()).unwrap();
    format!("{}{}{}", ac, fg, time)
}

fn main() {
    let fg = "%{F#d9e1e8}";
    //let ac = "%{F#cc6666}";
    let ac = "%{F#bd0a41}";
    //let fg = "%{F#d9e1e8}";
    //let ac = "%{F#600b0b}";
    //let ac = "%{F#9baec8}";
    //let ac = "%{F#81a1c1}";
    loop {
        println!(" {}%{{c}}{}%{{r}}{}{}{}{}{} ", nowplaying(fg, ac), window_title(), telegram_unread(fg, ac), battery(fg, ac), wifi_bssid(fg, ac), volume(fg, ac), clock(fg, ac));
        thread::sleep(time::Duration::from_millis(200));
    }
}
