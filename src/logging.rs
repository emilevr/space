use crate::cli::environment::EnvServiceTrait;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(test)]
#[path = "./logging_test.rs"]
mod logging_test;

pub(crate) const SPACE_LOG_LEVEL_ENV_VAR_NAME: &str = "SPACE_LOG_LEVEL";

// Set a logger. Do not panic or return an error if anything goes wrong.
pub(crate) fn configure_logger<T: EnvServiceTrait>(
    user_home_dir: Option<PathBuf>,
    env_service: &T,
) -> LevelFilter {
    let level = if let Ok(log_level) = env_service.var(SPACE_LOG_LEVEL_ENV_VAR_NAME) {
        if let Ok(level) = LevelFilter::from_str(log_level.as_str()) {
            level
        } else {
            LevelFilter::Warn
        }
    } else {
        LevelFilter::Warn
    };

    if let Some(user_home_dir) = user_home_dir {
        if let Ok(logfile) = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%dT%H:%M:%SZ)(utc)} - {m}{n}",
            )))
            .build(user_home_dir.join(".space").join("space.log"))
        {
            if let Ok(config) = Config::builder()
                .appender(Appender::builder().build("logfile", Box::new(logfile)))
                .build(Root::builder().appender("logfile").build(level))
            {
                let _ = log4rs::init_config(config);
            }
        }
    }

    level
}
