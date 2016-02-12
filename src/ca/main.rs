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

extern crate clap;
extern crate openssl;

mod generator;

use std::io::{self, Write};

fn main() {
    let app = clap::App::new("delix certificate management")
                  .version("0.1")
                  .author("Philipp Brüll <pb@simia.tech>")
                  .about("easy x509 certificate generation")
                  .subcommand(clap::SubCommand::with_name("generate-ca")
                                  .about("generates a new certificate authority")
                                  .arg(clap::Arg::with_name("bits")
                                           .help("key length")
                                           .short("-b")
                                           .long("--bits")
                                           .takes_value(true)
                                           .possible_values(&["1024", "2048", "4096"]))
                                  .arg(clap::Arg::with_name("days")
                                           .help("how many days the ca is valid")
                                           .short("-d")
                                           .long("--days")
                                           .takes_value(true))
                                  .arg(clap::Arg::with_name("cert")
                                           .help("certificate file name - use '-' for stdout")
                                           .short("-c")
                                           .long("--cert")
                                           .takes_value(true)
                                           .value_name("FILE"))
                                  .arg(clap::Arg::with_name("key")
                                           .help("private key file name - use '-' for stdout")
                                           .short("-k")
                                           .long("--key")
                                           .takes_value(true)
                                           .value_name("FILE")))
                  .subcommand(clap::SubCommand::with_name("generate")
                                  .about("generates a new certificate")
                                  .arg(clap::Arg::with_name("ca-cert")
                                           .help("ca certificate file name")
                                           .long("--ca-cert")
                                           .takes_value(true)
                                           .value_name("FILE"))
                                  .arg(clap::Arg::with_name("ca-key")
                                           .help("ca private key file name")
                                           .long("--ca-key")
                                           .takes_value(true)
                                           .value_name("FILE"))
                                  .arg(clap::Arg::with_name("bits")
                                           .help("key length")
                                           .short("-b")
                                           .long("--bits")
                                           .takes_value(true)
                                           .possible_values(&["1024", "2048", "4096"]))
                                  .arg(clap::Arg::with_name("days")
                                           .help("how many days the ca is valid")
                                           .short("-d")
                                           .long("--days")
                                           .takes_value(true))
                                  .arg(clap::Arg::with_name("cert")
                                           .help("certificate file name - use '-' for stdout")
                                           .short("-c")
                                           .long("--cert")
                                           .takes_value(true)
                                           .value_name("FILE"))
                                  .arg(clap::Arg::with_name("key")
                                           .help("private key file name - use '-' for stdout")
                                           .short("-k")
                                           .long("--key")
                                           .takes_value(true)
                                           .value_name("FILE"))
                                  .arg(clap::Arg::with_name("name").required(true)))
                  .get_matches();

    let result = match app.subcommand() {
        ("generate-ca", Some(matches)) => generator::CertificateAuthority::new(matches).generate(),
        ("generate", Some(matches)) => generator::Certificate::new(matches).generate(),
        (_, _) => {
            println!("{}", app.usage());
            Ok(())
        }
    };

    if let Err(error) = result {
        write!(io::stderr(), "error: {:?}\n", error).unwrap();
    }
}
