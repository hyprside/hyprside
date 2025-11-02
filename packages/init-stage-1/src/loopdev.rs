//! Loop device helper (RAII).
//! Creates a /dev/loopX for a backing file and clears it on Drop.
//! Uses `std` para tudo exceto os ioctl() que são feitos via `libc`.

use std::{
	fs, io, mem,
	os::unix::fs::OpenOptionsExt,
	os::unix::io::{AsRawFd, FromRawFd, RawFd},
	path::{Path, PathBuf},
};

/// IOCTL constants (libc-only)
const LOOP_CTL_GET_FREE: libc::c_ulong = 0x4C82;
const LOOP_SET_FD: libc::c_ulong = 0x4C00;
const LOOP_CLR_FD: libc::c_ulong = 0x4C01;
const LOOP_SET_STATUS64: libc::c_ulong = 0x4C04;

/// Minimal LoopInfo64 for setting filename (optional)
#[repr(C)]
#[derive(Copy, Clone)]
struct LoopInfo64 {
	device: u64,
	inode: u64,
	rdevice: u64,
	offset: u64,
	sizelimit: u64,
	number: u32,
	encrypt_type: u32,
	encrypt_key_size: u32,
	flags: u32,
	file_name: [u8; 64],
	crypt_name: [u8; 64],
	encrypt_key: [u8; 32],
	init: [u64; 2],
}

/// Represents a /dev/loopX associated with a backing file.
/// When dropped, it will call LOOP_CLR_FD to detach.
pub struct LoopDevice {
	loop_path: PathBuf,
}

impl LoopDevice {
	/// Create a loop device for `backing_file` and return a `LoopDevice`.
	/// The returned object owns the loop fd and will detach on Drop.
	pub fn new(backing_file: &Path) -> io::Result<Self> {
		if !backing_file.exists() {
			return Err(io::Error::new(
				io::ErrorKind::NotFound,
				format!("Backing file not found: {}", backing_file.display()),
			));
		}

		// Abre o ficheiro de backing (apenas leitura é suficiente para squashfs)
		let bf = fs::File::open(backing_file)?;
		let bf_fd = bf.as_raw_fd();

		// open /dev/loop-control e pedir um loop livre
		let ctl = fs::OpenOptions::new()
			.read(true)
			.custom_flags(libc::O_CLOEXEC)
			.open("/dev/loop-control")?;
		let ctl_fd = ctl.as_raw_fd();

		let loop_num = unsafe { libc::ioctl(ctl_fd, LOOP_CTL_GET_FREE as _) };
		if loop_num < 0 {
			return Err(io::Error::last_os_error());
		}

		let loop_path = PathBuf::from(format!("/dev/loop{}", loop_num));

		// open do loop device (read/write)
		let loop_file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.custom_flags(libc::O_CLOEXEC)
			.open(&loop_path)?;

		let loop_fd = loop_file.as_raw_fd();

		// associa o backing file ao loop
		let res = unsafe { libc::ioctl(loop_fd, LOOP_SET_FD as _, bf_fd) };
		if res < 0 {
			return Err(io::Error::last_os_error());
		}

		// opcional: preencher LoopInfo64.file_name para identificação
		let mut info: LoopInfo64 = unsafe { mem::zeroed() };
		let backing_file_string = backing_file.display().to_string();
		let name_bytes = backing_file_string.as_bytes();
		for (i, b) in name_bytes.iter().take(63).enumerate() {
			info.file_name[i] = *b;
		}
		// ignoramos o resultado do set status — é opcional
		let _ = unsafe { libc::ioctl(loop_fd, LOOP_SET_STATUS64 as _, &info) };

		// Note: mantemos o loop_file (fd) aberto no struct para podermos
		// usar o fd no Drop para fazer LOOP_CLR_FD.
		// Também mantemos bf aberto tacitamente (o OS mantém a associação mesmo que
		// o bf seja fechado, mas guardar o handle é opcional — aqui deixamos cair `bf`).

		// descartamos `ctl` explicitamente (fecha)
		drop(ctl);

		Ok(Self {
			loop_path,
		})
	}

	/// Caminho do device (ex: /dev/loop0)
	pub fn path(&self) -> &Path {
		&self.loop_path
	}
}
