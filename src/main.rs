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
               keys: &Option<HashMap<String, ValueAndSource>>,
               props_first: bool,
               used_keys: &mut HashMap<&'static str, Vec<(String, String)>>,
               result: &mut Vec<String>) {
    let re = regex!(r"(?P<full>\$\{\s*(?P<var>[^\}]*)\s*\})");
    let mut str_line: String = line.clone();

    for cap in re.captures_iter(&*str_line.clone()) {
        cap.name("var").map(|v| {
            let env_prop = env_or_prop(v.trim(), props_first, keys.clone()).map(|v2| {
                str_line = re.replace(&*str_line, &*v2.1);
                (v2.0, v2.1, v2.2)
            });

            env_prop.and_then(|v2| {
                used_keys.get_mut(&v2.2)
                         .and_then(|r| Some(r.push((v2.0.clone(), v2.1.clone()))))
                         .or_else(|| {
                             used_keys.insert(v2.2.clone(), vec![(v2.0.clone(), v2.1.clone())]);
                             Some(())
                         })
            })
        });
    }
    result.push(str_line.to_string());
}

fn env_or_prop(key: &str,
               props_first: bool,
               keys: &Option<HashMap<String, ValueAndSource>>)
               -> Option<(String, String, &'static str)> {
    match *keys {
        Some(ref k) => {
            let prop = k.get(key).and_then(|v| Some(v.clone()));
            let env = env::var_os(key).map(|v2| v2.clone().into_string().ok().unwrap());
            match (prop, props_first) {
                (Some(prop), true) => {
                    Some((key.to_string(), prop.value.clone(), KEY_FILE))
                }
                (Some(prop), false) => {
                    match env {
                        Some(e) => {
                            Some((key.to_string(), e.to_string(), ENV))
                        }
                        None => Some((key.to_string(), prop.value.clone(), KEY_FILE)),
                    }
                }
                (None, true) => {
                    match env {
                        Some(e) => {
                            Some((key.to_string(), e.to_string(), ENV))
                        }
                        None => Some((key.to_string(), key.to_string(), MISSING)),
                    }

                }
                (None, false) => {
                    match env {
                        Some(e) => {
                            Some((key.to_string(), e.to_string(), ENV))
                        }
                        None => Some((key.to_string(), key.to_string(), MISSING)),
                    }

                }
            }
        }
        None => {
            match env::var_os(key)
                      .map(|v| v.into_string().ok())
                      .unwrap_or(None) {
                Some(e) => {
                    Some((key.to_string(), e.to_string(), ENV))
                }
                None => {
                    Some((key.to_string(), key.to_string(), MISSING))
                }
            }
        }
    }
}

fn process(file: fs::File,
           used_keys: &mut HashMap<&'static str, Vec<(String, String)>>,
           dict: Option<HashMap<String, ValueAndSource>>,
           props_first: bool)
           -> Vec<String> {
    let buf_reader = io::BufReader::new(&file);
    let mut result: Vec<String> = vec![];

    for line in buf_reader.lines() {
        replace_var(line.unwrap(), &dict, props_first, used_keys, &mut result);
    }

    result
}

fn extract_keys(file: fs::File /* key_map: Option<HashMap<String, ValueAndSource>> */)
                -> HashMap<String, String> {
    let key_regex = regex!(r"^\s*(?P<key>\S*)\s*=\s*(?P<val>.*)");
    let buf_reader = io::BufReader::new(&file);
    let mut keys = HashMap::new();/*,
                                    };*/

    for line in buf_reader.lines().enumerate() {
        for cap in key_regex.captures_iter(&*line.1
                                                 .ok()
                                                 .expect("Could not read line")) {
            let key = cap.name("key")
                         .unwrap_or("???");
            let val = cap.name("val")
                         .unwrap_or("???");
            if !keys.contains_key(key) {
                keys.insert(key.to_string(),
                            // ValueAndSource {
                            // value:
                            val.to_string() /* ,
                                             * source: "".to_string(),
                                             * } */);
            }
        }
    }

    keys
}

#[derive(Debug)]
struct ValueAndSource {
    value: String,
    source: String,
}

fn insert_if_not_exist(dict: &mut HashMap<String, ValueAndSource>,
                       matches: HashMap<String, String>,
                       filename: String) {
    for key_val in matches.iter() {
        let key: String = (*key_val.0).clone();
        let val: String = (*key_val.1).clone();
        if !dict.contains_key(&key) {
            dict.insert(key,
                        ValueAndSource {
                            value: val,
                            source: filename.clone(),
                        });
        }
    }
}

fn read_keyfiles_and_generate_dict(key_filenames: Vec<String>)
                                   -> Option<HashMap<String, ValueAndSource>> {
    let mut dict: HashMap<String, ValueAndSource> = HashMap::new();
    for filename in key_filenames {
        fs::File::open(&filename)
            .and_then(|f| Ok(extract_keys(f)))
            .map(|v| {
                insert_if_not_exist(&mut dict, v, filename);
            });
    }

    match dict.len() {
        0 => None,
        _ => Some(dict),
    }
}


fn create_file(arg_parser: &ApplicationOptions, filename: &String) -> fs::File {
    match fs::File::create(filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("ERROR: Could not create file: {}\n{}\n", filename, e);
            arg_parser.print_usage_and_panic();
            None
        }
    }
    .unwrap()
}

fn open_file(arg_parser: &ApplicationOptions, filename: &String) -> fs::File {
    match fs::File::open(filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("ERROR: Could not open file: {}\n{}\n", filename, e);
            arg_parser.print_usage_and_panic();
            None
        }
    }
    .unwrap()
}

struct Settings<'a> {
    key_filenames: Vec<String>,
    prop_filename: &'a String,
    result_filename: &'a String,
    props_first: bool,
}

fn calc_result(arg_parser: &ApplicationOptions, settings: Settings) {
    let prop_file = open_file(&arg_parser, &settings.prop_filename);
    let mut result_buff = Vec::new();
    let mut used_keys = HashMap::new();

    let keys = read_keyfiles_and_generate_dict(settings.key_filenames);

    for line in process(prop_file, &mut used_keys, keys, settings.props_first).iter() {
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
        // println!("From keyfile {}:", key_filename.unwrap()) ;
        for v in vars {
            println!("${{{}}}", v.0);
        }
        println!("");
    }

    let missing_vars = used_keys.get(MISSING);
    if missing_vars.is_some() {
        let vars = missing_vars.unwrap();
        println!("Missing variables found in {}:", settings.prop_filename);
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

    let mut result_file = create_file(&arg_parser, settings.result_filename);

    for line in result_buff {
        result_file.write_all(&format!("{}\n", line).as_bytes()).unwrap();
    }
    result_file.flush().unwrap();

    println!("...DONE!");
}

fn create_arg_parser() -> ApplicationOptions {
    ApplicationOptions::new(|| {
        let mut o = OptionsAndFlags::new();
        o.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
        o.opt("k",
              "keys",
              "keyfile with variable substitutions",
              "FILE",
              getopts::HasArg::Yes,
              getopts::Occur::Multi)
         .unwrap();
        o.opt("p",
              "props-first",
              "properties takes precedence over environment variables (default: off)",
              "",
              getopts::HasArg::No,
              getopts::Occur::Optional)
         .unwrap();
        o.opt("?",
              "help",
              "print this help menu",
              "",
              getopts::HasArg::No,
              getopts::Occur::Optional)
         .unwrap();
        Ok(o)
    })
}

struct PropAndResultFilenames<'a> {
    prop_filename: &'a String,
    result_filename: &'a String,
}

fn get_prop_and_result_filename<'a>(arg_parser: &'a ApplicationOptions,
                                    free_args: &'a Vec<String>)
                                    -> PropAndResultFilenames<'a> {
    match free_args.len() {
        1 => {
            // Replace vars in the provided file
            PropAndResultFilenames {
                prop_filename: &free_args[0],
                result_filename: &free_args[0],
            }
        }
        2 => {
            // Replace vars into a separate output file
            PropAndResultFilenames {
                prop_filename: &free_args[0],
                result_filename: &free_args[1],
            }
        }
        _ => {
            println_stderr!("Wrong number of file parameters: {}", free_args.len());
            arg_parser.print_usage();
            process::exit(1);
        }
    }
}

fn get_props_first(matches: &Box<getopts::Matches>) -> bool {
    if matches.opt_present("p") {
        true
    } else {
        false
    }

}

fn get_parsed_args(arg_parser: &ApplicationOptions) -> Box<getopts::Matches> {
Box::new(arg_parser.parse().unwrap_or_else(|e| {
        println_stderr!("{}", e);
        process::exit(1);
    }))
}

#[allow(dead_code)]
fn main() {
    let arg_parser: ApplicationOptions = create_arg_parser();
    let matches = get_parsed_args(&arg_parser);

    let prop_and_result_filenames = get_prop_and_result_filename(&arg_parser, 
                                                                 &matches.free);
    calc_result(&arg_parser,
                Settings {
                    key_filenames: matches.opt_strs("k"),
                    prop_filename: prop_and_result_filenames.prop_filename,
                    result_filename: prop_and_result_filenames.result_filename,
                    props_first: get_props_first(&matches),
                });
}


#[cfg(test)]
mod tests {
    use super::replace_var;
    use std::collections::HashMap;
    use super::ValueAndSource;

    struct TestData {
        key_map: HashMap<String, ValueAndSource>,
        used_keys: HashMap<&'static str, Vec<(String, String)>>,
        result: Vec<String>,
    }

    fn create_test_data(keys: Vec<(&str, &str)>) -> TestData {
        let mut test_data = TestData {
            key_map: HashMap::new(),
            used_keys: HashMap::new(),
            result: Vec::new(),
        };

        for k in keys {
            test_data.key_map.insert(k.0.to_string(),
                                     ValueAndSource {
                                         value: k.1.to_string(),
                                         source: "".to_string(),
                                     });
        }

        test_data
    }

    #[test]
    fn can_sub_single_var() {
        let mut test_data = create_test_data(vec![("val", "world!")]);
        replace_var("Hello ${val}".to_string(),
                    &Some(test_data.key_map),
                    true,
                    &mut test_data.used_keys,
                    &mut test_data.result);
        assert!(test_data.result.len() == 1);
        assert!(test_data.used_keys[super::KEY_FILE][0].0 == "val".to_string());
        assert!(test_data.result[0] == "Hello world!".to_string());
    }
}
