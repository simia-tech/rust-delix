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
use std::sync::RwLock;

use node::{ID, request};
use transport::direct::{self, Link};

pub struct ServiceMap {
    balancer: Box<direct::Balancer>,
    entries: RwLock<HashMap<String, Entry>>,
}

struct Entry {
    local_handler: Option<Box<request::Handler>>,
    links: Vec<Link>,
    queue: Vec<Link>,
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
    pub fn new(balancer: Box<direct::Balancer>) -> ServiceMap {
        ServiceMap {
            balancer: balancer,
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_local(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), Entry::new());
        }
        let mut entry = entries.get_mut(name).unwrap();

        if entry.local_handler.is_some() {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.local_handler = Some(f);
        entry.links.push(Link::Local);

        Ok(())
    }

    pub fn insert_remote(&self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), Entry::new());
        }
        let mut entry = entries.get_mut(name).unwrap();

        if let Some(_) = entry.links.iter().find(|&link| Link::is_remote(link, &peer_node_id)) {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.links.push(Link::Remote(peer_node_id));

        Ok(())
    }

    pub fn select<L, R>(&self, name: &str, local_handler: L, remote_handler: R) -> request::Response
        where L: Fn(&Box<request::Handler>) -> request::Response,
              R: Fn(ID) -> request::Response
    {
        let mut entries = self.entries.write().unwrap();

        loop {
            let response_pair = {
                let mut entry = match entries.get_mut(name) {
                    Some(entry) => entry,
                    None => return Err(request::Error::ServiceDoesNotExists),
                };

                if entry.queue.is_empty() {
                    entry.queue.append(&mut self.balancer.build_round(name, &entry.links));
                    entry.queue.reverse();
                }
                let link = entry.queue.pop().expect("balancer did not build any round");

                match link {
                    Link::Local => (local_handler(entry.local_handler.as_ref().unwrap()), None),
                    Link::Remote(ref peer_node_id) => {
                        (remote_handler(*peer_node_id), Some(*peer_node_id))
                    }
                }
            };

            if let (Err(request::Error::ServiceDoesNotExists),
                    Some(peer_node_id)) = response_pair {
                let mut remove_entry = false;
                if let Some(entry) = entries.get_mut(name) {
                    entry.links.retain(|link| !Link::is_remote(&link, &peer_node_id));
                    remove_entry = entry.links.is_empty();
                }
                if remove_entry {
                    entries.remove(name);
                }
            } else {
                return response_pair.0;
            }
        }
    }

    pub fn select_local<L>(&self, name: &str, local_handler: L) -> request::Response
        where L: Fn(&Box<request::Handler>) -> request::Response
    {
        let entries = self.entries.read().unwrap();

        let entry = match entries.get(name) {
            Some(entry) => entry,
            None => return Err(request::Error::ServiceDoesNotExists),
        };

        let link = match entry.links.iter().find(|link| Link::is_local(link)) {
            Some(link) => link,
            None => return Err(request::Error::ServiceDoesNotExists),
        };

        if let Link::Local = *link {
            local_handler(entry.local_handler.as_ref().unwrap())
        } else {
            unreachable!();
        }
    }

    pub fn local_service_names(&self) -> Vec<String> {
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter_map(|(name, links)| {
                links.links.iter().find(|link| Link::is_local(link)).and(Some(name.to_string()))
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn remove_local(&self, name: &str) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        let remove = {
            let mut entry = match entries.get_mut(name) {
                Some(entry) => entry,
                None => return Err(Error::ServiceDoesNotExists),
            };
            entry.local_handler = None;
            entry.links.retain(|link| !Link::is_local(&&link));
            entry.queue.retain(|link| !Link::is_local(&&link));
            entry.links.len() == 0
        };
        if remove {
            entries.remove(name);
        }
        Ok(())
    }

    pub fn remove_remote(&self, peer_node_id: &ID) {
        let mut entries = self.entries.write().unwrap();
        let mut names = Vec::new();
        for (name, entry) in entries.iter_mut() {
            entry.links.retain(|link| !Link::is_remote(link, peer_node_id));
            entry.queue.retain(|link| !Link::is_remote(link, peer_node_id));
            if entry.links.len() == 0 {
                names.push(name.to_string());
            }
        }
        for name in names {
            entries.remove(&name);
        }
    }
}

impl Entry {
    fn new() -> Entry {
        Entry {
            local_handler: None,
            links: Vec::new(),
            queue: Vec::new(),
        }
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

    use node::ID;
    use super::ServiceMap;
    use super::super::balancer;

    #[test]
    fn insert_local() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);

        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec())))
                           .is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn insert_remote() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        let node_id = ID::new_random();

        assert!(service_map.insert_remote("test", node_id).is_ok());
        assert!(service_map.insert_remote("test", node_id).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn remove_local() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();
        service_map.insert_remote("test", ID::new_random()).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_local_and_clean_up() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn remove_remote() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        let node_id = ID::new_random();
        service_map.insert_remote("test", node_id).unwrap();
        service_map.insert_local("test", Box::new(|request| Ok(request.to_vec()))).unwrap();

        service_map.remove_remote(&node_id);

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_remote_and_clean_up() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        let node_id = ID::new_random();
        service_map.insert_remote("test", node_id).unwrap();

        service_map.remove_remote(&node_id);

        assert_eq!(0, service_map.len());
    }

}
