use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::process::Command;

extern crate time as time_date;

fn battery(fg: &str, ac: &str) -> String {
    let mut icon = String::from("");
    let mut charge = String::new();

    let power = fs::read_dir("/sys/class/power_supply").unwrap();
    let ac_supplies = power.filter(|x| x.as_ref().unwrap().path().to_str().unwrap().contains("AC")); 
    for ac_supply in ac_supplies {
        let ac = ac_supply.unwrap().path();
        let ac = ac.to_str().unwrap();

        let filename = format!("{}/online", ac);
        let mut f = File::open(filename).expect("file not found");
        
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the battery capacity");

        if contents.contains("1") {
            icon = String::from("");
        }
    }

    let power1 = fs::read_dir("/sys/class/power_supply").unwrap();
    let batteries = power1.filter(|x| x.as_ref().unwrap().path().to_str().unwrap().contains("BAT")); 
    for battery in batteries {
        let bat = battery.unwrap().path();
        let bat = bat.to_str().unwrap();

        let filename = format!("{}/capacity", bat);
        let mut f = File::open(filename).expect("file not found");
        
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the battery capacity");
        charge = format!("{}{}% ", charge, contents.trim());
    }

    format!("{}{}{}{}", ac, icon, fg, charge)
}

fn nowplaying(fg: &str, ac: &str) -> String {
    let process = Command::new("xdotool")
                                    .args(&["search", "--name", "YouTube"])
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let window_ids = std::string::String::from_utf8(process.stdout)
                                    .ok()
                                    .expect("Failed to read");
    let window_id = window_ids.split_whitespace().next().unwrap();

    let process = Command::new("xdotool")
                                    .args(&["getwindowname", window_id])
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let out = std::string::String::from_utf8(process.stdout)
                                    .ok()
                                    .expect("Failed to read");
    let playing = out.trim().replace(" - YouTube - Waterfox", "");
    format!("{}{}{}", ac, fg, playing)
}

fn wifi_bssid(fg: &str, ac: &str) -> String {
    let process = Command::new("get_wifi_bssid")
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let bssid = std::string::String::from_utf8(process.stdout)
                                    .ok()
                                    .expect("Failed to read");
    format!("{}{}{} ", ac, fg, bssid.trim())
}

fn volume(fg: &str, ac: &str) -> String {
    let mut icon = "";
    let process = Command::new("pamixer")
                                    .arg("--get-volume")
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let volume = std::string::String::from_utf8(process.stdout)
                                    .ok()
                                    .expect("Failed to read");

    let mute_process = Command::new("pamixer")
                                    .arg("--get-mute")
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let muted = std::string::String::from_utf8(mute_process.stdout)
                                    .ok()
                                    .expect("Failed to read");
    if muted.trim() == "true" {
        icon = "";
    }


    format!("{}{}{}{}% ", ac, icon, fg, volume.trim())
}

fn window_title() -> String {
    let process = Command::new("xdotool")
                                    .args(&["getwindowfocus", "getwindowname"])
                                    .output()
                                    .ok()
                                    .expect("Failed to execute");
    let out = std::string::String::from_utf8(process.stdout)
                                    .ok()
                                    .expect("Failed to read");
    format!("{}", out.trim())
}

fn clock(fg: &str, ac: &str) -> String {
    let time = time_date::strftime("%Y-%m-%d %R", &time_date::now()).unwrap();
    format!("{}{}{}", ac, fg, time)
}

fn main() {
    let fg = "%{F#d9e1e8}";
    let ac = "%{F#9baec8}";
    //print!("{} {}\n", battery(fg, ac), clock(fg, ac));
    loop {
        println!("{}%{{c}}{}%{{r}}{}{}{}{} ", nowplaying(fg, ac), window_title(), battery(fg, ac), wifi_bssid(fg, ac), volume(fg, ac), clock(fg, ac));
        thread::sleep(time::Duration::from_millis(200));
    }
}
