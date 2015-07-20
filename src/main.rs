use std::{env, fs, io};
use std::io::BufRead;
//use std::str::pattern::Pattern;
//use std::str::pattern::Searcher;

fn find_variable(line: String) -> Vec<usize> {
    let mut matches = vec!();
//    let mut searcher = "${".into_searcher(line.as_str());
//    for hit in searcher.next_match() {
//        matches.push(hit.0);
//        //println!("{}", hit.0);
//    }
    matches.push(1);   
    matches
}

fn process(file: fs::File){
    let bufReader = io::BufReader::new(&file);

    println!("Skriver ut: ");
    for line in bufReader.lines() {
        println!("{:?}", find_variable(line.unwrap()));
    }
}

#[allow(dead_code)]
fn main() {
    let mut args = env::args();

    let propFile: fs::File = fs::File::open(args.nth(1).unwrap()).unwrap();
    process(propFile);
}
