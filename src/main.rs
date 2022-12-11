use dns_lookup::lookup_host;
use scraper::Selector;
use std::env;
use std::fs::File;
use std::io::stdin;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::path::Path;
use std::process;

fn detected_wildcard(domain: &String) -> bool {
    let subsubdomain: String = "test.wildcard.".to_string();
    let subdomain: String = "wildcard.".to_string();

    let hostname: String = subsubdomain + domain;
    let hostname1: String = subdomain + domain;

    if lookup_host(hostname.trim()).is_ok() && lookup_host(hostname1.trim()).is_ok() {
        let sub_ips: Vec<IpAddr> = lookup_host(hostname1.trim()).unwrap();
        let subsub_ips: Vec<IpAddr> = lookup_host(hostname.trim()).unwrap();

        return subsub_ips == sub_ips;
    }

    return false;
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn attr_exists(nth: usize) -> bool {
    env::args().nth(nth).is_some()
}

fn get_attr(nth: usize) -> String {
    env::args().nth(nth).expect("Attribute is required")
}

fn main() {
    let mut host: String = String::new();
    let mut filename: String = String::new();

    if attr_exists(1) {
        host = get_attr(1);
    } else {
        println!("Enter a domain: ");
        stdin()
            .read_line(&mut host)
            .expect("Need domain name")
            .to_string();
    }

    if attr_exists(2) {
        filename = get_attr(2);
    } else {
        println!("Enter a subdomains file: ");
        stdin()
            .read_line(&mut filename)
            .expect("Need filename name")
            .to_string();
    }

    if detected_wildcard(&host) {
        println!("[ * ] Wildcard detected!");
        println!("Domain {:?} probably have wildcard!", host);
        process::exit(0x0100);
    }

    if let Ok(lines) = read_lines(filename) {
        for line in lines.flatten() {
            let subdomain = line;
            let hostname: String = subdomain + "." + &host;
            let mut info: String = hostname.clone();

            if lookup_host(hostname.trim()).is_ok() {
                if reqwest::blocking::get("https://".to_string() + &hostname).is_ok() {
                    let data = reqwest::blocking::get("https://".to_string() + &hostname)
                        .unwrap()
                        .text();
                    match data {
                        Ok(body) => {
                            let document = scraper::Html::parse_document(&body);
                            if Selector::parse("title").is_ok() {
                                let selector = Selector::parse("title").unwrap();

                                if document.select(&selector).next().is_some() {
                                    let title = document.select(&selector).next().unwrap();
                                    info = info + ": " + &title.inner_html();
                                }
                            }
                        }
                        Err(e) => println!("Something wrong {:?}", e),
                    }
                }

                let ips: Vec<IpAddr> = lookup_host(hostname.trim()).unwrap();
                println!("{:?} {:?},", info, ips)
            }
        }
    }
}
