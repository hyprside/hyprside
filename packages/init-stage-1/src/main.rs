use std::{fs, io, path::PathBuf};

use crate::{
    kernel_args::Args,
    mount::{MountFlags, mount},
    proc::Proc,
};
mod kernel_args;
mod mount;
mod proc;

fn main() {
    let proc = Proc::new().expect("Failed to mount /proc");
    let args = Args::parse(&proc).expect("Failed to parse args");
    mount_dev().expect("Failed to mount /dev");
    mount_sys().expect("Failed to mount /sys");
    let system_partition_path =
        mount_system_partition(&args).expect("Failed to mount system partition");
    dbg!(&args, &system_partition_path);
    ls(system_partition_path);
}

fn ls(path: impl Into<PathBuf>) {
    let path = path.into();
    println!("Listing contents of {}", path.display());
    match fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(e) => {
                        let name = e.file_name().to_string_lossy().into_owned();
                        let ftype = match e.file_type() {
                            Ok(t) if t.is_dir() => "dir",
                            Ok(t) if t.is_file() => "file",
                            Ok(_) => "other",
                            Err(_) => "unknown",
                        };
                        println!(" - {} ({})", name, ftype);
                    }
                    Err(err) => {
                        println!("Failed to read entry: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Failed to read directory {}: {}", path.display(), err);
        }
    }
}
fn mount_dev() -> io::Result<()> {
    const DEV_PATH: &str = "/dev";
    fs::create_dir_all(DEV_PATH)?;
    mount("devtmpfs", DEV_PATH, "devtmpfs", MountFlags::empty())?;
    Ok(())
}
fn mount_sys() -> io::Result<()> {
    const SYS_PATH: &str = "/sys";
    fs::create_dir_all(SYS_PATH)?;
    mount("sysfs", SYS_PATH, "sysfs", MountFlags::empty())?;
    Ok(())
}
fn mount_system_partition(args: &Args) -> io::Result<PathBuf> {
    const SYSTEMP_PATH: &str = "/systemp";
    fs::create_dir(SYSTEMP_PATH)?;
    mount(
        &dbg!(mount::resolve_uuid(
            args.system_data_partition.trim_start_matches("UUID=")
        )?),
        SYSTEMP_PATH,
        "btrfs",
        MountFlags::empty(),
    )?;
    Ok(SYSTEMP_PATH.into())
}
