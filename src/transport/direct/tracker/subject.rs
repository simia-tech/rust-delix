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

use node::ID;

use transport::direct::Link;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Subject {
    Local(String),
    Remote(String, ID),
}

impl Subject {
    pub fn from_name_and_link(name: &str, link: &Link) -> Subject {
        match *link {
            Link::Local => Self::local(name),
            Link::Remote(ref peer_node_id) => Self::remote(name, peer_node_id.clone()),
        }
    }

    pub fn local(name: &str) -> Subject {
        Subject::Local(name.to_string())
    }

    pub fn remote(name: &str, id: ID) -> Subject {
        Subject::Remote(name.to_string(), id)
    }
}
