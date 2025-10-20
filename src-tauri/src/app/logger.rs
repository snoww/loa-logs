use flexi_logger::{
    Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, FlexiLoggerError, Logger, LoggerHandle,
    Naming, Record, WriteMode,
};

use crate::app;

pub fn init() -> Result<LoggerHandle, FlexiLoggerError> {

    let mut logger = if cfg!(debug_assertions) {
        Logger::try_with_str("info, tao=off").unwrap()
    }
    else {
        Logger::try_with_str("info, live=warn, stats_api=warn, heartbeat_api=warn, tao=off").unwrap()
    };

    logger = logger
        .log_to_file(
            FileSpec::default()
                .suppress_timestamp()
                .basename("loa_logs")
                .directory(app::path::log_dir()),
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
        logger = logger.duplicate_to_stdout(Duplicate::All);
    }

    logger.start()
}

fn default_format_with_time(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] {} [{}] {}",
        now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}
