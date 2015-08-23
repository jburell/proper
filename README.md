# proper
[![Build Status](https://travis-ci.org/jburell/proper.svg?branch=master)](https://travis-ci.org/jburell/proper)
A simple application to substitute properties with values from environment 
variables or key/value files.

### Basic usage
Usage:
 Alt 1: target/release/proper [OPTIONS] <property-file> <result-file>
 Alt 2: target/release/proper [OPTIONS] <property-file> (will replace vars in property-file)

About:
This tool replaces occurances of ${<VAR>} in the property-file and replaces them either with environment variables or from keyfile (if given). Format in keyfile is: VAR=VALUE.

Options:
    -k --keys FILE      keyfile with variable substitutions
    -p --props-first    properties takes precedence over environment variables
                        (default: off)
    -? --help           print this help menu

