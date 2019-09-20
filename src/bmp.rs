//! Black Magic Probe interface.

use crate::{
    cli::{BmpCmd, BmpFlashCmd, BmpGdbCmd, BmpItmCmd, BmpResetCmd, BmpSubCmd},
    templates::Registry,
    utils::{mask_signals, run_command, temp_dir},
};
use drone_config as config;
use failure::{format_err, Error};
use std::{
    ffi::CString,
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    os::unix::{ffi::OsStrExt, io::AsRawFd},
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::{tempdir_in, TempDir};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

impl BmpCmd {
    /// Runs the `bmp` command.
    pub fn run(&self, shell: &mut StandardStream) -> Result<(), Error> {
        let Self { bmp_sub_cmd } = self;
        let registry = Registry::new()?;
        let config = config::Config::read_from_current_dir()?;
        match bmp_sub_cmd {
            BmpSubCmd::Reset(cmd) => cmd.run(&registry, &config),
            BmpSubCmd::Flash(cmd) => cmd.run(&registry, &config),
            BmpSubCmd::Gdb(cmd) => cmd.run(&registry, &config),
            BmpSubCmd::Itm(cmd) => cmd.run(&registry, &config, shell),
        }
    }
}

impl BmpResetCmd {
    /// Runs the `bmp reset` command.
    pub fn run(&self, registry: &Registry, config: &config::Config) -> Result<(), Error> {
        let Self {} = self;
        let script = registry.bmp_reset(&config)?;
        mask_signals();
        run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
            gdb.arg("--nx");
            gdb.arg("--batch");
            gdb.arg("--command").arg(script.path());
        })
    }
}

impl BmpFlashCmd {
    /// Runs the `bmp flash` command.
    pub fn run(&self, registry: &Registry, config: &config::Config) -> Result<(), Error> {
        let Self { firmware } = self;
        let script = registry.bmp_flash(&config)?;
        mask_signals();
        run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
            gdb.arg(firmware);
            gdb.arg("--nx");
            gdb.arg("--batch");
            gdb.arg("--command").arg(script.path());
        })
    }
}

impl BmpGdbCmd {
    /// Runs the `bmp gdb` command.
    pub fn run(&self, registry: &Registry, config: &config::Config) -> Result<(), Error> {
        let Self { firmware, reset } = self;
        let script = registry.bmp_gdb(&config, *reset)?;
        mask_signals();
        run_command(Path::new(&config.bmp()?.gdb_command), |gdb| {
            if let Some(firmware) = firmware {
                gdb.arg(firmware);
            }
            gdb.arg("--command").arg(script.path());
        })
    }
}

impl BmpItmCmd {
    /// Runs the `bmp itm` command.
    pub fn run(
        &self,
        registry: &Registry,
        config: &config::Config,
        shell: &mut StandardStream,
    ) -> Result<(), Error> {
        let Self {
            ports,
            reset,
            itmsink_args,
        } = self;
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
        let script = registry.bmp_itm(&config, ports, *reset, &pipe)?;
        let gdb_command = &config.bmp()?.gdb_command;
        let mut gdb = Command::new(gdb_command);
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        let mut gdb = gdb
            .spawn()
            .map_err(|err| format_err!("`{}` command failed to start: {}", gdb_command, err))?;

        let mut packet = [0];
        OpenOptions::new()
            .read(true)
            .open(&pipe)?
            .read_exact(&mut packet)?;

        exhaust_fifo(&config.bmp()?.uart_endpoint)?;
        let mut itmsink = Command::new("itmsink");
        itmsink.arg("--input").arg(&config.bmp()?.uart_endpoint);
        itmsink.args(itmsink_args);
        let mut itmsink = itmsink
            .spawn()
            .map_err(|err| format_err!("`itmsink` command failed to start: {}", err))?;

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
