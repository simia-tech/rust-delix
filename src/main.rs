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

#[cfg(not(test))]
extern crate delix;

#[cfg(not(test))]
extern crate getopts;
#[cfg(not(test))]
#[macro_use]
extern crate log;
#[cfg(not(test))]
extern crate time;
#[cfg(not(test))]
extern crate toml;
#[cfg(not(test))]
extern crate rustc_serialize;

#[cfg(not(test))]
mod arguments;
#[cfg(not(test))]
mod configuration;
#[cfg(not(test))]
mod loader;

#[cfg(not(test))]
fn main() {
    let arguments = match ::arguments::Arguments::parse() {
        Ok(arguments) => arguments,
        Err(error) => {
            error!("error while parsing arguments: {:?}", error);
            return;
        },
    };

    let configuration = match ::configuration::Configuration::read_file(&arguments.configuration_path) {
        Ok(configuration) => configuration,
        Err(error) => {
            error!("error while reading configuration: {:?}", error);
            return;
        },
    };

    let loader = ::loader::Loader::new(configuration);

    if let Err(error) = loader.load_log() {
        error!("error while loading log: {:?}", error);
        return;
    }

    let node = match loader.load_node() {
        Ok(node) => node,
        Err(error) => {
            error!("error while loading node: {:?}", error);
            return;
        }
    };

    let relays = match loader.load_relays(&node) {
        Ok(relays) => relays,
        Err(error) => {
            error!("error while loading relays: {:?}", error);
            return;
        }
    };

    loop {
        info!("state {}", node);
        ::std::thread::sleep_ms(5000);
    }
}
