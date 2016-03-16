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
use std::sync::{Arc, RwLock};

use metric::{self, Metric};
use node::{ID, Service, request};
use transport::direct::{self, Link};
use transport::direct::balancer::{self, Balancer};

pub struct ServiceMap {
    balancer_factory: Box<balancer::Factory>,
    entries: RwLock<HashMap<String, Entry>>,
    metric: Arc<Metric>,
    services_gauge: metric::item::Gauge,
    endpoints_gauge: metric::item::Gauge,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceAlreadyExists,
    ServiceDoesNotExists,
    ConnectionMap(direct::ConnectionMapError),
}

impl ServiceMap {
    pub fn new(balancer_factory: Box<balancer::Factory>, metric: Arc<Metric>) -> Self {
        ServiceMap {
            balancer_factory: balancer_factory,
            entries: RwLock::new(HashMap::default()),
            metric: metric.clone(),
            services_gauge: metric.gauge("services"),
            endpoints_gauge: metric.gauge("endpoints"),
        }
    }

    pub fn insert_local(&self, name: &str, f: Box<Service>) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(),
                           Entry::new(name,
                                      self.balancer_factory.build(name),
                                      self.metric.clone()));
            self.services_gauge.change(1);
        }
        let mut entry = entries.get_mut(name).unwrap();

        if entry.local_handler.is_some() {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.add_local_link(Arc::new(f));
        self.endpoints_gauge.change(1);

        Ok(())
    }

    pub fn insert_remote(&self, name: &str, peer_node_id: ID) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(name) {
            entries.insert(name.to_string(),
                           Entry::new(name,
                                      self.balancer_factory.build(name),
                                      self.metric.clone()));
            self.services_gauge.change(1);
        }
        let mut entry = entries.get_mut(name).unwrap();

        if let Some(_) = entry.links.iter().find(|&link| Link::is_remote(link, &peer_node_id)) {
            return Err(Error::ServiceAlreadyExists);
        }

        entry.add_remote_link(peer_node_id);
        self.endpoints_gauge.change(1);

        Ok(())
    }

    pub fn insert_remotes(&self, names: &[String], peer_node_id: ID) {
        let mut entries = self.entries.write().unwrap();

        for name in names {
            if !entries.contains_key(name) {
                entries.insert(name.to_string(),
                               Entry::new(name,
                                          self.balancer_factory.build(name),
                                          self.metric.clone()));
                self.services_gauge.change(1);
            }
            let mut entry = entries.get_mut(name).unwrap();

            if let None = entry.links.iter().find(|&link| Link::is_remote(link, &peer_node_id)) {
                entry.add_remote_link(peer_node_id);
                self.endpoints_gauge.change(1);
            }
        }
    }

    pub fn get(&self, name: &str) -> request::Result<(Link, Option<Arc<Box<Service>>>)> {
        let mut entries = self.entries.write().unwrap();

        let mut entry = match entries.get_mut(name) {
            Some(entry) => entry,
            None => return Err(request::Error::NoService),
        };

        let link = entry.select_link();

        Ok((link,
            entry.local_handler.as_ref().map(|handler| handler.clone())))
    }

    pub fn get_local(&self, name: &str) -> Option<Arc<Box<Service>>> {
        let entries = self.entries.read().unwrap();
        entries.get(name)
               .and_then(|entry| entry.select_local_link())
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
            entry.remove_local_link();
            self.endpoints_gauge.change(-1);
            !entry.has_links()
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
            entry.remove_remote_link(peer_node_id);
            self.endpoints_gauge.change(-1);
            !entry.has_links()
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
                entry.remove_remote_link(peer_node_id);
                self.endpoints_gauge.change(-1);
                !entry.has_links()
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
            entry.remove_remote_link(peer_node_id);
            self.endpoints_gauge.change(-1);
            if !entry.has_links() {
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
    name: String,
    balancer: Box<Balancer<Item = Link>>,
    metric: Arc<Metric>,
    local_handler: Option<Arc<Box<Service>>>,
    links: Vec<Link>,
    counters: HashMap<Link, metric::item::Counter>,
}

impl Entry {
    fn new(name: &str, balancer: Box<Balancer<Item = Link>>, metric: Arc<Metric>) -> Entry {
        Entry {
            name: name.to_string(),
            balancer: balancer,
            metric: metric,
            local_handler: None,
            links: Vec::new(),
            counters: HashMap::default(),
        }
    }

    fn add_local_link(&mut self, local_handler: Arc<Box<Service>>) {
        self.local_handler = Some(local_handler);
        self.links.push(Link::Local);
        self.counters.insert(Link::Local,
                             self.metric.counter(&format!("service.{}.endpoint.local.selected",
                                                          self.name)));
        self.balancer.set_links(&self.links);
    }

    fn remove_local_link(&mut self) {
        self.local_handler = None;
        self.links.retain(|link| !Link::is_local(link));
        self.counters.remove(&Link::Local);
        self.balancer.set_links(&self.links);
    }

    fn add_remote_link(&mut self, peer_node_id: ID) {
        self.links.push(Link::Remote(peer_node_id));
        self.counters.insert(Link::Remote(peer_node_id),
                             self.metric.counter(&format!("service.{}.endpoint.{}.selected",
                                                          self.name,
                                                          peer_node_id)));
        self.balancer.set_links(&self.links);
    }

    fn remove_remote_link(&mut self, peer_node_id: &ID) {
        self.links.retain(|link| !Link::is_remote(link, peer_node_id));
        self.counters.remove(&Link::Remote(*peer_node_id));
        self.balancer.set_links(&self.links);
    }

    fn select_link(&mut self) -> Link {
        let link = self.balancer.next().expect("balancer did not produce any link");
        self.counters.get(&link).unwrap().increment();
        link
    }

    fn select_local_link(&self) -> Option<Arc<Box<Service>>> {
        match self.local_handler {
            Some(ref local_handler) => Some(local_handler.clone()),
            None => None,
        }
    }

    fn has_links(&self) -> bool {
        !self.links.is_empty()
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
    use super::super::balancer::{self, Factory};
    use super::super::tracker::Statistic;

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
        let mut balancer_factory = Box::new(balancer::DynamicRoundRobinFactory::new());
        balancer_factory.set_statistic(Arc::new(Statistic::new()));
        let metric = Arc::new(metric::Memory::new());
        ServiceMap::new(balancer_factory, metric)
    }

}
