# proper [![Build Status](https://travis-ci.org/jburell/proper.svg?branch=master)](https://travis-ci.org/jburell/proper)
*Current version:* _v0.7.1_

A simple application to substitute properties with values from environment 
variables or key/value files.

### Basic usage
```
Usage:
 proper [OPTIONS] <property-file> <result-file>

About:
This tool replaces occurances of ${<VAR>} in the property-file and replaces them
either with environment variables or from keyfile (if given). 
Format in keyfile is: VAR=VALUE.

Options:
    -k --keys FILE(s)   keyfile(s) with variable substitutions, can occur
                        multiple times
    -p --props-first    properties takes precedence over environment variables
                        (default: off)
    -s --shadow-keys    When using multiple keyfiles, key-values will
                        overshadow each other. Default is that multiple values
                        for one key throws an error.
    -? --help           print this help menu
    -V --version        prints current version number
```

### Build instructions
```
$ git clone https://github.com/jburell/proper.git
$ cd proper
$ cargo build --release
```
### Installation instructions (Linux)
```
$ sudo cp target/release/proper /usr/local/bin/
(Or...)
$ sudo ln -s target/release/proper /usr/local/bin/proper
```
