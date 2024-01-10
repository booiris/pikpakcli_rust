use anyhow::Result;
use std::time::SystemTime;

pub fn setup_logger(level: log::LevelFilter, log_path: &str) -> Result<()> {
    let mut logger = fern::Dispatch::new()
        .level(level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                record.line().unwrap_or_default(),
                message
            ))
        })
        .chain(std::io::stdout());

    if !log_path.is_empty() {
        logger = logger.chain(fern::log_file(log_path)?)
    }
    Ok(logger.apply()?)
}

#[cfg(test)]
pub fn setup_test_logger() -> Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
