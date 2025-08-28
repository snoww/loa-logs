use std::path::Path;

use flexi_logger::Duplicate;
use flexi_logger::{Cleanup, Criterion, DeferredNow, FileSpec, Logger, Naming, WriteMode};
use log::*;


pub fn setup_logger(current_dir: &Path) {
    let logger = Logger::try_with_str("info, tao=off")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .suppress_timestamp()
                .basename("loa_logs")
                .directory(current_dir),
        )
        .use_utc()
        .write_mode(WriteMode::BufferAndFlush)
        .append()
        .format(default_format_with_time)
        .rotate(
            Criterion::Size(5_000_000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(2),
        );

    #[cfg(debug_assertions)]
    {
        logger.duplicate_to_stdout(Duplicate::All);
    }
}

pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        error!("Panicked: {:?}", info);

        log::logger().flush();
    }));
}

fn default_format_with_time(
    writer: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        writer,
        "[{}] {} [{}] {}",
        now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}