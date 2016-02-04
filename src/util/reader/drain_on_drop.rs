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

pub struct DrainOnDrop<R>
    where R: io::Read
{
    reader: R,
    eof_reached: bool,
}

impl<R> DrainOnDrop<R> where R: io::Read
{
    pub fn new(reader: R) -> Self {
        DrainOnDrop {
            reader: reader,
            eof_reached: false,
        }
    }
}

impl<R> io::Read for DrainOnDrop<R> where R: io::Read
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let result = self.reader.read(buffer);
        if let Ok(0) = result {
            self.eof_reached = true;
        }
        result
    }
}

impl<R> Drop for DrainOnDrop<R> where R: io::Read
{
    fn drop(&mut self) {
        if !self.eof_reached {
            let _ = io::copy(&mut self.reader, &mut io::sink());
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io;
    use super::DrainOnDrop;

    #[test]
    fn read() {
        let parent = io::Cursor::new(b"test message".to_vec());
        let mut reader = DrainOnDrop::new(parent);

        let mut output = Vec::new();
        io::copy(&mut reader, &mut output).unwrap();
        assert_eq!("test message", String::from_utf8_lossy(&output));
    }

    #[test]
    fn copy_leftover_to_sink_on_drop() {
        let mut parent = io::Cursor::new(b"test message".to_vec());
        {
            DrainOnDrop::new(&mut parent);
        }

        let mut output = Vec::new();
        io::copy(&mut parent, &mut output).unwrap();
        assert_eq!("", String::from_utf8_lossy(&output));
    }

}
