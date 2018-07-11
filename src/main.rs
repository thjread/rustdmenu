use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

extern crate serde;
extern crate serde_json;

const SAVEFILE: &'static str = ".rustdmenu_save";
const DMENU_ARGS: [&'static str; 10] = ["-o", "0.8", "-nb",
    "#2b2b2b", "-nf", "#839496", "-sb", "#268bd2", "-sf", "#eee8d5"];

fn save_path() -> PathBuf {
    let mut path = env::home_dir().unwrap();
    path.push(SAVEFILE);
    return path;
}

fn load_map() -> HashMap<String, i32> {
    let path = save_path();
    match File::open(path) {
        Ok(mut f) => {
            let mut data = String::new();
            f.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        }
        Err(_) => {
            let m: HashMap<String, i32> = HashMap::new();
            m
        }
    }
}

fn map_to_list(prog_map: &HashMap<String, i32>) -> Vec<&str> {
    let mut prog_map_list: Vec<(&String, &i32)> = prog_map.iter().collect();
    prog_map_list.sort_by(|a, b| b.1.cmp(a.1));
    prog_map_list.into_iter().map(|(a, _)| &a[..]).collect()
}

fn update_used(prog_map: &mut HashMap<String, i32>, used: &str) {
    let times = prog_map.entry(used.to_string()).or_insert(0);
    *times += 1;
}

fn save_map(prog_map: &HashMap<String, i32>) {
    let path = save_path();
    let mut file = File::create(path).expect("Failed to open file for writing");
    let encode = serde_json::to_string(&prog_map).unwrap();
    file.write_all(encode.as_bytes())
        .expect("File writing failed");
}

fn delete(prog: &str) {
    let mut prog_map = load_map();
    prog_map.remove(prog);
    save_map(&prog_map);
}

fn dmenu(maybe_font: Option<&str>) {
    let dmenu_path_output = Command::new("dmenu_path")
        .output()
        .expect("Failed to run dmenu_path");
    let dmenu_path_out = String::from_utf8_lossy(&dmenu_path_output.stdout);

    let mut prog_map = load_map();
    let mut dmenu_process;

    {
        let mut prog_list = map_to_list(&prog_map);

        for prog in dmenu_path_out.split("\n") {
            if !prog_map.contains_key(prog) {
                prog_list.push(prog);
            }
        }

        load_map();

        let mut args: Vec<&str> = Vec::new();
        args.extend(&DMENU_ARGS);
        if let Some(font) = maybe_font {
            args.push("-fn");
            args.push(font);
        }

        dmenu_process = Command::new("dmenu")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn dmenu");
        dmenu_process
            .stdin
            .take()
            .unwrap()
            .write_all(prog_list.join("\n").as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = dmenu_process
        .wait_with_output()
        .expect("Failed to wait for dmenu");
    let s = String::from_utf8_lossy(&output.stdout);
    let used = s.trim();

    println!("{}", used);

    if !used.eq("") {
        update_used(&mut prog_map, used);
    }

    save_map(&prog_map);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 && args[1].eq("delete") {
        delete(&args[2]);
    } else if args.len() > 2 && args[1].eq("-fn"){
        dmenu(Some(&args[2]));
    } else {
        dmenu(None);
    }
}
