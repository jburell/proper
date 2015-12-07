extern crate getopts;
use getopts::Options;
use std::{env};
use std::collections::{HashSet};
use std::process;

pub struct OptionsAndFlags {
    opts: Options,
    flags: HashSet<String>,
}

pub trait OptionManagement {
    fn new() -> Self;
    fn has_flag(&self, flag: &str) -> bool;
    fn opt(&mut self,
           short_name: &str,
           long_name: &str,
           desc: &str,
           hint: &str,
           hasarg: getopts::HasArg,
           occur: getopts::Occur) -> Result<(), String>;
    fn parse(&self, args: &[String]) -> Result<getopts::Matches, String>;
    fn parsing_style(&mut self, getopts::ParsingStyle);
    fn usage(&self, &str) -> String;
}

impl OptionManagement for OptionsAndFlags {
    fn new() -> Self {
        OptionsAndFlags {
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

    fn parse(&self, args: &[String]) -> Result<getopts::Matches, String> {
        self.opts.parse(args).or_else(|e| Err(e.to_string()))            
    }

    fn parsing_style(&mut self, style: getopts::ParsingStyle) {
        self.opts.parsing_style(style);
    }

    fn usage(&self, s: &str) -> String {
        self.opts.usage(s)
    }
}

pub struct ApplicationOptions {
    application_name: String,
    opts: OptionsAndFlags,
}

pub trait ParseResult {
    fn new<F>(setup_func: F) -> Self
        where F: FnOnce() -> Result<OptionsAndFlags, String>;
    fn print_usage(&self);
    fn print_usage_and_panic(&self);
    fn has_option(&self, opt: &str) -> bool;
    fn parse(&self) -> Result<getopts::Matches, String>;
}

impl ParseResult for ApplicationOptions {
    fn new<F>(setup_func: F) -> Self  
        where F: FnOnce() -> Result<OptionsAndFlags, String> {
        ApplicationOptions {
            application_name: env::args().next().unwrap(),
            opts: setup_func().unwrap(),
        }
    }

    fn print_usage(&self) {
        let brief = 
            format!("\nUsage:\n \
            Alt 1: {0} [OPTIONS] <property-file> <result-file>\n \
            Alt 2: {0} [OPTIONS] <property-file> \
            (will replace vars in property-file)\n\n\
            About:\n\
            This tool replaces occurances of ${{<VAR>}} \
            in the property-file and replaces them either with environment \
            variables or from keyfile (if given). \
            Format in keyfile is: VAR=VALUE.", 
            self.application_name);
        print!("{}", self.opts.usage(&*brief));
    }

    fn print_usage_and_panic(&self) {
        self.print_usage();
        process::exit(1);
    }

    fn has_option(&self, opt: &str) -> bool {
        self.opts.has_flag(opt)
    }

    fn parse(&self) -> Result<getopts::Matches, String> {
        let args: Vec<String> = env::args().collect();        
        let matches: getopts::Matches = (match self.opts.parse(&args[1..]) {
            Ok(m) =>  Ok(m),
            Err(f) => { 
                println!("ERROR: {}", f.to_string());
                self.print_usage_and_panic(); 
                Err(f)
            }
        }).unwrap();

        if matches.opt_present("?") {
           self.print_usage();
            process::exit(0);
        }

        Ok(matches)
    }
}


