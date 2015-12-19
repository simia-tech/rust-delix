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

extern crate delix;

extern crate getopts;
extern crate toml;

mod arguments;
mod configuration;
mod loader;

use std::thread::sleep_ms;

use arguments::Arguments;
use configuration::Configuration;
use loader::Loader;

#[cfg(not(test))]
fn main() {
    let arguments = match Arguments::parse() {
        Ok(arguments) => arguments,
        Err(err) => {
            println!("error while parsing arguments: {:?}", err);
            return;
        },
    };

    let configuration = match Configuration::read_file(&arguments.configuration_path) {
        Ok(configuration) => configuration,
        Err(err) => {
            println!("error while reading configuration: {:?}", err);
            return;
        },
    };

    let node = match Loader::load_node(&configuration) {
        Ok(node) => node,
        Err(err) => {
            println!("error while loading node: {:?}", err);
            return;
        }
    };

    println!("delix node {} loaded", node.id());

    loop {
        println!("state {:?}", node.state());
        sleep_ms(2000);
    }
}
