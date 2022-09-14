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
                                    .args(&["metadata", "-i", "YTMusic", "-f", " {{artist}}|SEPARATOR|{{title}}"])
                                    .output()
                                    .unwrap();
    let playing = String::from_utf8(process.stdout)
                                    .expect("Failed to read playerctl");

    if playing.trim().len() == 0 { // Error No players were found
	"".to_string()
    } else {
    	let info: Vec<&str> = playing.trim().split("|SEPARATOR|").collect();
    	let artist = info[0].replace(" - Topic", "").replace("VEVO", "").replace("vevo", "");
    	let title = info[1].replace(" | Free Listening on SoundCloud", "");

    	let mut formatted = title.to_string();

    	if artist.len() > 0 {
    	    formatted = format!("{} - {}", artist, title);
    	}
    	format!("{}{p}{}", icon, formatted, p = PADDING_INT)
    }
}

fn window_title() -> String {
    let process1 = Command::new("dunstctl")
                                    .args(&["is-paused"])
                                    .output()
                                    .expect("Failed to execute");
    let out1 = std::string::String::from_utf8(process1.stdout)
                                    .expect("Failed to read");

    if out1.trim() == "true" {
        format!("f0x Livestream")
    } else {
        let process = Command::new("xdotool")
                                         .args(&["getwindowfocus", "getwindowname"])
                                         .output()
                                         .expect("Failed to execute");
         let out = std::string::String::from_utf8(process.stdout)
                                         .expect("Failed to read");
         format!("{}", out.trim())
    }
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

fn temps() -> String {
    let process = Command::new("get_temps")
                                .output()
                                .unwrap();
    let temps = String::from_utf8(process.stdout)
                                .expect("Failed to read");
    temps.replace("\n", " ")
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
    let time = time_date::strftime("%Y-%m-%d %H:%M", &time_date::now()).unwrap();
    format!("{}{p}{}", icon, time, p = PADDING_INT)
}

fn get_wal() -> Value {
    let path: String = shellexpand::tilde("~/.cache/wal/colors.json").to_string();
    let file = File::open(path).unwrap();

    return serde_json::from_reader(file).unwrap()
}

fn colors(color: &str) -> String {
    let wal = get_wal();
    format!("%{{U{}}}", wal["colors"][color].to_string().replace("\"", ""))
}

fn foreground() -> String {
    let wal = get_wal();
    format!("%{{F{}}}", wal["special"]["foreground"].to_string().replace("\"", ""))
}

fn background() -> String {
    let wal = get_wal();
    //format!("%{{B{}}}", wal["colors"]["color6"].to_string().replace("\"", ""))
    format!("%{{B{}}}", wal["special"]["background"].to_string().replace("\"", ""))
}

fn main() {
    let yt_re = Regex::new(r"\(\d+\) ").unwrap();
    let unread_re = Regex::new(r"\((\d+)\)").unwrap();

    loop {
        print!("{}{}%{{+o}}", background(), foreground());
        print!("{}", colors("color2"));
        print!("{}", PADDING);
        print!("{}{}", colors("color2"), nowplaying(&yt_re));
        print!("{}", PADDING);

        print!("%{{c}}%{{-u}}%{{-o}}");
        print!("{}", background());
        print!("{}", window_title());
        print!("%{{r}}%{{+o}}");

        // print!("{}", colors("color3"));
        // print!("{}", PADDING);
        // print!("{}", telegram_unread(&unread_re));
        // print!("{}", PADDING);

        // print!("{}", colors("color7"));
        // print!("{}", PADDING);
        // print!("{}", mqtt(mqtt_b));
        // print!("{}", PADDING);

        print!("{}", colors("color4"));
        print!("{}", PADDING);
        print!("{}", temps());
        print!("{}", PADDING);

        print!("{}", colors("color5"));
        print!("{}", PADDING);
        print!("{}", volume());
        print!("{}", PADDING);

        print!("{}", colors("color6"));
        print!("{}", PADDING);
        print!("{}", clock());
        print!("{}", PADDING);
        print!("{}", background());
        println!();

        thread::sleep(time::Duration::from_millis(200));
    }
}
