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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Link {
    Local,
    Remote(ID),
}

impl Link {
    pub fn is_local(link: &Link) -> bool {
        match *link {
            Link::Local => true,
            _ => false,
        }
    }

    pub fn is_remote(link: &Link, peer_node_id: &ID) -> bool {
        match *link {
            Link::Remote(ref id) if id == peer_node_id => true,
            _ => false,
        }
    }
}
