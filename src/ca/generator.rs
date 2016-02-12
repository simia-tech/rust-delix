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

use std::fs;
use std::io;
use std::process;
use std::result;

use clap;
use openssl::{crypto, ssl, x509};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ssl(ssl::error::SslError),
    Io(io::Error),
}

pub struct CertificateAuthority {
    bits: u32,
    days: Option<u32>,
    cert_file_name: String,
    key_file_name: String,
}

impl CertificateAuthority {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        CertificateAuthority {
            bits: matches.value_of("bits")
                         .and_then(|value| value.parse::<u32>().ok())
                         .unwrap_or(2048),
            days: matches.value_of("days").and_then(|value| value.parse::<u32>().ok()),
            cert_file_name: matches.value_of("cert").unwrap_or("ca.crt").to_string(),
            key_file_name: matches.value_of("key").unwrap_or("ca.key").to_string(),
        }
    }

    pub fn generate(&self) -> Result<()> {
        let mut generator = x509::X509Generator::new()
                                .set_bitlength(self.bits)
                                .set_sign_hash(crypto::hash::Type::SHA256);
        if let Some(days) = self.days {
            generator = generator.set_valid_period(days)
        }

        let (certificate, private_key) = try!(generator.generate());

        if self.cert_file_name == "-" {
            try!(certificate.write_pem(&mut io::stdout()));
        } else {
            let mut file = try!(fs::File::create(&self.cert_file_name));
            try!(certificate.write_pem(&mut file));
        }

        if self.key_file_name == "-" {
            try!(private_key.write_pem(&mut io::stdout()));
        } else {
            let mut file = try!(fs::File::create(&self.key_file_name));
            try!(private_key.write_pem(&mut file));
        }

        Ok(())
    }
}

pub struct Certificate {
    ca_cert_file_name: String,
    ca_key_file_name: String,
    bits: u32,
    days: Option<u32>,
    cert_file_name: String,
    key_file_name: String,
}

impl Certificate {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let name = matches.value_of("name").unwrap();
        let default_cert_file_name = format!("{}.crt", name);
        let default_key_file_name = format!("{}.key", name);
        Certificate {
            ca_cert_file_name: matches.value_of("ca-cert").unwrap_or("ca.crt").to_string(),
            ca_key_file_name: matches.value_of("ca-key").unwrap_or("ca.key").to_string(),
            bits: matches.value_of("bits")
                         .and_then(|value| value.parse::<u32>().ok())
                         .unwrap_or(2048),
            days: matches.value_of("days").and_then(|value| value.parse::<u32>().ok()),
            cert_file_name: matches.value_of("cert").unwrap_or(&default_cert_file_name).to_string(),
            key_file_name: matches.value_of("key").unwrap_or(&default_key_file_name).to_string(),
        }
    }

    pub fn generate(&self) -> Result<()> {
        let mut generator = x509::X509Generator::new().set_bitlength(self.bits);
        if let Some(days) = self.days {
            generator = generator.set_valid_period(days)
        }

        let mut private_key = crypto::pkey::PKey::new();
        private_key.gen(self.bits as usize);
        let request = try!(generator.request(&private_key));

        let mut command = try!(process::Command::new("openssl")
                                   .arg("x509")
                                   .arg("-req")
                                   .arg("-CA")
                                   .arg(&self.ca_cert_file_name)
                                   .arg("-CAkey")
                                   .arg(&self.ca_key_file_name)
                                   .arg("-CAcreateserial")
                                   .stdin(process::Stdio::piped())
                                   .stdout(process::Stdio::piped())
                                   .stderr(process::Stdio::null())
                                   .spawn());

        try!(request.write_pem(&mut command.stdin.as_mut().unwrap()));

        let output = try!(command.wait_with_output());
        assert!(output.status.success());
        let certificate = try!(x509::X509::from_pem(&mut io::Cursor::new(output.stdout)));

        if self.cert_file_name == "-" {
            try!(certificate.write_pem(&mut io::stdout()));
        } else {
            let mut file = try!(fs::File::create(&self.cert_file_name));
            try!(certificate.write_pem(&mut file));
        }

        if self.key_file_name == "-" {
            try!(private_key.write_pem(&mut io::stdout()));
        } else {
            let mut file = try!(fs::File::create(&self.key_file_name));
            try!(private_key.write_pem(&mut file));
        }

        Ok(())
    }
}

impl From<ssl::error::SslError> for Error {
    fn from(error: ssl::error::SslError) -> Self {
        Error::Ssl(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
