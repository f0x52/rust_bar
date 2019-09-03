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

	let mut average: [u32; 0] = [];
	let mut count = 0;

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
			let as_int = capacity.parse::<u32>().expect("could not parse battery percentage as int ");
			average.push(as_int);

            format!("{}% ", capacity)
        })
        .fold("".to_string(), |mut values, cap| {values.push_str(&cap); values});
    let battery_cap = format!("{}", batteries);

    format!("{}{}{}{}", ac, icon, fg, battery_cap)
}

fn nowplaying(fg: &str, ac: &str, re: &regex::Regex) -> String {
    //first check if cmus is playing
    //
    let process = Command::new("cmus_playing")
                                .output()
                                .expect("failed to execute cmus_playing");
    let out = std::string::String::from_utf8(process.stdout)
                                .expect("failed to read");
    if out != "" {
        return format!("{}{}{}", ac, fg, out);
    }

    let replacements = ["YouTube", " (Official Video)", " (HQ)", " (Official Video)", " [FULL MUSIC VIDEO]", " (320kbps)", " (with lyrics)", " - Full album", " [Official Music Video]", "(Official Music Video)", "(Lyric Video)", "(lyric video)", "[HQ]", "High Quality Sound", "HD 720p", "[Lyrics]", "(MUSIC VIDEO)", "(Drive Original Movie Soundtrack)", "(Official Audio)", "- Official Video", "(Audio)", " - OFFICIAL VIDEO", "(official video)", "Official Video"];
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

fn telegram_unread(fg: &str, ac: &str, re: &regex::Regex) -> String {
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
                output = format!("{}{}{} ", ac, fg, unread);
            }
        }
    }
    output
}

//fn watson_status(fg: &str, ac: &str) -> String {
//    let mut output = String::new();
//    let process = Command::new("watson_stat")
//                                    .output()
//                                    .unwrap();
//    let watson = String::from_utf8(process.stdout)
//                                    .expect("Failed to read");
//    if watson.split_whitespace().next().is_some(){
//        if watson != "No project started" {
//            output = format!("{}{}{}", ac, fg, watson);
//        }
//    }
//    output
//}

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
    //let fg = "%{F#d9e1e8}";
    //let fg = "%{F#c2c2c2}";
    //let ac = "%{F#cc6666}";
    //let fg = "%{F#d9e1e8}";
    //let ac = "%{F#600b0b}";
    //let ac = "%{F#9baec8}";
    //let ac = "%{F#81a1c1}"; //light blue
    //let ac = "%{F#c45500}"; // orangy
    //let ac = "%{F#a54242}"; // light red
    //let ac = "%{F#cc6666}"; //lighter red
    let yt_re = Regex::new(r"\(\d+\) ").unwrap();
    let unread_re = Regex::new(r"\((\d+)\)").unwrap();

    loop {
        //println!(" {}%{{c}}{}%{{r}}{}{}{}{}{} ", nowplaying(fg, ac, &yt_re), window_title(), telegram_unread(fg, ac, &unread_re), battery(fg, ac), wifi_bssid(fg, ac), volume(fg, ac), clock(fg, ac));
        // BASE16-OCEAN
        //println!(" {}%{{c}}%{{F#ebcb8b}}{}%{{r}}{}{}{}{}{} ", nowplaying("%{F#a3be8c}", "%{F#a3be8c}", &yt_re), window_title(), telegram_unread("%{F#bf616a}","%{F#bf616a}", &unread_re), battery("%{F#8fa1b3}", "%{F#8fa1b3}"), wifi_bssid("%{F#b48ead}", "%{F#b48ead}"), volume("%{F#96b5b4}", "%{F#96b5b4}"), clock("%{F#a3be8c}", "%{F#a3be8c}"));
        // pink
        //println!(" %{{F#d8a198}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // red
        //println!(" %{{F#933d3d}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // baby blue
        //println!(" %{{F#bdd0d6}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // light purple
        //println!(" %{{F#bb8487}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // blank
        println!(" {}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // purple
        //println!(" %{{F#9951a8}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // dark red
        //println!(" %{{F#cd9b35}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // white
        //println!(" %{{F#f3f3ea}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // yellow-green
        //println!(" %{{F#bde077}}{}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying("", "", &yt_re), window_title(), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        // CYB
        //println!(" {}%{{c}}%{{F#ba58ae}}{}%{{r}}{}{}{}{}{} ", nowplaying("%{F#9b5aa9}", "%{F#9b5aa9}", &yt_re), window_title(), telegram_unread("%{F#4f5dc6}","%{F4f5dc6}", &unread_re), battery("%{F#a6337d}", "%{F#a6337d}"), wifi_bssid("%{F#702e70}", "%{F#702e70}"), volume("%{F#e33f78}", "%{F#e33f78}"), clock("%{F#9b5aa9}", "%{F#9b5aa9}"));
        // base16 println!(" {}%{{c}}%{{F#ba58ae}}{}%{{r}}{} {}{}{}{} ", nowplaying("%{F#a54242}", "%{F#a54242}", &yt_re), window_title(), telegram_unread("%{F#8c9440}","%{F#8c9440}", &unread_re), battery("%{F#d18a2e}", "%{F#d18a2e}"), wifi_bssid("%{F#5f819d}", "%{F#5f819d}"), volume("%{F#85678f}", "%{F#85678f}"), clock("%{F#5e8d87}", "%{F#5e8d87}"));
        // purple println!(" %{{F#ba58ae}}{}%{{c}}{}%{{r}}{} {}{}{}{}{} ", nowplaying("", "", &yt_re), window_title(), watson_status("", ""), telegram_unread("","", &unread_re), battery("", ""), wifi_bssid("", ""), volume("", ""), clock("", ""));
        //let ac = String::from("%{F#828282}");
        //let fg = String::from("%{F#e3e3e3}");
        //println!(" {}%{{c}}{}%{{r}}{} {}{}{}{} ", nowplaying(&fg, &ac, &yt_re), window_title(), telegram_unread(&fg,&ac, &unread_re), battery(&fg, &ac), wifi_bssid(&fg, &ac), volume(&fg, &ac), clock(&fg, &ac));
        //colorful

        //let red =    "%{F#ae1e28}";
        //let purple = "%{F#923f87}";
        //let orange = "%{F#ca5a2a}";
        //let blue =   "%{F#13487c}";
        //let cyan =   "%{F#2a4e70}";
        //

        //println!(" {}%{{c}}{}{}%{{r}}{} {}{}{}{} ", nowplaying(&red, "%{F#f24632}", &yt_re), &purple, window_title(), telegram_unread(&red, &red, &unread_re), battery(&orange, &orange), wifi_bssid(&blue, &blue), volume(&purple, &purple), clock(&cyan, &cyan));
        thread::sleep(time::Duration::from_millis(200));
    }
}
