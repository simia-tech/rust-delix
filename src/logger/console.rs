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

use ansi_term::Colour;
use log;

pub struct Console {
    level_filter: log::LogLevelFilter,
    target_prefix: String,
}

impl Console {
    pub fn init(level_filter: log::LogLevelFilter,
                target_prefix: &str)
                -> Result<(), log::SetLoggerError> {
        let result = log::set_logger(|maximal_log_level| {
            maximal_log_level.set(level_filter);
            Box::new(Console::new(level_filter, target_prefix))
        });
        result
    }
}

impl Console {
    pub fn new(level_filter: log::LogLevelFilter, target_prefix: &str) -> Console {
        Console {
            level_filter: level_filter,
            target_prefix: target_prefix.to_string(),
        }
    }
}

impl log::Log for Console {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &log::LogRecord) {
        let metadata = record.metadata();
        if !self.enabled(metadata) {
            return;
        }

        let target = metadata.target();
        if !target.starts_with(&self.target_prefix) {
            return;
        }

        let tag = match record.level() {
            log::LogLevel::Error => Colour::Red.paint("ERROR"),
            log::LogLevel::Warn => Colour::Yellow.paint(" WARN"),
            log::LogLevel::Info => Colour::Cyan.paint(" INFO"),
            log::LogLevel::Debug => Colour::Blue.paint("DEBUG"),
            log::LogLevel::Trace => Colour::White.paint("TRACE"),
        };

        println!("[{}] [{}] {}",
                 tag,
                 record.metadata().target(),
                 record.args());
    }
}
