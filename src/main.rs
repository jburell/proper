extern crate regex;
extern crate getopts;
#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::{env, fs, io};
use std::io::prelude::*;
mod options;
use options::*;
use std::process;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::{Entry};

macro_rules! regex { ($re:expr) => { ::regex::Regex::new($re).unwrap() } }
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        )
    );


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ENV: &'static str = "ENV";
const MISSING: &'static str = "Missing properties";

fn replace_var(line: String,
               keys: &Option<KeysAndSources>,
               props_first: bool,
               used_keys: &mut HashMap<String, Vec<(String, String)>>,
               result: &mut Vec<String>) {
    let re = regex!(r"(?P<full>\$\{\s*(?P<var>[^\}]*)\s*\})");
    let mut splits: Vec<&str> = line.split("#").collect();
    let mut str_line: String = splits[0].to_string();

    for cap in re.captures_iter(&*str_line.clone()) {
        cap.name("var")
           .map(|v| {
               let env_prop = env_or_prop(v.as_str().trim(), props_first, keys)
                                  .map(|v2| {
                                      str_line = re.replace(&*str_line,
                                                            &*(v2.1.value)).to_string();
                                      (v2.0, v2.1)
                                  });

               env_prop.and_then(|v2| {
                   used_keys.get_mut(&v2.1.source)
                            .and_then(|r| {
                                Some(r.push((v2.0.clone(), v2.1.value.clone())))
                            })
                            .or_else(|| {
                                used_keys.insert(v2.1.source.clone(),
                                                 vec![(v2.0.clone(),
                                                       v2.1.value.clone())]);
                                Some(())
                            })
               })
           });
    }

    let mut res_line = str_line.to_string();
    for s in splits.split_off(1) {
        res_line = res_line + "#" + s;
    }

    result.push(res_line);
}

fn no_property(key: &str,
               env: Option<String>)
               -> Option<(String, ValueAndSource)> {
    match env {
        Some(e) => {
            Some((key.to_string(),
                  ValueAndSource {
                value: e.to_string(),
                source: ENV.to_string(),
            }))
        }
        None => {
            Some((key.to_string(),
                  ValueAndSource {
                value: key.to_string(),
                source: MISSING.to_string(),
            }))
        }
    }
}

fn env_or_prop(key: &str,
               props_first: bool,
               keys: &Option<KeysAndSources>)
               -> Option<(String, ValueAndSource)> {
    match keys {
        &Some(ref k) => {
            let dict = &(k.dictionary);
            let prop = dict.get(key).and_then(|v| Some(v.clone()));
            let env = env::var_os(key)
                          .map(|v2| v2.clone().into_string().ok().unwrap());
            match (prop, props_first) {
                (Some(prop), true) => {
                    Some((key.to_string(),
                          ValueAndSource {
                        value: prop.value.clone(),
                        source: prop.source.clone(),
                    }))
                }
                (Some(prop), false) => {
                    match env {
                        Some(e) => {
                            println!("[WARNING]: key \"${{{}}}\" from \"{}\" \
                                      was overridden by an environment \
                                      variable.",
                                     key.to_string(),
                                     prop.source);
                            Some((key.to_string(),
                                  ValueAndSource {
                                value: e.to_string(),
                                source: ENV.to_string(),
                            }))
                        }
                        None => {
                            Some((key.to_string(),
                                  ValueAndSource {
                                value: prop.value.clone(),
                                source: prop.source.clone(),
                            }))
                        }
                    }
                }
                (None, true) => no_property(key, env),
                (None, false) => no_property(key, env),
            }
        }
        &None => {
            match env::var_os(key)
                      .map(|v| v.into_string().ok())
                      .unwrap_or(None) {
                Some(e) => {
                    Some((key.to_string(),
                          ValueAndSource {
                        value: e.to_string(),
                        source: ENV.to_string(),
                    }))
                }
                None => {
                    Some((key.to_string(),
                          ValueAndSource {
                        value: key.to_string(),
                        source: MISSING.to_string(),
                    }))
                }
            }
        }
    }
}

fn process(file: fs::File,
           used_keys: &mut HashMap<String, Vec<(String, String)>>,
           dict: &Option<KeysAndSources>,
           props_first: bool)
           -> Vec<String> {
    let buf_reader = io::BufReader::new(&file);
    let mut result: Vec<String> = vec![];

    for line in buf_reader.lines() {
        replace_var(line.unwrap(), &dict, props_first, used_keys, &mut result);
    }

    result
}

fn extract_keys(file: fs::File) -> HashMap<String, String> {
    let key_regex = regex!(r"^\s*(?P<key>\S*)\s*=\s*(?P<val>.*)");
    let buf_reader = io::BufReader::new(&file);
    let mut keys = HashMap::new();

    for line in buf_reader.lines().enumerate() {
        let curr_line: String = line.1.unwrap();
        let splits: Vec<&str> = curr_line.split("#").collect();
        for cap in key_regex.captures_iter(splits[0]) {
            let key = cap.name("key").map_or("???", |m| m.as_str());
            let val = cap.name("val").map_or("???", |m| m.as_str());
            if !keys.contains_key(key) {
                keys.insert(key.to_string(), val.to_string());
            }
        }
    }

    keys
}

#[derive(Debug)]
#[derive(Clone)]
struct ValueAndSource {
    value: String,
    source: String,
}

fn insert_key(entry: Entry<String, ValueAndSource>,
              keys_from_files: &mut HashMap<String, Vec<String>>,
              val: String,
              source: String,
              key: String) {
    match entry {
        Vacant(e) => {
            e.insert(ValueAndSource {
                value: val.clone(),
                source: source.clone(),
            });
            match keys_from_files.entry(source) {
                Vacant(k) => {
                    k.insert(vec![key]);
                }
                Occupied(mut k) => {
                    k.get_mut().push(key);
                }
            }
        },
        Occupied(mut e) => {
            e.insert(ValueAndSource {
                value: val.clone(),
                source: source.clone(),
            });
            match keys_from_files.entry(source) {
                Vacant(k) => {
                    k.insert(vec![key]);
                }
                Occupied(mut k) => {
                    k.get_mut().push(key);
                }
            }
        }
    }
}

fn insert_if_not_exist(dict: &mut HashMap<String, ValueAndSource>,
                       keys_from_files: &mut HashMap<String, Vec<String>>,
                       matches: HashMap<String, String>,
                       filename: String,
                       shadow: bool) {
    for key_val in matches.iter() {
        let key: String = (*key_val.0).clone();
        let val: String = (*key_val.1).clone();
        match dict.entry(key.clone()) { 
            Vacant(e) => {
                insert_key(Vacant(e),
                           keys_from_files,
                           val,
                           filename.clone(),
                           key.clone());
                // keys_from_files.insert(filename.clone(), key.clone());
            },
            Occupied(e) => {
                if shadow {
                    println!("[WARN]: key ({}) in file ({}) overrides the \
                              same key from file ({})",
                             key,
                             filename,
                             e.get().source);
                    insert_key(Occupied(e),
                               keys_from_files,
                               val,
                               filename.clone(),
                               key.clone());
                } else {
                    println!("[ERROR]: key ({}) in file ({}) tried to \
                              override the same key from file ({})",
                             key,
                             filename,
                             e.get().source);
                    process::exit(1);
                }
            }
        }
    }
}

fn read_keyfiles_to_dict(key_filenames: Vec<String>,
                         shadow: bool)
                         -> Option<KeysAndSources> {
    let mut dict: HashMap<String, ValueAndSource> = HashMap::new();
    let mut keys_from_files = HashMap::new();
    for filename in key_filenames {
        fs::File::open(&filename)
            .and_then(|f| Ok(extract_keys(f)))
            .map(|v| {
                insert_if_not_exist(&mut dict,
                                    &mut keys_from_files,
                                    v,
                                    filename,
                                    shadow);
            })
            .unwrap();
    }

    match dict.len() {
        0 => None,
        _ => {
            Some(KeysAndSources {
                dictionary: Box::new(dict), /* sources: Box::new(keys_from_files), */
            })
        }
    }
}


fn create_file(filename: &String) -> fs::File {
    match fs::File::create(filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("[ERROR]: Could not create file: {}\n{}\n",
                            filename,
                            e);
            APP.print_usage_and_panic();
            None
        }
    }
    .unwrap()
}

fn open_file(filename: &String) -> fs::File {
    match fs::File::open(filename) {
        Ok(f) => Some(f),
        Err(e) => {
            println_stderr!("[ERROR]: Could not open file: {}\n{}\n",
                            filename,
                            e);
            APP.print_usage_and_panic();
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
    shadow: bool,
}

struct KeysAndSources {
    dictionary: Box<HashMap<String, ValueAndSource>>, /* sources: Box<HashMap<String, Vec<String>>>, */
}

fn print_result(used_keys: &mut HashMap<String, Vec<(String, String)>>,
                prop_filename: &String) {

    println!("======== RESULTS ========");

    let env_vars = &used_keys.remove(ENV);
    if env_vars.is_some() {
        let vars = env_vars.clone().unwrap();
        println!("# Environment variables:");
        for v in vars {
            println!("${{{}}}", v.0);
        }
        println!("");
    }

    let missing_vars = used_keys.remove(MISSING);

    for src in used_keys.iter() {
        println!("# {}:", src.0);
        let mut keys_src = src.1.clone();
        keys_src.sort();
        for s in keys_src {
            println!("${{{}}}", s.0);
        }
        println!("");
    }

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

    if env_vars.is_none() && used_keys.len() == 0 && missing_vars.is_none() {
        println!("[No variables to substitute!]\n");
    }

    println!("...DONE!");
}

fn calc_result(settings: Settings) {
    let prop_file = open_file(&settings.prop_filename);
    let mut result_buff = Vec::new();
    let mut used_keys = HashMap::new();
    let keys_and_sources = read_keyfiles_to_dict(settings.key_filenames,
                                                 settings.shadow);

    let processed_buffer = process(prop_file,
                                   &mut used_keys,
                                   &keys_and_sources,
                                   settings.props_first);

    for line in processed_buffer.iter() {
        result_buff.push(line.clone());
    }


    let mut result_file = create_file(settings.result_filename);

    for line in result_buff {
        result_file.write_all(&format!("{}\n", line).as_bytes()).unwrap();
    }
    result_file.flush().unwrap();

    print_result(&mut used_keys, settings.prop_filename);
}

fn get_arg_parser() -> ApplicationOptions {
    ApplicationOptions::new(|| {
        let mut o = OptionsAndFlags::new();
        o.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
        o.opt("k",
              "keys",
              "keyfile(s) with variable substitutions, can occur multiple \
               times",
              "FILE(s)",
              getopts::HasArg::Yes,
              getopts::Occur::Multi)
         .unwrap();
        o.opt("p",
              "props-first",
              "properties takes precedence over environment variables \
               (default: off)",
              "",
              getopts::HasArg::No,
              getopts::Occur::Optional)
         .unwrap();
        o.opt("s",
              "shadow-keys",
              "When using multiple keyfiles, key-values will overshadow each \
               other. Default is that multiple values for one key throws an \
               error.",
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
        o.opt("V",
              "version",
              "prints current version number",
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

fn get_prop_and_result_filename<'a>(free_args: &'a Vec<String>)
                                    -> PropAndResultFilenames<'a> {
    match free_args.len() {
        /*1 => {
            // Replace vars in the provided file
            PropAndResultFilenames {
                prop_filename: &free_args[0],
                result_filename: &free_args[0],
            }
        }*/
        2 => {
            // Replace vars into a separate output file
            PropAndResultFilenames {
                prop_filename: &free_args[0],
                result_filename: &free_args[1],
            }
        }
        _ => {
            println_stderr!("ERROR: Wrong number of file parameters: {}, expected: {}",
                            free_args.len(), 2);
            APP.print_usage();
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

fn get_parsed_args() -> Box<getopts::Matches> {
    Box::new(APP.parse().unwrap_or_else(|e| {
        println_stderr!("{}", e);
        process::exit(1);
    }))
}

// Singletons
lazy_static! {
    static ref APP: ApplicationOptions = get_arg_parser();
}

#[allow(dead_code)]
fn main() {
    let matches = get_parsed_args();

    if matches.opt_present("V") {
        println!("proper: v{}", VERSION);
        process::exit(0);
    }

    let prop_and_result_filenames = get_prop_and_result_filename(&matches.free);
    calc_result(Settings {
        key_filenames: matches.opt_strs("k"),
        prop_filename: prop_and_result_filenames.prop_filename,
        result_filename: prop_and_result_filenames.result_filename,
        props_first: get_props_first(&matches),
        shadow: matches.opt_present("s"),
    });
}


#[cfg(test)]
mod tests {
    use super::replace_var;
    use std::collections::HashMap;
    use super::ValueAndSource;
    use super::KeysAndSources;

    struct TestData {
        key_map: KeysAndSources,
        used_keys: HashMap<String, Vec<(String, String)>>,
        result: Vec<String>,
    }

    fn create_test_data(keys: Vec<(&str, &str)>) -> TestData {
        let mut test_data = TestData {
            key_map: KeysAndSources {
                dictionary: Box::new(HashMap::new()), /* sources: Box::new(HashMap::new()), */
            },
            used_keys: HashMap::new(),
            result: Vec::new(),
        };

        for k in keys {
            test_data.key_map.dictionary.insert(k.0.to_string(),
                                                ValueAndSource {
                                                    value: k.1.to_string(),
                                                    source: "source_file"
                                                                .to_string(),
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
        assert!(test_data.used_keys["source_file"][0].0 == "val".to_string());
        assert!(test_data.result[0] == "Hello world!".to_string());
    }
}
