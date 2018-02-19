// This is a build file I've accumulated over time and I use in most of my
// projects, currently its purpose is to bump the build number and set the env
// variable `CARGO_PKG_PROFILE` to either `release` or `debug` based on what
// type of compilation is happening. Useful if you want to for example
// differentiate your development and prod logs on datadog.

extern crate git_version;
extern crate regex;
use regex::{ Regex, Captures };

use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let mut file1 = File::open("Cargo.toml").unwrap();
    let mut data = String::new();
    let _ = file1.read_to_string(&mut data);
    drop(file1);
    let mut file2 = File::create("Cargo.toml").unwrap();
    //Our holy regex for matching version number from TOML
    // (version\\s?=\\s?\"\\d+\\.\\d+\\.\\d+-.+?\\+build\\.)(\\d+)(\")
    let re = Regex::new("(version\\s?=\\s?\"\\d+\\.\\d+\\.\\d+-.+?\\+build\\.)(\\d+)(\")").unwrap();
    let new = re.replace(data.as_str(), |caps: &Captures| {
        format!("{}{}{}", &caps[1], (&caps[2]).to_string().parse::<u64>().unwrap() + 1, &caps[3])
    }).to_string();

    let _ = file2.write_all(new.as_bytes());
    drop(file2);

    println!("cargo:rustc-env=CARGO_PKG_PROFILE={}", env::var("PROFILE").unwrap());
    git_version::set_env();
}
