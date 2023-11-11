#[cfg(test)]
use mockall::{automock, predicate::*};
use std::{
    env::{self, VarError},
    io::{self},
    ops::Deref,
    path::PathBuf,
};

#[cfg_attr(test, automock)]
pub(crate) trait EnvServiceTrait {
    fn var(&self, key: &str) -> Result<String, VarError>;
    fn current_dir(&self) -> io::Result<PathBuf>;
}

#[derive(Default)]
pub(crate) struct DefaultEnvService {}

impl EnvServiceTrait for DefaultEnvService {
    fn var(&self, key: &str) -> Result<String, VarError> {
        env::var(key)
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        env::current_dir()
    }
}

impl EnvServiceTrait for Box<dyn EnvServiceTrait> {
    fn var(&self, key: &str) -> Result<String, VarError> {
        self.deref().var(key)
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        self.deref().current_dir()
    }
}

#[cfg(test)]
mod test {
    use super::{DefaultEnvService, EnvServiceTrait};

    #[test]
    fn default_env_service_current_dir_returns_env_current_dir() {
        // Arrange
        let service = DefaultEnvService::default();

        // Act
        let current_dir = service
            .current_dir()
            .expect("Should be able to get current dir via service");

        // Assert
        let env_current_dir =
            std::env::current_dir().expect("Should be able to get current dir via env");
        assert_eq!(env_current_dir, current_dir);
    }
}
