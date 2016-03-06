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
    buffer: RwLock<VecDeque<String>>,
}

impl Terminal {
    pub fn new(refresh_interval_ms: u64) -> Self {
        Terminal {
            refresh_interval_ms: refresh_interval_ms,
            memory: Memory::new(),
            buffer: RwLock::new(VecDeque::new()),
        }
    }

    fn draw_head(&self, rustbox: &RustBox) {
        let mut line = String::new();
        line.push_str(" Delix");
        pad(&mut line, rustbox.width());

        rustbox.print(0, 0, rustbox::RB_BOLD, Color::White, Color::Black, &line);
    }

    fn draw_main(&self, rustbox: &RustBox) {
        let buffer = self.buffer.read().unwrap();
        let mut row = 1;
        for log_line in buffer.iter() {
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
        let mut buffer = self.buffer.write().unwrap();

        while buffer.len() >= buffer.capacity() {
            buffer.pop_front();
        }
        buffer.push_back(format!("[{}] [{}] {}", tag, target, text));
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
            let mut buffer = self.buffer.write().unwrap();
            let mut new_buffer = VecDeque::with_capacity(rustbox.height() - 2);
            new_buffer.append(&mut *buffer);
            *buffer = new_buffer;
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
