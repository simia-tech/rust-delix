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

use std::sync::Arc;

use log;

use metric::Metric;

pub struct Console {
    metric: Arc<Metric>,
    level_filter: log::LogLevelFilter,
    target_prefix: String,
}

impl Console {
    pub fn init(level_filter: log::LogLevelFilter,
                target_prefix: &str,
                metric: &Arc<Metric>)
                -> Result<(), log::SetLoggerError> {
        let result = log::set_logger(|maximal_log_level| {
            maximal_log_level.set(level_filter);
            Box::new(Console::new(level_filter, target_prefix, metric))
        });
        result
    }
}

impl Console {
    pub fn new(level_filter: log::LogLevelFilter,
               target_prefix: &str,
               metric: &Arc<Metric>)
               -> Console {
        Console {
            metric: metric.clone(),
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
            log::LogLevel::Error => "ERROR",
            log::LogLevel::Warn => " WARN",
            log::LogLevel::Info => " INFO",
            log::LogLevel::Debug => "DEBUG",
            log::LogLevel::Trace => "TRACE",
        };
        let text = format!("{}", record.args());

        self.metric.log(&tag.to_string(), &target, &text);
    }
}
