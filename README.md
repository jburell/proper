# proper [![Build Status](https://travis-ci.org/jburell/proper.svg?branch=master)](https://travis-ci.org/jburell/proper)
*Current version:* _v0.6.0_

A simple application to substitute properties with values from environment 
variables or key/value files.

### Basic usage
```
Usage:
 Alt 1: proper [OPTIONS] <property-file> <result-file>
 Alt 2: proper [OPTIONS] <property-file> (will replace vars in property-file)

About:
This tool replaces occurances of ${<VAR>} in the property-file and replaces them either with environment variables or from keyfile (if given). Format in keyfile is: VAR=VALUE.

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
