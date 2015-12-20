// Copyright 2015 The Delix Project Authors. See the AUTHORS file at the top level directory.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::env;
use std::path::Path;
use std::process;
use std::result;

use getopts;

pub struct Arguments {
    pub configuration_path: String,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(getopts::Fail),
}

impl Arguments {
    pub fn parse() -> Result<Arguments> {
        let arguments: Vec<String> = env::args().collect();
        let program = arguments[0].clone();

        let mut options = getopts::Options::new();
        options.optopt("c", "config", "path of the configuration file", "PATH");
        options.optflag("h", "help", "print help");

        let matches = try!(options.parse(&arguments[1..]));

        if matches.opt_present("h") {
            print_usage(&program, options);
            process::exit(1);
        }

        let default_configuration_path = format!("{}.conf.toml",
                                                 Path::new(&program)
                                                     .file_stem()
                                                     .unwrap()
                                                     .to_str()
                                                     .unwrap());
        let configuration_path = if matches.opt_present("c") {
            matches.opt_default("c", &default_configuration_path).unwrap()
        } else {
            default_configuration_path
        };

        Ok(Arguments { configuration_path: configuration_path })
    }
}

impl From<getopts::Fail> for Error {
    fn from(error: getopts::Fail) -> Self {
        Error::ParseError(error)
    }
}

fn print_usage(program: &str, options: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", options.usage(&brief));
}
