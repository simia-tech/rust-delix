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
use transport::direct;
use transport::direct::tracker::Subject;

pub struct ServiceMap {
    balancer: Box<direct::Balancer>,
    entries: RwLock<HashMap<String, (Vec<Link>, Vec<Subject>)>>,
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
    pub fn new(balancer: Box<direct::Balancer>) -> ServiceMap {
        ServiceMap {
            balancer: balancer,
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_local(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), (Vec::new(), Vec::new()));
        }
        let mut links = entries.get_mut(name).unwrap();

        if let Some(_) = links.0.iter().find(local_link) {
            return Err(Error::ServiceAlreadyExists);
        }

        links.0.push(Link::Local(f));
        Ok(())
    }

    pub fn insert_remote(&self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), (Vec::new(), Vec::new()));
        }
        let mut links = entries.get_mut(name).unwrap();

        if let Some(_) = links.0.iter().find(|link| {
            match **link {
                Link::Local(_) => false,
                Link::Remote(id) => id == peer_node_id,
            }
        }) {
            return Err(Error::ServiceAlreadyExists);
        }

        links.0.push(Link::Remote(peer_node_id));
        Ok(())
    }

    pub fn select<L, R>(&self, name: &str, local_handler: L, remote_handler: R) -> request::Response
        where L: Fn(&Box<request::Handler>) -> request::Response,
              R: Fn(ID) -> request::Response
    {
        let mut entries = self.entries.write().unwrap();

        let link = match entries.get_mut(name).and_then(|entry| {
            if entry.1.is_empty() {
                entry.1.append(&mut self.balancer
                                        .build_round(&to_subjects(&entry.0, name)));
                entry.1.reverse();
            }
            let subject = entry.1.pop().expect("balancer did not build any round");
            match subject {
                Subject::Local(_) => entry.0.iter().find(local_link),
                Subject::Remote(_, peer_node_id) => {
                    entry.0.iter().find(|&link| {
                        match *link {
                            Link::Remote(id) if id == peer_node_id => true,
                            _ => false,
                        }
                    })
                }
            }
        }) {
            Some(subject) => subject,
            None => return Err(request::Error::ServiceDoesNotExists),
        };

        match *link {
            Link::Local(ref service_handler) => local_handler(service_handler),
            Link::Remote(ref peer_node_id) => remote_handler(*peer_node_id),
        }
    }

    pub fn select_local<L>(&self, name: &str, local_handler: L) -> request::Response
        where L: Fn(&Box<request::Handler>) -> request::Response
    {
        let entries = self.entries.read().unwrap();

        let link = match entries.get(name).and_then(|entry| entry.0.iter().find(local_link)) {
            Some(subject) => subject,
            None => return Err(request::Error::ServiceDoesNotExists),
        };

        if let Link::Local(ref service_handler) = *link {
            local_handler(service_handler)
        } else {
            unreachable!();
        }
    }

    pub fn local_service_names(&self) -> Vec<String> {
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter_map(|(name, links)| links.0.iter().find(local_link).and(Some(name.to_string())))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn remove_local(&self, name: &str) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        let remove = {
            let mut links = match entries.get_mut(name) {
                Some(links) => links,
                None => return Err(Error::ServiceDoesNotExists),
            };
            links.0.retain(|link| {
                match *link {
                    Link::Local(_) => false,
                    _ => true,
                }
            });
            links.0.len() == 0
        };
        if remove {
            entries.remove(name);
        }
        Ok(())
    }

    pub fn remove_remote(&self, peer_node_id: &ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        let mut names = Vec::new();
        for (name, links) in entries.iter_mut() {
            links.0.retain(|link| {
                match *link {
                    Link::Remote(ref node_id) if node_id == peer_node_id => false,
                    _ => true,
                }
            });
            if links.0.len() == 0 {
                names.push(name.to_string());
            }
        }
        for name in names {
            entries.remove(&name);
        }
        Ok(())
    }
}

fn local_link(link: &&Link) -> bool {
    match **link {
        Link::Local(_) => true,
        Link::Remote(_) => false,
    }
}

fn to_subjects(links: &[Link], name: &str) -> Vec<Subject> {
    links.iter()
         .map(|link| {
             match *link {
                 Link::Local(_) => Subject::local(name),
                 Link::Remote(peer_node_id) => Subject::remote(name, peer_node_id),
             }
         })
         .collect()
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

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_remote_and_clean_up() {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let service_map = ServiceMap::new(balancer);
        let node_id = ID::new_random();
        service_map.insert_remote("test", node_id).unwrap();

        assert!(service_map.remove_remote(&node_id).is_ok());

        assert_eq!(0, service_map.len());
    }

}
