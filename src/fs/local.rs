use super::*;

#[cfg(unix)] pub use localhost::LocalHost;
#[cfg(windows)] pub use localhost::LocalHost;

#[cfg(unix)]
mod localhost {
    pub struct LocalHost;
    use super::*;
    use std::os::unix::fs::MetadataExt;

    impl File {
        fn from_host(path: PathBuf) -> io::Result<Self> {
            let name = Self::name(&path);
            let ext = Self::ext(&path);
            let metadata = std::fs::metadata(&path)?;

            let file_type = if metadata.is_dir() {
                FileType::Dir
            } else if metadata.is_file() {
                FileType::File
            } else if metadata.is_symlink() {
                FileType::SymLink
            } else {
                FileType::Other
            };

            let accessed = metadata.accessed().ok().map(Into::into);
            let created = metadata.created().ok().map(Into::into);
            let modified = metadata.modified().ok().map(Into::into);
            let len = metadata.len();
            let permissions = Permissions::from_mode(metadata.mode());
            let metadata = Metadata {
                accessed,
                created,
                modified,
                len,
                permissions,
            };

            Ok(Self { path, name, ext, metadata, file_type })
        }
    }

    impl Host for LocalHost {
        fn read_dir(&mut self, dir: &Path) -> io::Result<Vec<File>> {
            let mut out = vec![];
            out.push(File::new_dot_dot(dir.join("..").canonicalize()?)?);
            for dir_entry in std::fs::read_dir(dir)? {
                let dir_entry = dir_entry?;
                match File::from_host(dir_entry.path()) {
                    Ok(f) => { out.push(f); }
                    Err(e) => { log::error!("Fail to load path: {:?}", e); }
                }
            }
            Ok(out)
        }

        fn create_dir(&mut self, path: &Path) -> io::Result<()> {
            std::fs::create_dir(path)
        }

        fn create_file(&mut self, path: &Path) -> io::Result<()> {
            let _ = std::fs::File::create(path)?;
            Ok(())
        }

        fn try_exists(&mut self, path: &Path) -> io::Result<bool> {
            Ok(path.exists())
        }
    }
}

#[cfg(windows)]
mod localhost {

}

