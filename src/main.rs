#[macro_use]
extern crate clap;
extern crate regex;

use ansi_term::Style;
use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{prelude::*, BufReader, Result, SeekFrom};
use std::process::Command;

// https://developer.apple.com/library/archive/technotes/tn2151/_index.html#//apple_ref/doc/uid/DTS40008184-CH1-SYMBOLICATE_WITH_ATOS

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("crash_log")
                .required(true)
                .short("c")
                .long("crash_log")
                .takes_value(true)
                .help("The path to a crash log"),
        )
        .arg(
            Arg::with_name("app")
                .required(true)
                .short("a")
                .long("app")
                .takes_value(true)
<<<<<<< HEAD
                .help("The path to an application binary (eg. Some.app/Some)"),
=======
                .help("The path to an application binary (eg. Trello.app/Trello)"),
>>>>>>> 5c64fab... Producing a valid highlighted crash log
        )
        .get_matches();

    let crash_log_path = matches.value_of("crash_log").unwrap();
    let mut crash_file = File::open(crash_log_path)?;

    let app = matches.value_of("app").unwrap();

    let arch = determine_architecture(&crash_file)
        .expect(format!("Could not determine architecture from {}", crash_log_path).as_str());

    println!("{}", arch);

    crash_file.seek(SeekFrom::Start(0))?;

    // 2    XYZLib    0x34648e88    0x83000 + 8740
    let regex = Regex::new(r"^(.*)(0x[a-fA-F0-9]+) (0x[a-fA-F0-9]+) .*$")
        .expect("Regular expression was invalid");

    for line in BufReader::new(&crash_file)
        .lines()
        .filter_map(|r| r.ok())
        .take_while(|l| !l.starts_with("Binary Images:"))
    {
        if let Some(captures) = regex.captures(&line) {
            if let (Some(prefix), Some(method_address), Some(main_address)) =
                (&captures.get(1), &captures.get(2), &captures.get(3))
            {
                // xcrun atos -o Trello.app/Trello -arch arm64 -l 0x102194000 0x102b60ffc
                let result = Command::new("xcrun")
                    .args(&[
                        "atos",
                        "-o",
                        app,
                        "-arch",
                        &arch,
                        "-l",
                        main_address.as_str(),
                        method_address.as_str(),
                    ])
                    .output();
                if let Ok(output) = result {
                    if let Ok(symbolicated_line) = String::from_utf8(output.stdout) {
                        let style = Style::new().bold();
                        println!(
                            "{}{} {}",
                            style.paint(prefix.as_str()),
                            style.paint(method_address.as_str()),
                            style.paint(symbolicated_line.trim())
                        );
                        continue;
                    }
                }
            }
        }
        println!("{}", line);
    }

    Ok(())
}

fn determine_architecture(crash_file: &File) -> Option<String> {
    let binary_image_line = BufReader::new(crash_file)
        .lines()
        .filter_map(|r| r.ok())
        .skip_while(|l| !l.starts_with("Binary Images:"))
        .skip(1)
        .next()
        .expect("Could not find `Binary Images:` section");

    let binary_regex =
        Regex::new(r"^0x[a-fA-F0-9]+ - 0x[a-fA-F0-9]+ .+ ([\S]+)  <[a-fA-F0-9]+> .*$")
            .expect("Binary regular expression was invalid");

    if let Some(captures) = binary_regex.captures(&binary_image_line) {
        if let Some(arch) = &captures.get(1) {
            return Some(String::from(arch.as_str()));
        }
    }

    None
}
