extern crate regex;
extern crate getopts;
use std::collections::{HashMap, HashSet};
use std::{env, fs, io};
use std::io::prelude::*;
mod options;
use options::*;

macro_rules! regex { ($re:expr) => { ::regex::Regex::new($re).unwrap() } }

fn replace_var(line: String, keys: &HashMap<String, String>, result: &mut Vec<String>) {
    let re = regex!(r"(?P<full>\$\{\s*(?P<var>\S*)\s*\})");
    let mut str_line: String = line.clone();

    for cap in re.captures_iter(&*str_line.clone()) {
        cap.name("var").map(|v| {
            env_or_prop(v, keys).map(|v2|{
                str_line = re.replace(&*str_line, &*v2);
            });
        });
    }

    result.push(str_line.to_string());
}

fn env_or_prop(key: &str, keys: &HashMap<String, String>) -> Option<String> {
    env::var_os(key).or(
        keys.get(key).map(|v2| { 
            std::ffi::OsString::from(v2 as &String)
        })
        ).and_then(|v| v.into_string().ok())
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

fn extract_keys(file: fs::File) -> HashMap<String, String> {
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

fn free_args(flag_args: HashSet<String>, free_args: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    println!("{}", flag_args.len());
    for a in free_args {
        println!("{}", a);
        if !flag_args.contains(&a) {
            result.push(a);
        }
    }

    result
}

#[allow(dead_code)]
fn main() {
    let arg_parser: ArgOpts  = ArgOpts::new(|| {
        let mut o = ApplicationOptions::new();
        o.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
        o.opt("k", "keys", 
              "keyfile with variable substitutions", "FILE", 
              getopts::HasArg::Yes, getopts::Occur::Optional).unwrap();
        o.opt("p", "props-first", 
              "properties takes precedence over environment variables (default: off)", "", 
              getopts::HasArg::No, getopts::Occur::Optional).unwrap();
        o.opt("?", "help", 
              "print this help menu", "",
              getopts::HasArg::No, getopts::Occur::Optional).unwrap();
        Ok(o)
    })/*.unwrap_or_else(|e| {
        panic!(e);
    })*/;

    let matches: getopts::Matches = arg_parser.parse().unwrap_or_else(|e| {
        panic!(e);
    });

    let mut opt_args: HashSet<String> = HashSet::new(); 

    let key_file: Option<fs::File> = if matches.opt_present("k") {
        match matches.opt_str("k") {
            Some(v) => {
                if arg_parser.has_option(&*v) {
                    println!("ERROR: Expected key-filename, found: {}", &*v);
                    arg_parser.print_usage_and_panic();
                    None
                } else {
                    //println!("key_filename {}", v);
                    opt_args.insert(v.clone());
                    fs::File::open(&v).ok()
                }
            },
            None => { 
                println!("ERROR: Expected file parameter after 'keys' parameter");
                arg_parser.print_usage_and_panic();
                None
            }
        }
    } else {
        None
    };
    
    let free_args = free_args(opt_args, matches.free);

    let prop_filename;
    let result_filename;
    match free_args.len() {
        1 => {
            // Replace vars in the provided file
            prop_filename = &free_args[0];
            result_filename = &free_args[0];
        },
        2 => {
            // Replace vars into a separate output file
            prop_filename = &free_args[0];
            result_filename = &free_args[1];
        },
        _ => {
            println!("Wrong number of file parameters: {}", free_args.len());
            arg_parser.print_usage();
            return;
        }
    }

    let prop_file = fs::File::open(&prop_filename);//  open_file(&prop_filename);
    let mut result_file = fs::File::create(result_filename).unwrap();
   
    key_file.map(|k| {
        for line in process(prop_file.unwrap(), &extract_keys(k)).iter().enumerate() {
            println!("{}: {}", line.0, line.1);
            result_file.write_all(&format!("{}\n", line.1).as_bytes()).unwrap();
        }
    });
}
