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

use std::io;

use byteorder::{self, ReadBytesExt};

pub fn read_size<R>(reader: &mut R) -> io::Result<usize>
    where R: io::Read
{
    match reader.read_u64::<byteorder::BigEndian>() {
        Ok(size) => Ok(size as usize),
        Err(byteorder::Error::Io(ref error)) if error.kind() == io::ErrorKind::Other &&
                                                format!("{}", error) == "unexpected EOF" => {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF"));
        }
        Err(byteorder::Error::Io(error)) => return Err(error),
        Err(byteorder::Error::UnexpectedEOF) => {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF"));
        }
    }
}
