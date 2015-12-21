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

use std::fmt;
use std::str::FromStr;
use rand::random;
use rustc_serialize::hex::{FromHex, FromHexError, ToHex};

const ID_BITS: usize = 40;
const ID_BYTES: usize = ID_BITS / 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ID {
    value: [u8; ID_BYTES],
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidLength(usize),
    FromHexError(FromHexError),
}

impl ID {
    pub fn new(value: Vec<u8>) -> Result<ID> {
        if value.len() != ID_BYTES {
            return Err(Error::InvalidLength(value.len()));
        }
        let mut id = ID { value: [0; ID_BYTES] };
        for index in 0..ID_BYTES {
            id.value[index] = value[index];
        }
        Ok(id)
    }

    pub fn new_random() -> ID {
        ID { value: random::<[u8; ID_BYTES]>() }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for item in self.value.iter() {
            result.push(*item);
        }
        result
    }
}

impl FromStr for ID {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let bytes = try!(s.from_hex());
        ID::new(bytes)
    }
}

impl ToHex for ID {
    fn to_hex(&self) -> String {
        self.value.to_hex()
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Self {
        Error::FromHexError(error)
    }
}

#[cfg(test)]
mod tests {

    use super::ID;
    use rustc_serialize::hex::ToHex;

    #[test]
    fn test_random_id() {
        let id_one = ID::new_random();
        let id_two = ID::new_random();
        assert!(id_one != id_two);
    }

    #[test]
    fn test_hex_coding() {
        let id = "56789abcde".parse::<ID>().unwrap();
        assert_eq!("56789abcde", id.to_hex());

        assert!("a".parse::<ID>().is_err());
        assert!("56789abcdX".parse::<ID>().is_err());
    }

}
