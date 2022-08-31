use std::{fs::File, path::PathBuf};
use xdg::{BaseDirectories, BaseDirectoriesError};

lazy_static! {
    /// The public cacher which can be used to interact with leftwm's cache.
    pub static ref CACHER: Cacher = Cacher::new();
}

#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error("Couldn't open base directory.")]
    BaseDirError(#[from] BaseDirectoriesError),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IOError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cacher {
    cache_dir: PathBuf,
}

impl Cacher {
    const CACHE_DIR_NAME: &'static str = "leftwm";

    fn new() -> Self {
        let cache_dir = BaseDirectories::new().unwrap();
        let cache_dir = cache_dir
            .create_cache_directory(Self::CACHE_DIR_NAME)
            .unwrap();

        Self { cache_dir }
    }

    /// Creates the a cache file in the cache directory of `leftwm`.
    ///
    /// # Params
    /// - `path`: The *relative* path of the `leftwm` cache dir.
    ///
    /// # Examples
    /// ```rust
    /// use crate::utils::Cacher;
    ///
    /// let cacher = Cacher::new().unwrap();
    ///
    /// // (for UNIX-based systems) creates the file `~/.cache/leftwm/yeet.rofl`
    /// cacher.get_file(PathBuf::from(r"yeet.rofl")).unwrap();
    /// ```
    pub fn get_file(&self, path: PathBuf) -> Result<File, Error> {
        let mut cache_file_path: PathBuf = self.cache_dir.clone();
        cache_file_path.push(path);

        File::create(cache_file_path.as_path()).map_err(|err| Error::IOError(err.to_string()))
    }
}
