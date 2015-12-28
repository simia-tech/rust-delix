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
use std::sync::{RwLock, RwLockWriteGuard};

use node::{ID, request};
use transport::direct;

pub struct ServiceMap {
    map: RwLock<HashMap<String, Vec<Link>>>,
}

enum Link {
    Local(Box<request::Handler>),
    Remote(ID),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceAlreadyExists,
    ServiceDoesNotExists,
    Connection(direct::ConnectionError),
    ConnectionMap(direct::ConnectionMapError),
}

impl ServiceMap {
    pub fn new() -> ServiceMap {
        ServiceMap { map: RwLock::new(HashMap::new()) }
    }

    pub fn insert_local(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        let mut map = self.map.write().unwrap();

        let mut links = get_or_add_links(&mut map, name);
        if let Some(_) = links.iter().find(local_link) {
            return Err(Error::ServiceAlreadyExists);
        }

        links.push(Link::Local(f));
        Ok(())
    }

    pub fn insert_remote(&self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut map = self.map.write().unwrap();

        let mut links = get_or_add_links(&mut map, name);
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

    pub fn call_local_handler_or<F>(&self, name: &str, data: &[u8], f: F) -> request::Response
        where F: Fn(ID) -> request::Response
    {
        let map = self.map.read().unwrap();

        let link = match map.get(name).and_then(|links| links.first()) {
            Some(link) => link,
            None => return Err(request::Error::ServiceDoesNotExists),
        };

        match *link {
            Link::Local(ref service_handler) => {
                service_handler(data).map_err(|text| request::Error::Internal(text))
            }
            Link::Remote(ref peer_node_id) => f(*peer_node_id),
        }
    }

    pub fn local_service_names(&self) -> Vec<String> {
        self.map
            .read()
            .unwrap()
            .iter()
            .filter_map(|(name, links)| links.iter().find(local_link).and(Some(name.to_string())))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.read().unwrap().len()
    }

    pub fn remove_local(&self, name: &str) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let remove = {
            let mut links = match map.get_mut(name) {
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
            map.remove(name);
        }
        Ok(())
    }

    pub fn remove_remote(&self, peer_node_id: &ID) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let mut names = Vec::new();
        for (name, links) in map.iter_mut() {
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
            map.remove(&name);
        }
        Ok(())
    }
}

fn get_or_add_links<'a>(map: &'a mut RwLockWriteGuard<HashMap<String, Vec<Link>>>,
                        name: &str)
                        -> &'a mut Vec<Link> {
    if !map.contains_key(name) {
        map.insert(name.to_string(), Vec::new());
    }
    map.get_mut(name).unwrap()
}

fn local_link(link: &&Link) -> bool {
    match **link {
        Link::Local(_) => true,
        Link::Remote(_) => false,
    }
}

unsafe impl Send for ServiceMap {}

unsafe impl Sync for ServiceMap {}

impl From<direct::ConnectionError> for Error {
    fn from(error: direct::ConnectionError) -> Self {
        Error::Connection(error)
    }
}

impl From<direct::ConnectionMapError> for Error {
    fn from(error: direct::ConnectionMapError) -> Self {
        Error::ConnectionMap(error)
    }
}

#[cfg(test)]
mod tests {

    use super::ServiceMap;
    use node::ID;

    #[test]
    fn insert_local() {
        let service_map = ServiceMap::new();

        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec())))
                           .is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn insert_remote() {
        let node_id = ID::new_random();
        let service_map = ServiceMap::new();

        assert!(service_map.insert_remote("test", node_id).is_ok());
        assert!(service_map.insert_remote("test", node_id).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn remove_local() {
        let service_map = ServiceMap::new();
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();
        service_map.insert_remote("test", ID::new_random()).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_local_and_clean_up() {
        let service_map = ServiceMap::new();
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn remove_remote() {
        let node_id = ID::new_random();
        let service_map = ServiceMap::new();
        service_map.insert_remote("test", node_id).unwrap();
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_remote_and_clean_up() {
        let node_id = ID::new_random();
        let service_map = ServiceMap::new();
        service_map.insert_remote("test", node_id).unwrap();

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(0, service_map.len());
    }

}
