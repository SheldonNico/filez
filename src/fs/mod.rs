// ref to: https://github.com/ogham/exa/tree/master/src/fs
pub(crate) mod local;

use std::io;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

pub use local::LocalHost;

pub trait Host {
    fn read_dir(&mut self, dir: &Path) -> io::Result<Vec<File>>;
    fn try_exists(&mut self, path: &Path) -> io::Result<bool>;

    fn create_dir(&mut self, path: &Path) -> io::Result<()>;
    fn create_file(&mut self, path: &Path) -> io::Result<()>;
    // fn remove(&mut self, path: &Path);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    File,
    SymLink,
    Dir,

    Other,
    DotDot,
}

pub struct File {
    pub path: PathBuf,
    pub name: String,
    pub ext: Option<String>,
    pub metadata: Metadata,

    pub file_type: FileType,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub accessed: Option<DateTime<Local>>,
    pub created: Option<DateTime<Local>>,
    pub modified: Option<DateTime<Local>>,
    pub len: u64,
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub user_read:      bool,
    pub user_write:     bool,
    pub user_execute:   bool,

    pub group_read:     bool,
    pub group_write:    bool,
    pub group_execute:  bool,

    pub other_read:     bool,
    pub other_write:    bool,
    pub other_execute:  bool,
}

impl File {
    pub fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Dir)
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self.file_type, FileType::SymLink)
    }

    pub fn is_dot_dot(&self) -> bool {
        matches!(self.file_type, FileType::DotDot)
    }

    fn new_dot_dot(path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            path,
            name: "..".to_string(),
            ext: None,
            metadata: Metadata {
                accessed: None,
                modified: None,
                created: None,
                len: 0,
                permissions: Permissions::from_mode(0),
            },
            file_type: FileType::DotDot
        })
    }

    // check exa code
    fn name(path: &Path) -> String {
        if let Some(back) = path.components().next_back() {
            back.as_os_str().to_string_lossy().to_string()
        } else {
            log::error!("Path {:?} has no last component", path);
            path.display().to_string()
        }
    }

    // check exa
    fn ext(path: &Path) -> Option<String> {
        let name = path.file_name().map(|f| f.to_string_lossy().to_string())?;
        let idx = name.rfind(".")?;
        Some(name[idx+1..].to_ascii_lowercase())
    }
}

impl Permissions {
    // check exa
    fn from_mode(bits: u32) -> Self {
        let has_bit = |bit| bits & bit == bit;

        Permissions {
            user_read:      has_bit(modes::USER_READ),
            user_write:     has_bit(modes::USER_WRITE),
            user_execute:   has_bit(modes::USER_EXECUTE),

            group_read:     has_bit(modes::GROUP_READ),
            group_write:    has_bit(modes::GROUP_WRITE),
            group_execute:  has_bit(modes::GROUP_EXECUTE),

            other_read:     has_bit(modes::OTHER_READ),
            other_write:    has_bit(modes::OTHER_WRITE),
            other_execute:  has_bit(modes::OTHER_EXECUTE),

            // sticky:         has_bit(modes::STICKY),
            // setgid:         has_bit(modes::SETGID),
            // setuid:         has_bit(modes::SETUID),
        }
    }
}

/// More readable aliases for the permission bits exposed by libc.
#[allow(trivial_numeric_casts)]
mod modes {

    // The `libc::mode_t` type’s actual type varies, but the value returned
    // from `metadata.permissions().mode()` is always `u32`.
    pub type Mode = u32;

    pub const USER_READ: Mode     = libc::S_IRUSR as Mode;
    pub const USER_WRITE: Mode    = libc::S_IWUSR as Mode;
    pub const USER_EXECUTE: Mode  = libc::S_IXUSR as Mode;

    pub const GROUP_READ: Mode    = libc::S_IRGRP as Mode;
    pub const GROUP_WRITE: Mode   = libc::S_IWGRP as Mode;
    pub const GROUP_EXECUTE: Mode = libc::S_IXGRP as Mode;

    pub const OTHER_READ: Mode    = libc::S_IROTH as Mode;
    pub const OTHER_WRITE: Mode   = libc::S_IWOTH as Mode;
    pub const OTHER_EXECUTE: Mode = libc::S_IXOTH as Mode;

    pub const STICKY: Mode        = libc::S_ISVTX as Mode;
    pub const SETGID: Mode        = libc::S_ISGID as Mode;
    pub const SETUID: Mode        = libc::S_ISUID as Mode;
}
