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

extern crate rustbox;

use std::collections::VecDeque;
use std::default::Default;
use std::sync::RwLock;
use std::time::Duration;

use self::rustbox::{Color, Key, RustBox};

use super::{Memory, Metric, Query, Value, item};

pub struct Terminal {
    refresh_interval_ms: u64,
    memory: Memory,

    screen: RwLock<Screen>,
    log_buffer: RwLock<VecDeque<String>>,
}

enum Screen {
    Services,
    Log,
}

impl Terminal {
    pub fn new(refresh_interval_ms: u64) -> Self {
        Terminal {
            refresh_interval_ms: refresh_interval_ms,
            memory: Memory::new(),
            screen: RwLock::new(Screen::Services),
            log_buffer: RwLock::new(VecDeque::new()),
        }
    }

    fn draw_head(&self, rustbox: &RustBox) {
        let mut line = String::new();
        line.push_str(" Delix");
        pad(&mut line, rustbox.width());

        rustbox.print(0, 0, rustbox::RB_BOLD, Color::White, Color::Black, &line);
    }

    fn draw_main(&self, rustbox: &RustBox) {
        match *self.screen.read().unwrap() {
            Screen::Services => self.draw_main_services(rustbox),
            Screen::Log => self.draw_main_log(rustbox),
        }
    }

    fn draw_main_services(&self, rustbox: &RustBox) {
        let map = self.memory.get_all_with_prefix("service.");
        let mut keys = map.keys().collect::<Vec<_>>();
        keys.sort();

        let mut row = 1;
        let mut current_service = "";
        let mut current_direction = "";
        for key in keys {
            let parts = key.split('.').collect::<Vec<&str>>();
            let service = parts[1];
            let direction = parts[2];
            let endpoint = parts[3];

            if current_service != service {
                current_service = service;
                current_direction = "";

                let mut line = format!("{}", service);
                pad(&mut line, rustbox.width());
                rustbox.print(0,
                              row,
                              rustbox::RB_BOLD,
                              Color::White,
                              Color::Default,
                              &line);
                row += 1;
            }

            if current_direction != direction {
                current_direction = direction;

                let mut line = format!("  {}", direction);
                pad(&mut line, rustbox.width());
                rustbox.print(0,
                              row,
                              rustbox::RB_NORMAL,
                              Color::White,
                              Color::Default,
                              &line);
                row += 1;
            }

            let value = map.get(key).unwrap();

            let mut line = match *value {
                Value::Counter(v) => format!("    {:<12} {:>6?}", endpoint, v),
                Value::Gauge(v) => format!("    {:<12} {:>6?}", endpoint, v),
            };
            pad(&mut line, rustbox.width());
            rustbox.print(0,
                          row,
                          rustbox::RB_NORMAL,
                          Color::White,
                          Color::Default,
                          &line);
            row += 1;
        }

        let mut blank_line = String::new();
        pad(&mut blank_line, rustbox.width());
        while row < (rustbox.height() - 1) {
            rustbox.print(0,
                          row,
                          rustbox::RB_NORMAL,
                          Color::White,
                          Color::Default,
                          &blank_line);
            row += 1;
        }
    }

    fn draw_main_log(&self, rustbox: &RustBox) {
        let log_buffer = self.log_buffer.read().unwrap();
        let mut row = 1;
        for log_line in log_buffer.iter() {
            let mut line = String::new();
            line.push_str(log_line);
            pad(&mut line, rustbox.width());

            rustbox.print(0,
                          row,
                          rustbox::RB_NORMAL,
                          Color::White,
                          Color::Default,
                          &line);
            row += 1;
        }
    }

    fn draw_foot(&self, rustbox: &RustBox) {
        let connections = match self.memory.get("connections") {
            Some(Value::Gauge(value)) => value,
            _ => 0,
        };

        let services = match self.memory.get("services") {
            Some(Value::Gauge(value)) => value,
            _ => 0,
        };

        let endpoints = match self.memory.get("endpoints") {
            Some(Value::Gauge(value)) => value,
            _ => 0,
        };

        let requests = match self.memory.get("requests") {
            Some(Value::Counter(value)) => value,
            _ => 0,
        };

        let mut line = format!(" {} connection(s) / {} service(s) / {} endpoint(s) / {} \
                                request(s)",
                               connections,
                               services,
                               endpoints,
                               requests);

        pad(&mut line, rustbox.width());

        rustbox.print(0,
                      rustbox.height() - 1,
                      rustbox::RB_BOLD,
                      Color::White,
                      Color::Black,
                      &line);
    }
}

impl Metric for Terminal {
    fn log(&self, tag: &str, target: &str, text: &str) {
        let mut log_buffer = self.log_buffer.write().unwrap();

        while log_buffer.len() >= log_buffer.capacity() {
            log_buffer.pop_front();
        }
        log_buffer.push_back(format!("[{}] [{}] {}", tag, target, text));
    }

    fn counter(&self, key: &str) -> item::Counter {
        self.memory.counter(key)
    }

    fn gauge(&self, key: &str) -> item::Gauge {
        self.memory.gauge(key)
    }

    fn display(&self) {
        let rustbox = RustBox::init(Default::default()).unwrap();

        {
            let mut log_buffer = self.log_buffer.write().unwrap();
            let mut new_buffer = VecDeque::with_capacity(rustbox.height() - 2);
            new_buffer.append(&mut *log_buffer);
            *log_buffer = new_buffer;
        }

        self.draw_head(&rustbox);
        self.draw_main(&rustbox);
        self.draw_foot(&rustbox);
        rustbox.present();

        loop {
            match rustbox.peek_event(Duration::from_millis(self.refresh_interval_ms), false)
                         .unwrap() {
                rustbox::Event::KeyEvent(key) => {
                    match key {
                        Key::Char('1') => {
                            *self.screen.write().unwrap() = Screen::Services;
                        }
                        Key::Char('2') => {
                            *self.screen.write().unwrap() = Screen::Log;
                        }
                        Key::Char('q') => {
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            self.draw_main(&rustbox);
            self.draw_foot(&rustbox);
            rustbox.present();
        }
    }
}

fn pad(s: &mut String, size: usize) {
    while s.len() < size {
        s.push_str(" ");
    }
}
