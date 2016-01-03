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

use std::fs::File;
use std::io;
use std::io::Read;
use std::result;

use toml;
use rustc_serialize::hex::FromHex;

#[derive(Debug)]
pub struct Configuration {
    root: toml::Value,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    TOMLParserError(Vec<toml::ParserError>),
}

impl Configuration {
    pub fn read_file(path: &str) -> Result<Configuration> {
        let mut configuration_file = try!(File::open(path));
        let mut configuration = String::new();
        try!(configuration_file.read_to_string(&mut configuration));

        let mut parser = toml::Parser::new(&configuration);
        let value = match parser.parse() {
            Some(value) => toml::Value::Table(value),
            None => {
                return Err(Error::TOMLParserError(parser.errors));
            }
        };

        Ok(Configuration { root: value })
    }

    pub fn i64_at(&self, path: &str) -> Option<i64> {
        self.root.lookup(path).and_then(|value| value.as_integer())
    }

    pub fn string_at(&self, path: &str) -> Option<String> {
        self.root.lookup(path).and_then(|value| value.as_str()).map(|value| value.to_string())
    }

    pub fn strings_at(&self, path: &str) -> Option<Vec<String>> {
        self.root
            .lookup(path)
            .and_then(|value| value.as_slice())
            .map(|values| {
                values.to_vec()
                      .iter()
                      .map(|value| value.as_str().unwrap().to_string())
                      .collect::<Vec<String>>()
            })
    }

    pub fn bytes_at(&self, path: &str) -> Option<Vec<u8>> {
        self.string_at(path).and_then(|value| value.from_hex().ok())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError(error)
    }
}
