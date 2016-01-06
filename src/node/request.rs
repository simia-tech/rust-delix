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

pub type Handler = Fn(&[u8]) -> Response + Send;

pub type Response = Result<Vec<u8>, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    ServiceDoesNotExists,
    ServiceUnavailable,
    Timeout,
    Internal(String),

    // this is used only during tests to simulate a request duration
    TestDuration(Vec<u8>),
}
