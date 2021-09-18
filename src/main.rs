use clap::{App, Arg, SubCommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use nix::unistd::Uid;
use regex::Regex;
use std::{convert::TryInto, process::Command};

fn main() {
    let matches = App::new("pertrickstence")
        .about(
            "Allows for persistence with debian packages without needing a persistence partition by using a folder on the flash drive",
        )
        .help("Path to USB stick. Used to store saved packages. You can drag and drop the root folder from your flash drive")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .version("1.0.0")
        .author("Jabster28 <justynboyer@gmail.com>")
        .subcommand(
            SubCommand::with_name("add")
                .about("add a package to your live debian install and the flash drive").arg(Arg::with_name("packages").value_name("PACKAGES") .help("Packages to install").case_insensitive(true).takes_value(true).index(1).required(true).multiple(true)).arg(Arg::with_name("only-needed").short("o").long("only-needed").help("Only append packages that aren't already present. NOTE: MAY BREAK FUTURE INSTALLS IF VANILLA APT IS USED DURING SESSION"))
        .arg(Arg::with_name("path").long("path").short("p").case_insensitive(true).takes_value(true).required(true)))
        .subcommand(
            SubCommand::with_name("install")
                .about("install all packages to your live install").arg(Arg::with_name("path").long("path").short("p").case_insensitive(true).takes_value(true).required(true)))
        .get_matches();
    if !Uid::effective().is_root() {
        panic!("You must run this executable with root permissions");
    }
    if let Some(ref matches) = matches.subcommand_matches("add") {
        let dir = std::path::Path::new(matches.value_of("path").unwrap());
        let _a = Command::new("mkdir")
            .current_dir(dir)
            .arg("-p")
            .arg(".pertrickstenceDownloads")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        println!("{}", "Running `apt-get update`...".green());
        Command::new("apt-get")
            .arg("update")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        println!("Saving apt lists...");
        Command::new("tar")
            .args("zcf".split(' '))
            .arg(dir.join(".pertrickstenceDownloads/lists.tar.gz"))
            .arg("/var/lib/apt/lists/")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        let o: Vec<&str> = matches.values_of("packages").unwrap().collect();
        // solution taken from https://stackoverflow.com/a/45489718
        let mut g = vec![];
        let x = Command::new("apt-cache").args("depends --recurse --no-recommends --no-suggests --no-conflicts --no-breaks --no-replaces --no-enhances --no-pre-depends".split(' ')).args(o).output().unwrap();
        let regex = Regex::new(r"(?m)^\w.*").unwrap();

        // result will be an iterator over tuples containing the start and end indices for each match in the string
        let result = regex.captures_iter(std::str::from_utf8(&x.stdout).unwrap());
        let w = result.map(|e| e.get(0).map_or("", |f| f.as_str()));
        for i in w {
            g.push(i)
        }
        let mut j = vec![];
        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");
        let pb = ProgressBar::new(g.len().try_into().unwrap());
        pb.set_style(sty);
        if matches.is_present("only-needed") {
            let u = &Command::new("dpkg")
                .arg("--get-selections")
                .output()
                .unwrap()
                .stdout
                .clone();
            let txt = std::str::from_utf8(u).unwrap();
            g.iter().for_each(|f| {
                pb.set_message(format!("Checking for {}", f));
                let reg = Regex::new(&(format!(r"(?m)^{}\b", f))[..]).unwrap();
                let res = reg.is_match(txt);
                pb.inc(1);
                if !res {
                    j.push(f);
                }
            });
            pb.finish();
        }
        let mut l = vec![];
        if matches.is_present("only-needed") {
            j.iter().for_each(|&f| l.push(f));
        } else {
            g.iter().for_each(|f| l.push(f));
        }
        let cool: Vec<String> = l.iter().map(|f| f.green().to_string()).collect();
        println!("Downloading: {}", cool.join(", "));
        Command::new("apt-get")
            .arg("download")
            .arg("-y")
            .args(l.clone())
            .current_dir(dir.join(".pertrickstenceDownloads"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        Command::new("apt-get")
            .arg("install")
            .arg("-y")
            .args(l)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    } else if let Some(ref matches) = matches.subcommand_matches("install") {
        let dir = std::path::Path::new(matches.value_of("path").unwrap());
        println!("{}", "Re-syncing database...");
        Command::new("tar")
            .args("zxf".split(' '))
            .arg(dir.join(".pertrickstenceDownloads/lists.tar.gz"))
            .current_dir("/")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        Command::new("dpkg")
            .arg("-i")
            .arg("*.deb")
            .current_dir(dir.join(".pertrickstenceDownloads"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        Command::new("apt-get")
            .arg("--fix-broken")
            .arg("install")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
