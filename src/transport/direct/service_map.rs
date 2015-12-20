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

use std::collections::HashMap;
use std::result;

use node::{ID, ServiceHandler};

pub struct ServiceMap {
    map: HashMap<String, Vec<Link>>,
}

enum Link {
    Local(Box<ServiceHandler>),
    Remote(ID),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceAlreadyExists,
}

impl ServiceMap {
    pub fn new() -> ServiceMap {
        ServiceMap { map: HashMap::new() }
    }

    pub fn insert_local(&mut self, name: &str, f: Box<ServiceHandler>) -> Result<()> {
        let mut links = self.get_or_add_links(name);
        if let Some(_) = links.iter().find(local_link) {
            return Err(Error::ServiceAlreadyExists);
        }

        links.push(Link::Local(f));
        Ok(())
    }

    pub fn insert_remote(&mut self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut links = self.get_or_add_links(name);
        if let Some(_) = links.iter().find(|link| {
            match **link {
                Link::Local(_) => false,
                Link::Remote(id) => id == peer_node_id,
            }
        }) {
            return Err(Error::ServiceAlreadyExists);
        }

        links.push(Link::Remote(peer_node_id));
        Ok(())
    }

    pub fn local_service_names(&self) -> Vec<&str> {
        self.map
            .iter()
            .filter_map(|(name, links)| links.iter().find(local_link).and(Some(name.as_ref())))
            .collect()
    }

    pub fn remote_service_names(&self) -> Vec<&str> {
        self.map
            .iter()
            .filter_map(|(name, links)| links.iter().find(remote_link).and(Some(name.as_ref())))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn get_or_add_links(&mut self, name: &str) -> &mut Vec<Link> {
        if !self.map.contains_key(name) {
            self.map.insert(name.to_string(), Vec::new());
        }
        self.map.get_mut(name).unwrap()
    }
}

fn local_link(link: &&Link) -> bool {
    match **link {
        Link::Local(_) => true,
        Link::Remote(_) => false,
    }
}

fn remote_link(link: &&Link) -> bool {
    match **link {
        Link::Local(_) => false,
        Link::Remote(_) => true,
    }
}

#[cfg(test)]
mod tests {

    use super::ServiceMap;
    use node::ID;

    #[test]
    fn insert_local() {
        let mut service_map = ServiceMap::new();

        assert!(service_map.insert_local("test", Box::new(|request| request)).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| request)).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
        assert_eq!(vec!["test"], service_map.remote_service_names());
    }

    #[test]
    fn insert_remote() {
        let node_id = ID::new_random();
        let mut service_map = ServiceMap::new();

        assert!(service_map.insert_remote("test", node_id).is_ok());
        assert!(service_map.insert_remote("test", node_id).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| request)).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
        assert_eq!(vec!["test"], service_map.remote_service_names());
    }

}
