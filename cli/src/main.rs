extern crate clap;
use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;
use std::time::{Instant};
use lazy::semantic_analyzer::{file_host::{FSFileHost, FileHost}};
use lazy::errors::builder::ErrorFormatter;

fn get_extention_validity(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

fn main() {
    let matches = App::new("lazy")
    .version("0.0.0") 
    .about("A command-line interface to interact with the lazy programming language")
    .arg(
    Arg::new("run")
    .short('r')
    .long("run")
    .about("Runs a lazy file")
    .takes_value(true)
    )
    .arg(
    Arg::new("time")
    .short('t')
    .long("time")
    .about("Shows you the parsing time of the code")
    .takes_value(false)
    )
    .get_matches();

if let Some(exe_file) = matches.value_of("run") {
    if Path::new(&exe_file).exists() {
        if get_extention_validity(&exe_file) == Some("lazy") {
            let mut files = FSFileHost::new();
            let before = Instant::now();
            let module = files.create(&exe_file);
            if matches.is_present("time") {
                println!("Parsing took {} nanoseconds", before.elapsed().as_nanos());
            }
            match module {
                Err(errors) => {
                    for error in errors {
                        println!("{}", files.format_err(&error).unwrap());
                    }
                },
                Ok(_module) => {
                    println!("Successfully parsed and analyzed module!");
                }
            }
        }
        else{
            println!("Could not parse the source code. Error: Could not find a lazy file.")
        }

    }
    else{
        println!("Path does not exist.")
    }
}
}