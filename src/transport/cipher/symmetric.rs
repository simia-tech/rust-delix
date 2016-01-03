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

use std::iter;

use crypto::aes::KeySize;
use crypto::aes_gcm::AesGcm;
use crypto::aead::{AeadEncryptor, AeadDecryptor};
use protobuf::{self, Message};
use rand::random;

use message;
use transport::cipher::{Cipher, Error, Result};

const NONCE_SIZE: usize = 12;

pub struct Symmetric {
    key_size: KeySize,
    key: Vec<u8>,
    nonce: Option<Vec<u8>>,
}

impl Symmetric {
    pub fn new(key: &[u8], nonce: Option<&[u8]>) -> Result<Symmetric> {
        let key_size = match key.len() {
            16 => KeySize::KeySize128,
            24 => KeySize::KeySize192,
            32 => KeySize::KeySize256,
            _ => return Err(Error::InvalidKeyLength(key.len())),
        };

        Ok(Symmetric {
            key_size: key_size,
            key: key.to_vec(),
            nonce: nonce.map(|nonce| nonce.to_vec()),
        })
    }
}

impl Cipher for Symmetric {
    fn encrypt(&self, plain_text: &[u8]) -> Result<Vec<u8>> {
        let nonce_random = random::<[u8; NONCE_SIZE]>().to_vec();
        let nonce = self.nonce.as_ref().unwrap_or(&nonce_random);

        let mut cipher = AesGcm::new(self.key_size, &self.key, nonce, &[]);
        let mut cipher_text = iter::repeat(0).take(plain_text.len()).collect::<Vec<u8>>();
        let mut tag = iter::repeat(0).take(16).collect::<Vec<u8>>();
        cipher.encrypt(plain_text, &mut cipher_text, &mut tag);

        let mut encrypted = message::Encrypted::new();
        encrypted.set_cipher_type(message::Encrypted_CipherType::AESGCM);
        encrypted.set_cipher_text(cipher_text);
        encrypted.set_nonce(nonce.to_vec());
        encrypted.set_tag(tag);
        encrypted.write_to_bytes().map_err(|_| Error::Write)
    }

    fn decrypt(&self, cipher_text: &[u8]) -> Result<Vec<u8>> {
        let encrypted = match protobuf::parse_from_bytes::<message::Encrypted>(cipher_text) {
            Ok(encrypted) => encrypted,
            Err(_) => return Err(Error::Read),
        };

        let mut cipher = AesGcm::new(self.key_size, &self.key, encrypted.get_nonce(), &[]);
        let mut plain_text = iter::repeat(0)
                                 .take(encrypted.get_cipher_text().len())
                                 .collect::<Vec<u8>>();
        if !cipher.decrypt(encrypted.get_cipher_text(),
                           &mut plain_text,
                           encrypted.get_tag()) {
            return Err(Error::DecryptionFailed);
        }
        Ok(plain_text)
    }
}

#[cfg(test)]
mod tests {

    use rustc_serialize::hex::{FromHex, ToHex};
    use super::Symmetric;
    use super::super::{Cipher, Error};

    struct Set {
        key: Vec<u8>,
        nonce: Vec<u8>,
        plain_text: Vec<u8>,
        cipher_text: Vec<u8>,
    }

    fn sets() -> [Set; 3] {
        [Set {
             key: "000102030405060708090a0b0c0d0e0f".from_hex().ok().unwrap(),
             nonce: "000000000000000000000000".from_hex().ok().unwrap(),
             plain_text: b"test message".to_vec(),
             cipher_text: "0801120c0000000000000000000000001a0c3db3f427b9f6c3ff90e81d0d22102958d0a3\
                           2be787b9c59da25053419e41"
                              .from_hex()
                              .ok()
                              .unwrap(),
         },
         Set {
             key: "000102030405060708090a0b0c0d0e0f0001020304050607".from_hex().ok().unwrap(),
             nonce: "000000000000000000000000".from_hex().ok().unwrap(),
             plain_text: b"test message".to_vec(),
             cipher_text: "0801120c0000000000000000000000001a0c499864dd11b518a12286eacf221076b25875\
                           be43895b643f78087bf38494"
                              .from_hex()
                              .ok()
                              .unwrap(),
         },
         Set {
             key: "000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f"
                      .from_hex()
                      .ok()
                      .unwrap(),
             nonce: "000000000000000000000000".from_hex().ok().unwrap(),
             plain_text: b"test message".to_vec(),
             cipher_text: "0801120c0000000000000000000000001a0c057376ca7d93a3d3d411d1a02210fa7ebca9\
                           7ce635db1901e71a75c6f79c"
                              .from_hex()
                              .ok()
                              .unwrap(),
         }]
    }

    #[test]
    fn encrypt() {
        for set in sets().iter() {
            let cipher = Symmetric::new(&set.key, Some(&set.nonce)).unwrap();
            let cipher_text = cipher.encrypt(&set.plain_text).unwrap();
            assert_eq!(set.cipher_text.to_hex(), cipher_text.to_hex());
        }
    }

    #[test]
    fn decrypt() {
        for set in sets().iter() {
            let cipher = Symmetric::new(&set.key, Some(&set.nonce)).unwrap();
            let plain_text = cipher.decrypt(&set.cipher_text).unwrap();
            assert_eq!(String::from_utf8_lossy(&set.plain_text),
                       String::from_utf8_lossy(&plain_text));
        }
    }

    #[test]
    fn decrypt_failure_on_invalid_key() {
        for set in sets().iter() {
            let cipher = Symmetric::new(b"--invalid  key--", Some(&set.nonce)).unwrap();
            assert_eq!(Err(Error::DecryptionFailed),
                       cipher.decrypt(&set.cipher_text));
        }
    }

}
