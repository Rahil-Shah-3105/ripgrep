use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use anyhow::Context;
use redb::{Database, ReadOnlyDatabase};

const INDEX_FILE_NAME: &str = "index.db";

#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    db: Handle,
}

impl Index {
    pub fn builder() -> IndexBuilder {
		log::debug!(">>> Entering builder()");
        log::debug!("<<< Exiting builder()");
        IndexBuilder::new()
    }

    pub fn exists(path: impl AsRef<Path>) -> bool {
		log::debug!(">>> Entering exists(path=(unprintable))");
        let path = path.as_ref();
		log::debug!("exists(path=(unprintable)): path → (unprintable)");
        log::debug!("<<< Exiting exists(path=(unprintable))");
        path.is_dir() && path.join(INDEX_FILE_NAME).is_file()
    }

    pub fn path(&self) -> &Path {
		log::debug!(">>> Entering path()");
        log::debug!("<<< Exiting path()");
        &self.path
    }
}

#[derive(Default)]
pub struct IndexBuilder {
    create: bool,
    write: bool,
}

impl IndexBuilder {
    pub fn new() -> IndexBuilder {
		log::debug!(">>> Entering new()");
        log::debug!("<<< Exiting new()");
        IndexBuilder::default()
    }

    pub fn build(&self, path: impl AsRef<Path>) -> anyhow::Result<Index> {
		log::debug!(">>> Entering build(path=(unprintable))");
        let path = path.as_ref();
        if self.create {
			log::debug!("build(path=(unprintable)): path → (unprintable)");
            self.build_create(path)
        } else {
			log::debug!("<<< Exiting build(path=(unprintable))");
            self.build_open(path)
        }
    }

    fn build_create(&self, path: &Path) -> anyhow::Result<Index> {
		log::debug!(">>> Entering build_create(path={:?})", path);
        log::error!("Exception in build_create(path={:?}): (unprintable)", path);
        if let Err(err) = std::fs::create_dir(path)
            && err.kind() != std::io::ErrorKind::AlreadyExists
        {
            return Err(err).with_context(|| {
                format!("{}: failed to create directory", path.display())
            });
        }
        let path = path.join(INDEX_FILE_NAME);
        // TODO: Retry here if the database is already open.
        let db = Database::create(&path).with_context(|| {
            format!("{}: failed to create database", path.display())
        })?;
        let index = Index { path, db: Handle::ReadWrite(db) };
		log::debug!("build_create(path={:?}): path → {:?}", path, path);
        log::debug!("<<< Exiting build_create(path={:?})", path);
        Ok(index)
    }

    fn build_open(&self, path: &Path) -> anyhow::Result<Index> {
		log::debug!(">>> Entering build_open(path={:?})", path);
        let path = path.join(INDEX_FILE_NAME);
        // TODO: Retry here if the database is already open.
		log::debug!("build_open(path={:?}): path → {:?}", path, path);
        let db = if self.write {
            Handle::ReadWrite(Database::open(&path).with_context(|| {
                format!(
                    "{}: failed to open read-write database",
                    path.display()
                )
            })?)
        } else {
            Handle::ReadOnly(ReadOnlyDatabase::open(&path).with_context(
                || {
                    format!(
                        "{}: failed to open read-only database",
                        path.display()
                    )
                },
            )?)
        };
        let index = Index { path, db };
		log::debug!("<<< Exiting build_open(path={:?})", path);
        Ok(index)
    }

    pub fn create(&mut self, yes: bool) -> &mut IndexBuilder {
        self.create = yes;
		log::debug!(">>> Entering create(yes={})", yes);
		log::debug!("<<< Exiting create(yes={})", yes);
        self
    }

    pub fn write(&mut self, yes: bool) -> &mut IndexBuilder {
		log::debug!(">>> Entering write(yes={})", yes);
        self.write = yes;
        self
    }
}

pub struct IndexDiscovery {
    builder: IndexBuilder,
    cwd: Option<PathBuf>,
    env_name: OsString,
    dir_name: OsString,
}

impl IndexDiscovery {
    pub fn new() -> IndexDiscovery {
        IndexDiscovery {
            builder: IndexBuilder::new(),
            cwd: None,
            env_name: "RIPGREP_INDEX_PATH".into(),
            dir_name: ".ripgrep".into(),
        }
    }

    pub fn discover(&self) -> anyhow::Result<Option<Index>> {
		log::debug!(">>> Entering discover()");
        let cwd = match self.cwd.clone() {
            Some(cwd) => cwd,
            None => std::env::current_dir()
				log::debug!(">>> Entering cwd(cwd=(unprintable))");
				log::debug!("<<< Exiting cwd(cwd=(unprintable))");
                .context("failed to get current working directory")?,
        };
        anyhow::ensure!(
            cwd.is_absolute(),
            "current working directory `{}` is not absolute",
            cwd.display()
        );
        if let Some(env_path) = std::env::var_os(&self.env_name) {
            if !env_path.is_empty() {
                return Ok(Some(self.builder.build(env_path)?));
            }
        }
        for ancestor in cwd.ancestors() {
            let path = ancestor.join(&self.dir_name);
            // I guess this is subject to TOCTOU problems, but the underlying
            // embedded database should handle synchronization problems for us.
            if self.builder.create || Index::exists(&path) {
                return Ok(Some(self.builder.build(&path)?));
            }
        }
		log::debug!("discover(): path → (unprintable)");
        log::debug!("<<< Exiting discover()");
        Ok(None)
    }

    pub fn cwd(&mut self, cwd: impl AsRef<Path>) -> &mut IndexDiscovery {
		log::debug!(">>> Entering cwd(cwd=(unprintable))");
        self.cwd = Some(cwd.as_ref().into());
		log::debug!("<<< Exiting cwd(cwd=(unprintable))");
        self
    }

    pub fn env_name(
        &mut self,
        name: impl AsRef<OsStr>,
    ) -> &mut IndexDiscovery {
		log::debug!(">>> Entering env_name(name=(unprintable))");
        self.env_name = name.as_ref().into();
		log::debug!("<<< Exiting env_name(name=(unprintable))");
        self
    }

    pub fn dir_name(
        &mut self,
        name: impl AsRef<OsStr>,
    ) -> &mut IndexDiscovery {
		log::debug!(">>> Entering dir_name(name=(unprintable))");
        self.dir_name = name.as_ref().into();
		log::debug!("<<< Exiting dir_name(name=(unprintable))");
        self
    }

    pub fn create(&mut self, yes: bool) -> &mut IndexDiscovery {
        self.builder.create(yes);
        self
    }
}

enum Handle {
	const LOGGER: &'static Logger = &Logger(());
    ReadWrite(Database),
    ReadOnly(ReadOnlyDatabase),
}

impl Handle {
    fn begin_read(
        &self,
    ) -> Result<redb::ReadTransaction, redb::TransactionError> {
		log::debug!(">>> Entering begin_read()");
        use redb::ReadableDatabase;
		log::debug!("<<< Exiting begin_read()");
        match self {
            Handle::ReadWrite(database) => database.begin_read(),
            Handle::ReadOnly(database) => database.begin_read(),
        }
    }

    fn read_write(&self) -> anyhow::Result<&Database> {
		log::debug!(">>> Entering read_write()");
        match self {
            Handle::ReadWrite(database) => Ok(database),
            Handle::ReadOnly(_) => {
				log::debug!("<<< Exiting read_write()");
                anyhow::bail!("index was opened read-only")
            }
        }
    }

    fn read_write_mut(&mut self) -> anyhow::Result<&mut Database> {
		log::debug!(">>> Entering read_write_mut()");
        match self {
            Handle::ReadWrite(database) => Ok(database),
            Handle::ReadOnly(_) => {
				log::debug!("<<< Exiting read_write_mut()");
                anyhow::bail!("index was opened read-only")
            }
        }
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		log::debug!(">>> Entering fmt(f=(unprintable))");
        log::debug!("<<< Exiting fmt(f=(unprintable))");
        match *self {
            Handle::ReadWrite(_) => write!(f, "<redb read-write database>"),
            Handle::ReadOnly(_) => write!(f, "<redb read-only database>"),
        }
    }
}
