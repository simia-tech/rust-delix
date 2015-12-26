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

pub enum Link {
    Local(Box<ServiceHandler>),
    Remote(ID),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceAlreadyExists,
    ServiceDoesNotExists,
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

    pub fn get_link(&self, name: &str) -> Option<&Link> {
        self.map.get(name).and_then(|links| links.first())
    }

    pub fn local_service_names(&self) -> Vec<&str> {
        self.map
            .iter()
            .filter_map(|(name, links)| links.iter().find(local_link).and(Some(name.as_ref())))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn remove_local(&mut self, name: &str) -> Result<()> {
        let remove = {
            let mut links = match self.map.get_mut(name) {
                Some(links) => links,
                None => return Err(Error::ServiceDoesNotExists),
            };
            links.retain(|link| {
                match *link {
                    Link::Local(_) => false,
                    _ => true,
                }
            });
            links.len() == 0
        };
        if remove {
            self.map.remove(name);
        }
        Ok(())
    }

    pub fn remove_remote(&mut self, peer_node_id: &ID) -> Result<()> {
        let mut names = Vec::new();
        for (name, links) in self.map.iter_mut() {
            links.retain(|link| {
                match *link {
                    Link::Remote(ref node_id) if node_id == peer_node_id => false,
                    _ => true,
                }
            });
            if links.len() == 0 {
                names.push(name.to_string());
            }
        }
        for name in names {
            self.map.remove(&name);
        }
        Ok(())
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

unsafe impl Send for ServiceMap {}

unsafe impl Sync for ServiceMap {}

#[cfg(test)]
mod tests {

    use super::ServiceMap;
    use super::Link;
    use node::ID;

    #[test]
    fn insert_local() {
        let mut service_map = ServiceMap::new();

        assert!(service_map.insert_local("test", Box::new(|request| request.to_vec())).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| request.to_vec())).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn insert_remote() {
        let node_id = ID::new_random();
        let mut service_map = ServiceMap::new();

        assert!(service_map.insert_remote("test", node_id).is_ok());
        assert!(service_map.insert_remote("test", node_id).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| request.to_vec())).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn remove_local() {
        let mut service_map = ServiceMap::new();
        service_map.insert_local("test", Box::new(|request| request.to_vec())).unwrap();
        service_map.insert_remote("test", ID::new_random()).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_local_and_clean_up() {
        let mut service_map = ServiceMap::new();
        service_map.insert_local("test", Box::new(|request| request.to_vec())).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn remove_remote() {
        let node_id = ID::new_random();
        let mut service_map = ServiceMap::new();
        service_map.insert_remote("test", node_id).unwrap();
        service_map.insert_local("test", Box::new(|request| request.to_vec())).unwrap();

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_remote_and_clean_up() {
        let node_id = ID::new_random();
        let mut service_map = ServiceMap::new();
        service_map.insert_remote("test", node_id).unwrap();

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn get_link() {
        let node_id = ID::new_random();
        let mut service_map = ServiceMap::new();
        service_map.insert_remote("test", node_id).unwrap();

        let link = service_map.get_link("test");
        assert!(link.is_some());
        match *link.unwrap() {
            Link::Remote(id) => assert_eq!(node_id, id),
            _ => unreachable!(),
        }
    }

}
