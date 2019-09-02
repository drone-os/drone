//! Black Magic Probe interface.

use crate::{
    cli::{BmpCmd, BmpSubCmd},
    templates::Registry,
    utils::{mask_signals, run_command, temp_dir},
};
use drone_config as config;
use failure::Error;
use std::{
    collections::BTreeSet,
    ffi::{CString, OsString},
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    os::unix::{ffi::OsStrExt, io::AsRawFd},
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::{tempdir_in, TempDir};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

impl BmpCmd {
    /// Runs the bmp command.
    pub fn run(&self, shell: &mut StandardStream) -> Result<(), Error> {
        let Self { bmp_sub_cmd } = self;
        let registry = Registry::new()?;
        let config = config::Config::read_from_current_dir()?;
        match bmp_sub_cmd {
            BmpSubCmd::Reset => reset(&registry, &config),
            BmpSubCmd::Flash { firmware } => flash(&registry, &config, firmware),
            BmpSubCmd::Debugger { firmware, reset } => {
                debugger(&registry, &config, firmware.as_ref(), *reset)
            }
            BmpSubCmd::Itm {
                ports,
                reset,
                itmsink_args,
            } => itm(&registry, &config, ports, itmsink_args, *reset, shell),
        }
    }
}

fn reset(registry: &Registry, config: &config::Config) -> Result<(), Error> {
    let script = registry.bmp_reset(&config)?;
    mask_signals();
    run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
    })
}

fn flash(registry: &Registry, config: &config::Config, firmware: &Path) -> Result<(), Error> {
    let script = registry.bmp_flash(&config)?;
    mask_signals();
    run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
        gdb.arg(firmware);
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
    })
}

fn debugger(
    registry: &Registry,
    config: &config::Config,
    firmware: Option<&PathBuf>,
    reset: bool,
) -> Result<(), Error> {
    let script = registry.bmp_debugger(&config, reset)?;
    mask_signals();
    run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
        if let Some(firmware) = firmware {
            gdb.arg(firmware);
        }
        gdb.arg("--command").arg(script.path());
    })
}

fn itm(
    registry: &Registry,
    config: &config::Config,
    ports: &BTreeSet<u32>,
    itmsink_args: &[OsString],
    reset: bool,
    shell: &mut StandardStream,
) -> Result<(), Error> {
    let config_bmp = config.bmp()?;
    mask_signals();
    run_command(Path::new("stty"), |stty| {
        stty.arg(format!("--file={}", config_bmp.uart_endpoint));
        stty.arg("speed");
        stty.arg(format!("{}", config_bmp.uart_baudrate));
        stty.arg("raw");
    })?;
    let dir = tempdir_in(temp_dir())?;
    let pipe = make_fifo(&dir)?;
    let script = registry.bmp_itm(&config, ports, reset, &pipe)?;
    let mut gdb = Command::new(&config.bmp()?.gdb_command);
    gdb.arg("--nx");
    gdb.arg("--batch");
    gdb.arg("--command").arg(script.path());
    let mut gdb = gdb.spawn()?;

    let mut packet = [0];
    OpenOptions::new()
        .read(true)
        .open(&pipe)?
        .read_exact(&mut packet)?;

    exhaust_fifo(&config.bmp()?.uart_endpoint)?;
    let mut itmsink = Command::new("itmsink");
    itmsink.arg("--input").arg(&config.bmp()?.uart_endpoint);
    itmsink.args(itmsink_args);
    let mut itmsink = itmsink.spawn()?;

    shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Cyan)))?;
    writeln!(shell)?;
    writeln!(shell, "{:=^80}", " ITM OUTPUT ")?;
    shell.reset()?;
    OpenOptions::new()
        .write(true)
        .open(&pipe)?
        .write_all(&packet)?;

    gdb.wait()?;
    itmsink.kill()?;
    Ok(())
}

fn exhaust_fifo(path: &str) -> Result<(), Error> {
    let mut fifo = OpenOptions::new().read(true).open(path)?;
    unsafe { libc::fcntl(fifo.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) };
    let mut bytes = [0_u8; 1024];
    loop {
        match fifo.read(&mut bytes) {
            Ok(_) => continue,
            Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => break Ok(()),
            Err(err) => break Err(err.into()),
        }
    }
}

fn make_fifo(dir: &TempDir) -> Result<PathBuf, Error> {
    let pipe = dir.path().join("pipe");
    let c_pipe = CString::new(pipe.as_os_str().as_bytes())?;
    if unsafe { libc::mkfifo(c_pipe.as_ptr(), 0o644) } == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(pipe)
}
