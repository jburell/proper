extern crate regex;
use std::{env, fs, io};
use std::collections::HashMap;
use std::io::prelude::*;

macro_rules! regex { ($re:expr) => { ::regex::Regex::new($re).unwrap() } }

fn replace_var(line: String, keys: &HashMap<String, String>, result: &mut Vec<String>) {
    let re = regex!(r"(?P<full>\$\{\s*(?P<var>\S*)\s*\})");
    let mut str_line: String = line.clone();

    for cap in re.captures_iter(&*str_line.clone()) {
        cap.name("var").map(|v| { 
            keys.get(v).map(|v2| { 
                str_line = re.replace(&*str_line, &*v2 as &str);
            })
        });
    }

    result.push(str_line.to_string());
}

fn process(file: fs::File, dict: &HashMap<String, String>) -> Vec<String> {
    let buf_reader = io::BufReader::new(&file);
    let mut result: Vec<String> = vec!();

    println!("Skriver ut: ");
    for line in buf_reader.lines() {
        replace_var(line.unwrap(), &dict, &mut result);
    }

    result
}

fn extract_keys(file: fs::File) -> HashMap<String, String>{
    let key_regex = regex!(r"^\s*(?P<key>\S*)\s*=\s*(?P<val>.*)");
    let buf_reader = io::BufReader::new(&file);
    let mut key_map = HashMap::new();

    for line in buf_reader.lines().enumerate() {
        for cap in key_regex.captures_iter(&*line.1.ok().expect("Could not read line")) {
            let key = cap.name("key").unwrap_or("???");
            let val = cap.name("val").unwrap_or("???");
            key_map.insert(key.to_string(), val.to_string());
            println!("{}: key({}), val({})", line.0, key, val);
        }
    }

    key_map
}

fn open_file(filename: &str) -> fs::File {
    fs::File::open(filename)
                .ok().expect(&*format!("Could not open: {}", filename))
}

#[allow(dead_code)]
fn main() {
    let mut args = env::args();
    if args.len() != 4 {
        println!("Usage: {} <property-file> <key-file> <result-file>", 
                 args.next().unwrap());
        std::process::exit(1); 
    }

    let prop_filename = args.nth(1).unwrap();
    let key_filename = args.next().unwrap();
    let result_filename = args.next().unwrap();

    let prop_file = open_file(&*prop_filename);
    let key_file = open_file(&*key_filename);

    let mut result_file = fs::File::create(result_filename).unwrap();
    for line in process(prop_file, &extract_keys(key_file)).iter().enumerate() {
        println!("{}: {}", line.0, line.1);
        result_file.write_all(&format!("{}\n", line.1).as_bytes()).unwrap();
    }
}
