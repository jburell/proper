extern crate regex;
extern crate getopts;
use getopts::Options;
use std::{env, fs, io};
use std::collections::{HashMap, HashSet};
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

fn print_usage(program: &str, opts: &ApplicationOptions) {
    let brief = format!("Usage: {} [OPTIONS] <property-file> <result-file>", program);
    print!("{}", opts.usage(&*brief));
}

fn print_usage_and_panic(program: &str, opts: &ApplicationOptions) {
    print_usage(program, opts);
    panic!();
}

fn get_free_args(flag_args: HashSet<String>, free_args: Vec<String>) -> Vec<String> {
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


struct ApplicationOptions {
    opts: Options,
    flags: HashSet<String>,
}

trait OptionManagement {
    fn new() -> Self;
    fn has_flag(&self, flag: &str) -> bool;
    fn opt(&mut self,
           short_name: &str,
           long_name: &str,
           desc: &str,
           hint: &str,
           hasarg: getopts::HasArg,
           occur: getopts::Occur) -> Result<(), String>;
   // fn optflag(&mut self, short: &str, long: &str, desc: &str);
    fn parse(&mut self, args: &[String]) -> Result<getopts::Matches, String>;
    fn parsing_style(&mut self, getopts::ParsingStyle);
    fn usage(&self, &str) -> String;
}

impl OptionManagement for ApplicationOptions {
    fn new() -> Self {
        ApplicationOptions {
            opts: Options::new(),
            flags: HashSet::new(),
        }
    }

    fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    fn opt(&mut self,
           short_name: &str,
           long_name: &str,
           desc: &str,
           hint: &str,
           hasarg: getopts::HasArg,
           occur: getopts::Occur) -> Result<(), String> {
        if self.flags.contains(short_name) {
            Err(format!("The flag {} is only allowed once", short_name))
        } else if self.flags.contains(long_name) {
            Err(format!("The flag {} is only allowed once", long_name))
        } else {
            //println!("{:?}", self.flags);
            if short_name.len() > 0 {
                self.flags.insert(format!("-{}", short_name.to_string()));
            }
            if long_name.len() > 0 {
                self.flags.insert(format!("--{}",long_name.to_string()));
            }
            self.opts.opt(short_name,
                long_name,
                desc,
                hint,
                hasarg,
                occur);
            Ok(())
        }
    }

    /*fn optflag(&mut self, short: &str, long: &str, desc: &str) {
        self.opts.optflag(short, long, desc);
    }*/

    fn parse(&mut self, args: &[String]) -> Result<getopts::Matches, String> {
        self.opts.parse(args).or_else(|e| Err(e.to_string()))            
    }

    fn parsing_style(&mut self, style: getopts::ParsingStyle) {
        self.opts.parsing_style(style);
    }

    fn usage(&self, s: &str) -> String {
        self.opts.usage(s)
    }
}

struct ArgOpts {
    application_name: String,
    opts: ApplicationOptions,
}

trait ParseOptions {
    fn new() -> Self;
    fn has_option(&self, opt: &str) -> bool;
    fn parse<F>(&mut self, setup_func: F) -> Result<getopts::Matches, String>
      where F: Fn(&mut ApplicationOptions) -> Result<&mut ApplicationOptions, String>;
    // fn options(self) -> Vec<String>;
}

impl ParseOptions for ArgOpts {
    fn new() -> Self { 
        ArgOpts {
            application_name: env::args().next().unwrap(),
            opts: ApplicationOptions::new(),
        }
    }

    fn has_option(&self, opt: &str) -> bool {
        self.opts.has_flag(opt)
    }

    fn parse<F>(&mut self, setup_func: F) -> Result<getopts::Matches, String> 
        where F: FnOnce(&mut ApplicationOptions) -> Result<&mut ApplicationOptions, String> {
        let opts = setup_func(&mut (self.opts)).unwrap();
        let args: Vec<String> = env::args().collect();        
        let matches: getopts::Matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(f) => { 
                println!("ERROR: {}", f.to_string());
                print_usage(&*self.application_name, &opts); panic!(); 
            }
        };

        if matches.opt_present("?") {
            print_usage(&*self.application_name, &opts);
            std::process::exit(1);
        }

        Ok(matches)
        
    }
}

#[allow(dead_code)]
fn main() {
    let arg_parser: &mut ArgOpts  = &mut ParseOptions::new();

    let matches: getopts::Matches = arg_parser.parse(|o: &mut ApplicationOptions| {
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
    }).unwrap_or_else(|e| {
        panic!(e);
    });

    let mut opt_args: HashSet<String> = HashSet::new(); 

    let key_file: Option<fs::File> = if matches.opt_present("k") {
        let app_name = &*arg_parser.application_name.clone();
        let opts = &arg_parser.opts;
        match matches.opt_str("k") {
            Some(v) => {
                if arg_parser.has_option(&*v) {
                    println!("ERROR: Expected key-filename, found: {}", &*v);
                    print_usage_and_panic(app_name, opts);
                    None
                } else {
                    //println!("key_filename {}", v);
                    opt_args.insert(v.clone());
                    fs::File::open(&v).ok()
                }
            },
            None => { 
                println!("ERROR: Expected file parameter after 'keys' parameter");
                print_usage_and_panic(app_name, opts);
                None
            }
        }
    } else {
        None
    };
    
    let free_args = get_free_args(opt_args, matches.free);

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
            print_usage(&*arg_parser.application_name, &arg_parser.opts);
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
