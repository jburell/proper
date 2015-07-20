use std::env;

#[test]
fn read_file(){
    for argument in env::args() {
        println!("{}", argument);
    }
}
