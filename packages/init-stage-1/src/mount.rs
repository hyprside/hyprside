use libc::{MNT_DETACH, mount as sys_mount, umount2};
use std::io;
use std::{ffi::CString, fs};

bitflags::bitflags! {
    /// Safe Rust representation of the MS_* mount flags.
    #[derive(Debug, Clone, Copy)]
    pub struct MountFlags: libc::c_ulong {
        const RDONLY     = libc::MS_RDONLY;
        const NOSUID     = libc::MS_NOSUID;
        const NODEV      = libc::MS_NODEV;
        const NOEXEC     = libc::MS_NOEXEC;
        const SYNCHRONOUS= libc::MS_SYNCHRONOUS;
        const REMOUNT    = libc::MS_REMOUNT;
        const MANDLOCK   = libc::MS_MANDLOCK;
        const DIRSYNC    = libc::MS_DIRSYNC;
        const NOATIME    = libc::MS_NOATIME;
        const NODIRATIME = libc::MS_NODIRATIME;
        const RELATIME   = libc::MS_RELATIME;
    }
}

/// Safe wrapper around the `mount(2)` syscall.
pub fn mount(source: &str, target: &str, fstype: &str, flags: MountFlags) -> io::Result<()> {
    let source_c = CString::new(source)?;
    let target_c = CString::new(target)?;
    let fstype_c = CString::new(fstype)?;

    let res = unsafe {
        sys_mount(
            source_c.as_ptr(),
            target_c.as_ptr(),
            fstype_c.as_ptr(),
            flags.bits(),
            std::ptr::null(),
        )
    };

    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Safe wrapper for umount2(target, MNT_DETACH)
pub fn umount(target: &str) -> io::Result<()> {
    let target_c = CString::new(target)?;
    let res = unsafe { umount2(target_c.as_ptr(), MNT_DETACH) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn resolve_uuid(uuid: &str) -> io::Result<String> {
    fs::read_dir("/sys/class/block")?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = dbg!(entry.path().join("uevent"));
            dbg!(fs::read_to_string(&path).ok())
        })
        .find_map(|contents| {
            let (mut devname, mut partuuid) = (None, None);

            for line in contents.lines() {
                if let Some(v) = line.strip_prefix("DEVNAME=") {
                    devname = Some(v.to_string());
                } else if let Some(v) = line.strip_prefix("PARTUUID=") {
                    partuuid = Some(v.to_string());
                }

                // 🔁 se já encontrou os dois, não há razão pra continuar
                if devname.is_some() && partuuid.is_some() {
                    break;
                }
            }

            match (devname, partuuid) {
                (Some(dev), Some(id)) if id == uuid => Some(format!("/dev/{}", dev)),
                _ => None,
            }
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("UUID {} not found", uuid)))
}
