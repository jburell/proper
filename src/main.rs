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

fn map_args(args: &Vec<String>) -> HashMap<ValidArgs, Option<String>> {
    let mut arg_map = HashMap::new();
    for i in args {
        arg_map.insert(ValidArgs::PropertyFilename, None /*Some("val1".to_string())*/);
    }
    arg_map
}

fn validate_args<'a>(args: &'a HashMap<ValidArgs, Option<String>>) -> Result<&'a HashMap<ValidArgs, Option<String>>, &str> {
    match args.len() != 4 {
        true => Ok(args),
        false => Err("There needs to be 4 args"),
    }
}

static help_txt: &'static str = "Usage: proper [OPTIONS] <property-file> <result-file>";

#[derive(Eq)]
#[derive(Hash)]
enum ValidArgs {
    PropertyFilename,
    KeysFilename,
    ResultFilename,
}

impl PartialEq for ValidArgs {
    fn eq(&self, other: &ValidArgs) -> bool {
        self == other
    }
    fn ne(&self, other: &ValidArgs) -> bool {
        self != other
    }
}

#[allow(dead_code)]
fn main() {
  /*  let mut cmd_args = env::args();
    let application_name = cmd_args.next().unwrap();
    let args = &map_args(&cmd_args.collect());
    let valid_args = validate_args(&args).unwrap_or_else(|v| {
        println!("ERROR: {}", v);
        println!("{}", help_txt);
        std::process::exit(1);
    });

    let prop_maybe = valid_args.get(&ValidArgs::PropertyFilename);
    let prop_maybe2 = prop_maybe.unwrap();
    let prop_filename = prop_maybe2.unwrap();
    let key_filename = args.get(&ValidArgs::KeysFilename).unwrap().unwrap();
    let result_filename = args.get(&ValidArgs::ResultFilename).unwrap().unwrap();
    let prop_file = open_file(&*prop_filename);
    let key_file = open_file(&*key_filename);

    let mut result_file = fs::File::create(result_filename).unwrap();
    for line in process(prop_file, &extract_keys(key_file)).iter().enumerate() {
        println!("{}: {}", line.0, line.1);
        result_file.write_all(&format!("{}\n", line.1).as_bytes()).unwrap();
    }*/
}
