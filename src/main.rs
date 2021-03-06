use std::fs;
use std::fs::File;
use std::io::Read;
use std::{thread, time};
use std::process::Command;
use regex::Regex;

extern crate time as time_date;
extern crate regex;
extern crate serde_json;
extern crate shellexpand;

use serde_json::Value;

static PADDING: &'static str = "%{O10}";
static PADDING_INT: &'static str = "%{O4}";

fn nowplaying(re: &regex::Regex) -> String {
    let icon = "";

    let process = Command::new("playerctl")
                                    .args(&["metadata", "-f", " {{artist}}|SEPARATOR|{{title}}"])
                                    .output()
                                    .unwrap();
    let playing = String::from_utf8(process.stdout)
                                    .expect("Failed to read playerctl");

    let info: Vec<&str> = playing.trim().split("|SEPARATOR|").collect();
    let artist = info[0].replace(" - Topic", "");
    let title = info[1].replace(" | Free Listening on SoundCloud", "");

    let mut formatted = title.to_string();

    if artist.len() > 0 {
        formatted = format!("{} - {}", artist, title);
    }
    format!("{}{p}{}", icon, formatted, p = PADDING_INT)
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

fn telegram_unread(re: &regex::Regex) -> String {
    let icon = "";
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
        if re.captures(out.as_str()).is_some() {
            let caps = re.captures(out.as_str()).unwrap();
            let unread = caps.get(1).map_or("", |m| m.as_str());
            if unread != "" {
                output = format!("{}{p}{}", icon, unread, p = PADDING_INT);
            }
        }
    }
    output
}

fn battery() -> String {
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
        "" // charging battery utf-8
    } else {
        "" // full battery utf-8
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

    format!("{}{p}{}", icon, battery_cap, p = PADDING_INT)
}

fn wifi(re: &regex::Regex) -> String {
    let icon = "";
    let mut ssid = "disconnected";
    let process = Command::new("wpa_cli")
                                    .args(&["-i", "wlan0", "status"])
                                    .output()
                                    .expect("Failed to execute wpa_cli");
    let status = std::string::String::from_utf8(process.stdout)
                                    .expect("Failed to read");
    let capture = re.captures(status.as_str());
    if capture.is_some() {
        let caps = capture.unwrap();
        let matched = caps.get(1).map_or("", |m| m.as_str());

        if matched != "" {
            ssid = matched;
        }
    }
    format!("{}{p}{}", icon, ssid, p = PADDING_INT)
}

fn volume() -> String {
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


    format!("{}{p}{}% ", icon, volume.trim(), p = PADDING_INT)
}

fn clock() -> String {
    let icon = "";
    let time = time_date::strftime("%m-%d %R", &time_date::now()).unwrap();
    format!("{}{p}{}", icon, time, p = PADDING_INT)
}

fn get_wal() -> Value {
    let path: String = shellexpand::tilde("~/.cache/wal/colors.json").to_string();
    let file = File::open(path).unwrap();

    return serde_json::from_reader(file).unwrap()
}

fn colors(color: &str) -> String {
    let wal = get_wal();
    format!("%{{B{}}}", wal["colors"][color].to_string().replace("\"", ""))
}

fn foreground() -> String {
    let wal = get_wal();
    format!("%{{F{}}}", wal["special"]["background"].to_string().replace("\"", ""))
}

fn background() -> String {
    let wal = get_wal();
    format!("%{{B{}}}", wal["colors"]["color6"].to_string().replace("\"", ""))
}

fn main() {
    let yt_re = Regex::new(r"\(\d+\) ").unwrap();
    let unread_re = Regex::new(r"\((\d+)\)").unwrap();
    let wifi_re = Regex::new(r"(?m)^ssid=(.+)").unwrap();

    loop {
        print!("{}{}", background(), foreground());
        print!("{}", colors("color2"));
        print!("{}", PADDING);
        print!("{}", nowplaying(&yt_re));
        print!("{}", PADDING);

        print!("%{{c}}");
        print!("{}", background());
        print!("{}", window_title());
        print!("%{{r}}");

        print!("{}", colors("color2"));
        print!("{}", PADDING);
        print!("{}", telegram_unread(&unread_re));
        print!("{}", PADDING);

        print!("{}", colors("color3"));
        print!("{}", PADDING);
        print!("{}", battery());
        print!("{}", PADDING);

        print!("{}", colors("color4"));
        print!("{}", PADDING);
        print!("{}", wifi(&wifi_re));
        print!("{}", PADDING);

        print!("{}", colors("color5"));
        print!("{}", PADDING);
        print!("{}", volume());
        print!("{}", PADDING);

        print!("{}", colors("color7"));
        print!("{}", PADDING);
        print!("{}", clock());
        print!("{}", PADDING);
        print!("{}", background());
        println!();

        thread::sleep(time::Duration::from_millis(200));
    }
}
