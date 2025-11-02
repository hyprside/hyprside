use std::fs;
use std::io;
use std::marker::PhantomData;

use crate::mount::umount;
use crate::mount::{MountFlags, mount};

pub struct Proc {
	_private: PhantomData<()>,
}

impl Proc {
	/// Mounts /proc and returns a managed handle.
	pub fn new() -> io::Result<Self> {
		fs::create_dir_all("/proc")?;
		mount("proc", "/proc", "proc", MountFlags::empty())?;
		Ok(Self {
			_private: PhantomData,
		})
	}

	/// Reads /proc/cmdline and returns the contents as a string.
	pub fn read_cmdline(&self) -> io::Result<String> {
		self.read_file("cmdline").map(|s| s.trim().to_string())
	}

	/// Reads any arbitrary file under /proc.
	pub fn read_file(&self, path: &str) -> io::Result<String> {
		let full_path = format!("/proc/{path}");
		fs::read_to_string(full_path)
	}
}
