use std::io::{self, Write};

use log::{Log, Metadata, Record};

use crate::colors::Colors;
use crate::line::Line;
use crate::timestamp::Timestamp;

/// Writes every enabled log record as one line on stderr.
pub(crate) struct Logger {
    colors: Colors,
}

impl Logger {
    pub(crate) fn new(colors: Colors) -> Self {
        Self { colors }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = Line {
            timestamp: Timestamp::now(),
            record,
            colors: self.colors,
        };
        write_whole(&format!("{line}\n"));
    }

    /// Stderr is not buffered, so there is never anything to flush.
    fn flush(&self) {}
}

/// Writes a line in one piece, so that lines from different threads never get mixed.
///
/// When stderr itself is gone there is nobody left to tell, so a failed write is ignored.
fn write_whole(line: &str) {
    let _ = io::stderr().lock().write_all(line.as_bytes());
}
