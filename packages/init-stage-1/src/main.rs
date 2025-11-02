mod kernel_args;
mod loopdev;
mod mount;
mod proc;

use std::{env, ffi::CString, fs, io, path::PathBuf, time::Duration};

use crate::{
	kernel_args::Args,
	loopdev::LoopDevice,
	mount::{mount, resolve_uuid, umount, MountFlags},
	proc::Proc,
};

fn main() -> io::Result<()> {
	println!("------------------------------ Init stage 1 ------------------------------");
	let proc = Proc::new()?;
	let args = Args::parse(&proc)?;

	mount_dev()?;
	mount_sys()?;

	let system_partition_path = mount_system_partition(&args)?;
	dbg!(&args, &system_partition_path);
	ls(&system_partition_path)?;

	let squashfs_path = system_partition_path.join("system.squashfs");

	// cria o loop device (associa o ficheiro -> /dev/loopX) e mantém o objecto vivo
	let loop_device = Box::leak(Box::new(LoopDevice::new(&squashfs_path)?));
	println!("Loop device created: {}", loop_device.path().display());

	// monta o loop device em /system
	let system_root_path = mount_system_image(loop_device.path())?;
	ls(&system_root_path)?;
	std::thread::sleep(Duration::from_secs(5));

	switch_root(system_root_path)?;
	ls("/")?;
	std::thread::sleep(Duration::from_secs(5));

	Ok(())
}

fn ls<P: AsRef<std::path::Path>>(path: P) -> io::Result<()> {
	let path = path.as_ref();
	println!("Listing contents of {}", path.display());

	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let ty = entry.file_type()?;
		let kind = if ty.is_dir() {
			"dir"
		} else if ty.is_file() {
			"file"
		} else {
			"other"
		};
		println!(" - {} ({})", entry.file_name().to_string_lossy(), kind);
	}
	Ok(())
}

fn mount_dev() -> io::Result<()> {
	fs::create_dir_all("/dev")?;
	mount("devtmpfs", "/dev", "devtmpfs", MountFlags::empty())
}

fn mount_sys() -> io::Result<()> {
	fs::create_dir_all("/sys")?;
	mount("sysfs", "/sys", "sysfs", MountFlags::empty())
}

fn mount_system_partition(args: &Args) -> io::Result<PathBuf> {
	const SYSTEMP_PATH: &str = "/systemp";
	fs::create_dir_all(SYSTEMP_PATH)?;
	let dev = resolve_uuid(args.system_data_partition.trim_start_matches("UUID="))?;
	mount(&dev, SYSTEMP_PATH, "btrfs", MountFlags::empty())?;
	Ok(PathBuf::from(SYSTEMP_PATH))
}

fn mount_system_image(device: impl Into<PathBuf>) -> io::Result<PathBuf> {
	let device = device.into();
	const SYSTEM_PATH: &str = "/system";
	fs::create_dir_all(SYSTEM_PATH)?;
	mount(
		device.as_os_str().to_str().unwrap(),
		SYSTEM_PATH,
		"squashfs",
		MountFlags::RDONLY | MountFlags::NODEV | MountFlags::NOEXEC,
	)?;
	println!("✅ Mounted {} to {}", device.display(), SYSTEM_PATH);
	Ok(PathBuf::from(SYSTEM_PATH))
}
pub fn switch_root(new_root: impl Into<PathBuf>) -> io::Result<()> {
	let new_root = new_root.into();
	println!("switch_root: switching root to {}", new_root.display());

	// 1️⃣ Ensure the new root exists and is accessible
	if !new_root.exists() {
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			format!("new root {} does not exist", new_root.display()),
		));
	}

	// 2️⃣ Try to move or unmount the usual mountpoints
	let mounts = ["/dev", "/proc", "/sys", "/run"];
	for &mnt in mounts.iter() {
		let new_target = new_root.join(mnt.strip_prefix('/').unwrap_or(mnt));

		println!("switch_root: moving {} -> {}", mnt, new_target.display());
		match mount(mnt, &new_target.display().to_string(), None, MountFlags::MOVE) {
			Ok(_) => {}
			Err(e) => {
				eprintln!(
					"switch_root: failed to move {}, detaching instead: {}",
					mnt, e
				);
				let _ = umount(mnt);
			}
		}
	}

	// 3️⃣ Change current directory to the new root
	env::set_current_dir(&new_root)?;
	println!("switch_root: changed directory to {}", new_root.display());

	// 4️⃣ Move new_root over "/"
	mount(&new_root.display().to_string(), "/", None, MountFlags::MOVE)?;
	println!("switch_root: new root mounted over /");

	// 5️⃣ Replace process root (chroot("."))
	let dot = CString::new(".").unwrap();
	let res = unsafe { libc::chroot(dot.as_ptr()) };
	if res != 0 {
		return Err(io::Error::last_os_error());
	}

	env::set_current_dir("/")?;
	println!("switch_root: successfully chrooted into new root");

	Ok(())
}
