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

extern crate rustty;

use std::collections::VecDeque;
use std::sync::RwLock;

use self::rustty::{Attr, Cell, CellAccessor, Color, HasSize};
use self::rustty::ui::{self, Alignable, Painter};

use super::{Memory, Metric, Query, Value, item};

pub struct Terminal {
    refresh_interval_ms: u32,
    memory: Memory,
    buffer: RwLock<VecDeque<String>>,
}

impl Terminal {
    pub fn new(refresh_interval_ms: u32) -> Self {
        Terminal {
            refresh_interval_ms: refresh_interval_ms,
            memory: Memory::new(),
            buffer: RwLock::new(VecDeque::new()),
        }
    }

    fn draw_head(&self, widget: &mut ui::Widget) {
        let style = Cell::with_style(Color::White, Color::Black, Attr::Bold);
        widget.clear(style);

        let title = "Delix";
        let x = widget.halign_line(title, ui::HorizontalAlign::Middle, 0);
        widget.printline_with_cell(x, 0, title, style);
    }

    fn draw_main(&self, widget: &mut ui::Widget) {
        let style = Cell::with_style(Color::White, Color::Default, Attr::Default);
        widget.clear(style);

        let buffer = self.buffer.read().unwrap();
        let mut row = 0;
        for line in buffer.iter() {
            widget.printline_with_cell(0, row, &line, style);
            row += 1;
        }
    }

    fn draw_foot(&self, widget: &mut ui::Widget) {
        let style = Cell::with_style(Color::White, Color::Black, Attr::Bold);
        widget.clear(style);

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

        let status = format!("{} connection(s) / {} service(s) / {} endpoint(s) / {} request(s)",
                             connections,
                             services,
                             endpoints,
                             requests);

        let x = widget.halign_line(&status, ui::HorizontalAlign::Middle, 0);
        widget.printline_with_cell(x, 0, &status, style);
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
        let mut terminal = rustty::Terminal::new().unwrap();
        let (cols, rows) = terminal.size();

        let mut head = ui::Widget::new(cols, 1);
        head.align(&terminal,
                   ui::HorizontalAlign::Left,
                   ui::VerticalAlign::Top,
                   0);

        let mut main = ui::Widget::new(cols, rows - 2);
        main.align(&terminal,
                   ui::HorizontalAlign::Left,
                   ui::VerticalAlign::Middle,
                   0);

        let mut foot = ui::Widget::new(cols, 1);
        foot.align(&terminal,
                   ui::HorizontalAlign::Left,
                   ui::VerticalAlign::Bottom,
                   0);

        {
            let mut buffer = self.buffer.write().unwrap();
            let mut new_buffer = VecDeque::with_capacity(rows - 2);
            new_buffer.append(&mut *buffer);
            *buffer = new_buffer;
        }

        loop {
            if let Some(rustty::Event::Key(key)) =
                   terminal.get_event(self.refresh_interval_ms as isize)
                           .unwrap() {
                match key {
                    'q' => break,
                    _ => {}
                }
            }

            self.draw_head(&mut head);
            self.draw_main(&mut main);
            self.draw_foot(&mut foot);

            head.draw_into(&mut terminal);
            main.draw_into(&mut terminal);
            foot.draw_into(&mut terminal);
            terminal.swap_buffers().unwrap();
        }

    }
}
