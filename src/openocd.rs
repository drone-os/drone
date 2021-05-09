//! OpenOCD integration.

use anyhow::{anyhow, Result};
use drone_openocd_sys::{openocd_main, stderr, stdout, SCRIPTS_FINGERPRINT, SCRIPTS_TAR_BZ2};
use libc::{setvbuf, _IONBF};
use std::{
    convert::TryInto,
    ffi::{CString, OsStr},
    fs,
    io::prelude::*,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::{exit, Command, Stdio},
    ptr,
};

const SCRIPTS_PATH: &str = "/tmp/drone-openocd";

/// Runs OpenOCD with given arguments. This function normally never returns.
pub fn exit_with_openocd<'a, T: IntoIterator<Item = &'a OsStr>>(args: T) -> Result<!> {
    unpack_scripts()?;
    let mut prefix_args =
        vec![OsStr::new("drone-openocd"), OsStr::new("--search"), OsStr::new(SCRIPTS_PATH)];
    prefix_args.extend(args);
    let full_args = prefix_args
        .into_iter()
        .map(|arg| Ok(CString::new(arg.as_bytes())?.into_raw()))
        .collect::<Result<Vec<_>>>()?
        .leak();
    let ret = unsafe {
        setvbuf(stdout.cast(), ptr::null_mut(), _IONBF, 0);
        setvbuf(stderr.cast(), ptr::null_mut(), _IONBF, 0);
        openocd_main(full_args.len().try_into().unwrap(), full_args.as_mut_ptr())
    };
    exit(ret);
}

/// Unpacks the TCL scripts to a temporary location. This function maintains a
/// fingerprint file for not unpacking the scripts on each invocation.
pub fn unpack_scripts() -> Result<()> {
    let root_path = PathBuf::from(SCRIPTS_PATH);
    let fingerprint_path = root_path.join(".fingerprint");
    if root_path.exists() {
        if let Ok(fingerprint) = fs::read(&fingerprint_path) {
            if fingerprint == SCRIPTS_FINGERPRINT {
                log::info!("OpenOCD scripts are up-to-date");
                return Ok(());
            }
        }
        log::info!("OpenOCD scripts are outdated");
        fs::remove_dir_all(&root_path)?;
    }
    log::info!("Unpacking OpenOCD scripts");
    fs::create_dir_all(&root_path)?;
    let mut tar = Command::new("tar")
        .arg("--extract")
        .arg("--bzip2")
        .arg("--file=-")
        .stdin(Stdio::piped())
        .current_dir(&root_path)
        .spawn()?;
    tar.stdin.take().unwrap().write_all(SCRIPTS_TAR_BZ2)?;
    tar.wait()?.success().then_some(()).ok_or_else(|| anyhow!("tar failed"))?;
    fs::write(&fingerprint_path, SCRIPTS_FINGERPRINT)?;
    Ok(())
}
