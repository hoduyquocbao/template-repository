use kernel::logger::Logger;
use kernel::config::Config;
use std::time::Duration;

fn main() {
    let config = Config::new();
    let logger = Logger::new(&config).unwrap();
    logger.info("Info message");
    logger.warn("Warning message");
    logger.error("Error message");
    logger.debug("Debug message");
    logger.trace("Trace message");
    logger.context("EXAMPLE", "Context message");
    logger.performance("operation", Duration::from_millis(123));
} 