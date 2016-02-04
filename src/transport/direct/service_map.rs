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
use std::sync::{Arc, Mutex, RwLock};

use metric::{self, Metric};
use node::{ID, request};
use transport::direct::{self, Link};

pub struct ServiceMap {
    balancer: Box<direct::Balancer>,
    entries: RwLock<HashMap<String, Entry>>,
    services_gauge: metric::item::Gauge,
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
    pub fn new(balancer: Box<direct::Balancer>, metric: Arc<Metric>) -> Self {
        ServiceMap {
            balancer: balancer,
            entries: RwLock::new(HashMap::new()),
            services_gauge: metric.gauge("services"),
        }
    }

    pub fn insert_local(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), Entry::new());
            self.services_gauge.change(1);
        }
        let mut entry = entries.get_mut(name).unwrap();

        if entry.local_handler.is_some() {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.local_handler = Some(Arc::new(Mutex::new(f)));
        entry.links.push(Link::Local);

        Ok(())
    }

    pub fn insert_remote(&self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(), Entry::new());
            self.services_gauge.change(1);
        }
        let mut entry = entries.get_mut(name).unwrap();

        if let Some(_) = entry.links.iter().find(|&link| Link::is_remote(link, &peer_node_id)) {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.links.push(Link::Remote(peer_node_id));

        Ok(())
    }

    pub fn insert_remotes(&self, names: &[String], peer_node_id: ID) {
        let mut entries = self.entries.write().unwrap();

        for name in names {
            if !entries.contains_key(name) {
                entries.insert(name.to_string(), Entry::new());
                self.services_gauge.change(1);
            }
            let mut entry = entries.get_mut(name).unwrap();

            if let None = entry.links.iter().find(|&link| Link::is_remote(link, &peer_node_id)) {
                entry.links.push(Link::Remote(peer_node_id));
            }
        }
    }

    pub fn select<L, R>(&self,
                        name: &str,
                        reader: Box<request::Reader>,
                        response_writer: Box<request::ResponseWriter>,
                        local_handler: L,
                        remote_handler: R)
                        -> request::Response
        where L: FnOnce(Box<request::Reader>,
                        Box<request::ResponseWriter>,
                        &Arc<Mutex<Box<request::Handler>>>)
                        -> request::Response,
              R: FnOnce(Box<request::Reader>, Box<request::ResponseWriter>, ID) -> request::Response
    {
        let mut entries = self.entries.write().unwrap();

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
            Link::Local => {
                local_handler(reader,
                              response_writer,
                              entry.local_handler.as_ref().unwrap())
            }
            Link::Remote(ref peer_node_id) => {
                remote_handler(reader, response_writer, *peer_node_id)
            }
        }
    }

    pub fn select_local<L>(&self, name: &str, local_handler: L) -> request::Response
        where L: FnOnce(&Box<request::Handler>) -> request::Response
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
            local_handler(&*entry.local_handler.as_ref().unwrap().lock().unwrap())
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
            self.services_gauge.change(-1);
        }
        Ok(())
    }

    pub fn remove_remote(&self, name: &str, peer_node_id: &ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        let remove = {
            let mut entry = match entries.get_mut(name) {
                Some(entry) => entry,
                None => return Err(Error::ServiceDoesNotExists),
            };
            entry.links.retain(|link| !Link::is_remote(&&link, peer_node_id));
            entry.queue.retain(|link| !Link::is_remote(&&link, peer_node_id));
            entry.links.len() == 0
        };
        if remove {
            entries.remove(name);
            self.services_gauge.change(-1);
        }
        Ok(())
    }

    pub fn remove_remotes(&self, names: &[String], peer_node_id: &ID) {
        let mut entries = self.entries.write().unwrap(); // block
        for name in names {
            let remove = {
                let mut entry = match entries.get_mut(name) {
                    Some(entry) => entry,
                    None => continue,
                };
                entry.links.retain(|link| !Link::is_remote(&&link, peer_node_id));
                entry.queue.retain(|link| !Link::is_remote(&&link, peer_node_id));
                entry.links.len() == 0
            };
            if remove {
                entries.remove(name);
                self.services_gauge.change(-1);
            }
        }
    }

    pub fn remove_all_remotes(&self, peer_node_id: &ID) {
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
            self.services_gauge.change(-1);
        }
    }
}

struct Entry {
    local_handler: Option<Arc<Mutex<Box<request::Handler>>>>,
    links: Vec<Link>,
    queue: Vec<Link>,
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

    use std::sync::Arc;
    use metric;
    use node::ID;
    use super::ServiceMap;
    use super::super::balancer;

    #[test]
    fn insert_local() {
        let service_map = build_service_map();

        assert!(service_map.insert_local("test", Box::new(|request| Ok(request))).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request)))
                           .is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn insert_remote() {
        let service_map = build_service_map();
        let node_id = ID::new_random();

        assert!(service_map.insert_remote("test", node_id).is_ok());
        assert!(service_map.insert_remote("test", node_id).is_err());
        assert!(service_map.insert_remote("test", ID::new_random()).is_ok());
        assert!(service_map.insert_local("test", Box::new(|request| Ok(request))).is_ok());

        assert_eq!(vec!["test"], service_map.local_service_names());
    }

    #[test]
    fn remove_local() {
        let service_map = build_service_map();
        service_map.insert_local("test", Box::new(|request| Ok(request))).unwrap();
        service_map.insert_remote("test", ID::new_random()).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_local_and_clean_up() {
        let service_map = build_service_map();
        service_map.insert_local("test", Box::new(|request| Ok(request))).unwrap();

        assert!(service_map.remove_local("test").is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn remove_remote() {
        let service_map = build_service_map();
        let id_one = ID::new_random();
        let id_two = ID::new_random();
        service_map.insert_remote("test", id_one).unwrap();
        service_map.insert_remote("test", id_two).unwrap();

        assert!(service_map.remove_remote("test", &id_one).is_ok());

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_remote_and_clean_up() {
        let service_map = build_service_map();
        let id = ID::new_random();
        service_map.insert_remote("test", id).unwrap();

        assert!(service_map.remove_remote("test", &id).is_ok());

        assert_eq!(0, service_map.len());
    }

    #[test]
    fn remove_all_remotes() {
        let service_map = build_service_map();
        let node_id = ID::new_random();
        service_map.insert_remote("test", node_id).unwrap();
        service_map.insert_local("test", Box::new(|request| Ok(request))).unwrap();

        service_map.remove_all_remotes(&node_id);

        assert_eq!(1, service_map.len());
    }

    #[test]
    fn remove_all_remotes_and_clean_up() {
        let service_map = build_service_map();
        let node_id = ID::new_random();
        service_map.insert_remote("test", node_id).unwrap();

        service_map.remove_all_remotes(&node_id);

        assert_eq!(0, service_map.len());
    }

    fn build_service_map() -> ServiceMap {
        let balancer = Box::new(balancer::DynamicRoundRobin::new());
        let metric = Arc::new(metric::Memory::new());
        ServiceMap::new(balancer, metric)
    }

}
