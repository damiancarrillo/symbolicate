#[macro_use]
extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Result, SeekFrom, Write};
use std::process::Command;
use ansi_term::Style;

// https://developer.apple.com/library/archive/technotes/tn2151/_index.html#//apple_ref/doc/uid/DTS40008184-CH1-SYMBOLICATE_WITH_ATOS

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("crash_file_path").required(true).index(1))
        .get_matches();

    let crash_file_path = matches.value_of("crash_file_path").unwrap();
    let mut crash_file = File::open(crash_file_path)?;

    let arch = determine_architecture(&crash_file)
        .expect(format!("Could not determine architecture from {}", crash_file_path).as_str());

    println!("{}", arch);

    crash_file.seek(SeekFrom::Start(0))?;

    // xcrun atos -o Trello.app/Trello -arch arm64 -l 0x102194000 0x102b60ffc

    // 2    XYZLib    0x34648e88    0x83000 + 8740
    let regex = Regex::new(r"^(.*)(0x[a-fA-F0-9]+) (0x[a-fA-F0-9]+) .*$")
        .expect("Regular expression was invalid");

    for line in BufReader::new(&crash_file)
        .lines()
        .filter_map(|r| r.ok())
        .take_while(|l| !l.starts_with("Binary Images:"))
    {
        if let Some(captures) = regex.captures(&line) {
            if let (Some(a), Some(b), Some(c)) = (
                &captures.get(1),
                &captures.get(2),
                &captures.get(3),
            ) {
                let result = Command::new("xcrun")
                    .args(&[
                        "atos",
                        "-o",
                        "Trello.app/Trello",
                        "-arch",
                        &arch,
                        "-l",
                        c.as_str(),
                        b.as_str(),
                    ])
                    .output();
                if let Ok(output) = result {
                    // println!("yo yo yo");
                    print!("{}{} ", Style::new().bold().paint(a.as_str()), Style::new().bold().paint(b.as_str()));
                    io::stdout().write_all(&output.stdout).unwrap();
                    continue;
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
