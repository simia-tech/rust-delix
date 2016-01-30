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

extern crate hyper;

use std::io::Read;

use self::hyper::client::response::Response;
use self::hyper::status::StatusCode;

pub fn assert_response(expected_status_code: StatusCode,
                       expected_body: &[u8],
                       response: &mut Response) {
    assert_eq!(expected_status_code, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!(String::from_utf8_lossy(expected_body), response_body);
}

pub fn assert_contains_all<T: PartialEq>(expected: &[T], actual: &Vec<T>) {
    for e in expected {
        assert!(actual.contains(e));
    }
}
