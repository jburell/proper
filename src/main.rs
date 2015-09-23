extern crate regex;
extern crate getopts;
use std::collections::{HashMap, HashSet};
use std::{env, fs, io};
use std::io::prelude::*;
mod options;
use options::*;
use std::process;

macro_rules! regex { ($re:expr) => { ::regex::Regex::new($re).unwrap() } }
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        )
    );

const KEY_FILE: &'static str = "KEY_FILE";
const ENV: &'static str = "ENV";
const MISSING: &'static str = "Missing properties";

fn replace_var(line: String, 
               keys: Option<HashMap<String, String>>, 
               props_first: bool,
               used_keys: &mut HashMap<&'static str, Vec<(String, String)>>,
               result: &mut Vec<String>) {
   let re = regex!(r"(?P<full>\$\{\s*(?P<var>[^\}]*)\s*\})");
   let mut str_line: String = line.clone();

   for cap in re.captures_iter(&*str_line.clone()) {
       cap.name("var").map(|v| {
           let env_prop = env_or_prop(v.trim(), 
                                      props_first, 
                                      keys.clone()).map(|v2|{
                                          str_line = re.replace(
                                              &*str_line, 
                                              &*v2.1);
                                          (v2.0, v2.1, v2.2)
                                      });

           env_prop.map(|v2| {
               let op = {
                   let val = used_keys.get_mut(&v2.2);
                   match val {
                       Some(r) => {
                           Some(r.push((v2.0.clone(), v2.1.clone())))
                       },
                       None => {
                           None
                       }
                   }
               };
               if op.is_none() {
                   used_keys.insert(v2.2.clone(), 
                                    vec!((v2.0.clone(), v2.1.clone())));
               }
           })
       });
   }
   result.push(str_line.to_string());
}

fn env_or_prop(key: &str, 
               props_first: bool, 
               keys: Option<HashMap<String, String>>) 
                    -> Option<(String, String, &'static str)> {
    match keys {
        Some(k) => {
            let prop = k.get(key).and_then(|v| Some(v.clone()));
            let env = env::var_os(key).map(|v2| { 
                v2.clone().into_string().ok().unwrap()
            });
            match (prop, props_first) {
                (Some(prop), true) => {
                    Some((key.to_string(), prop.to_string(), KEY_FILE))
                },
                (Some(prop), false) => {
                    match env {
                        Some(e) => {
                            Some((key.to_string(), e.to_string(), ENV))
                        },
                        None => Some((key.to_string(), prop.to_string(), KEY_FILE)),
                    }
                },
                (None, true) => {
                    match env {
                        Some(e) => {
                            Some((key.to_string(), e.to_string(), ENV))
                        },
                        None => Some((key.to_string(), key.to_string(), MISSING)),
                    }

                },
                (None, false) => {match env {
                    Some(e) => {
                        Some((key.to_string(), e.to_string(), ENV))
                    },
                    None => Some((key.to_string(), key.to_string(), MISSING)),
                }

                }
            }
            },
        None => { 
            match env::var_os(key).map(|v| {
                v.into_string().ok()
            }).unwrap_or(None) {
                Some(e) => {
                    Some((key.to_string(), e.to_string(), ENV))
                },
                None => {
                    Some((key.to_string(), key.to_string(), MISSING))
                }
            }
        }
    }
}

fn process(file: fs::File,
           used_keys: &mut HashMap<&'static str, Vec<(String, String)>>,
           dict: Option<HashMap<String, String>>, 
           props_first: bool) -> Vec<String> {
   let buf_reader = io::BufReader::new(&file);
   let mut result: Vec<String> = vec!();

   for line in buf_reader.lines() {
       replace_var(line.unwrap(), 
                   dict.clone(), 
                   props_first, 
                   used_keys,
                   &mut result);
   }

   result
}

fn extract_keys(file: fs::File) -> HashMap<String, String> {
    let key_regex = regex!(r"^\s*(?P<key>\S*)\s*=\s*(?P<val>.*)");
    let buf_reader = io::BufReader::new(&file);
    let mut key_map = HashMap::new();

    for line in buf_reader.lines().enumerate() {
        for cap in key_regex.captures_iter(&*line.1
                                           .ok()
                                           .expect("Could not read line")) {
           let key = cap.name("key")
               .unwrap_or("???");
           let val = cap.name("val")
               .unwrap_or("???");
           key_map.insert(key.to_string(),
           val.to_string());
        }
    }

    key_map
}

fn free_args(flag_args: HashSet<String>, 
             free_args: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for a in free_args {
        if !flag_args.contains(&a) {
            result.push(a);
        }
    }

    result
}

#[allow(dead_code)]
fn main() {
    let arg_parser: ApplicationOptions  = ApplicationOptions::new(|| {
        let mut o = OptionsAndFlags::new();
        o.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
        o.opt("k", "keys", 
              "keyfile with variable substitutions", "FILE", 
              getopts::HasArg::Yes, getopts::Occur::Optional).unwrap();
        o.opt("p", "props-first", 
              "properties takes precedence over environment variables (default: off)", 
              "", 
              getopts::HasArg::No, getopts::Occur::Optional).unwrap();
        o.opt("?", "help", 
              "print this help menu", "",
              getopts::HasArg::No, getopts::Occur::Optional).unwrap();
        Ok(o)
    });

    let matches: getopts::Matches = arg_parser.parse().unwrap_or_else(|e| {
        println_stderr!("{}", e);
        process::exit(1);
    });

    let mut opt_args: HashSet<String> = HashSet::new(); 

    let key_file: Option<fs::File> = if matches.opt_present("k") {
        match matches.opt_str("k") {
            Some(v) => {
                if arg_parser.has_option(&*v) {
                    println_stderr!("ERROR: Expected key-filename, found: {}", &*v);
                    arg_parser.print_usage_and_panic();
                    None
                } else {
                    opt_args.insert(v.clone());
                    match fs::File::open(&v) {
                        Ok(f) => Some(f),
                        Err(e) => {
                            println_stderr!(
                                "ERROR: Could not open key-file: {}\n{}\n", 
                                v, 
                                e);
                            arg_parser.print_usage_and_panic();
                            None
                        }
                    }
                }
            },
            None => { 
                println_stderr!("ERROR: Expected file parameter after 'keys' parameter");
                arg_parser.print_usage_and_panic();
                None
            }
        }
    } else {
        None
    };

    let key_filename = matches.opt_str("k");

    let props_first = if matches.opt_present("p") { true }else{ false };
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
            println_stderr!("Wrong number of file parameters: {}", free_args.len());
            arg_parser.print_usage();
            return;
        }
    }

    let prop_file = match fs::File::open(&prop_filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("ERROR: Could not open property-file: {}\n{}\n", 
                     prop_filename, 
                     e);
            arg_parser.print_usage_and_panic();
            None
        }
    }.unwrap();
    let mut result_buff = Vec::new();
    let keys = match key_file {
        Some(k) => Some(extract_keys(k)),
        None => None,
    };

    let mut used_keys =  HashMap::new();
    for line in process(prop_file, &mut used_keys, keys, props_first).iter() {
        result_buff.push(line.clone());
    }

    println!("======== RESULTS ========");

    let env_vars = used_keys.get(ENV);
    if env_vars.is_some() {
        let vars = env_vars.unwrap();
        println!("Environment variables:");
        for v in vars {
            println!("${{{}}}", v.0);
        }
        println!("");
    }

    let prop_vars = used_keys.get(KEY_FILE);
    if prop_vars.is_some() {
        let vars = prop_vars.unwrap();
        println!("From keyfile {}:", key_filename.unwrap()) ;
        for v in vars {
            println!("${{{}}}", v.0);
        }
        println!("");
    }

    let missing_vars = used_keys.get(MISSING);
    if missing_vars.is_some() {
        let vars = missing_vars.unwrap();
        println!("Missing variables found in {}:", prop_filename);
        for v in vars {
            println!("${{{}}}", v.0);
        }
        println!("");
        println_stderr!("ERROR: Must replace all found variables\n\n");
        process::exit(1);
    }

    if env_vars.is_none() && prop_vars.is_none() && missing_vars.is_none() {
        println!("[No variables to substitute!]\n");
    }

    let mut result_file = match fs::File::create(result_filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("ERROR: Could not open result-file: {}\n{}\n", 
                     result_filename, 
                     e);
            arg_parser.print_usage_and_panic();
            None
        }
    }.unwrap();

    for line in result_buff {
        result_file.write_all(&format!("{}\n", line).as_bytes()).unwrap();
    }
    result_file.flush().unwrap(); 

    println!("...DONE!");
}
