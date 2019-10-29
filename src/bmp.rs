//! Black Magic Probe interface.

use crate::{
    cli::{BmpCmd, BmpFlashCmd, BmpGdbCmd, BmpItmCmd, BmpResetCmd, BmpSubCmd},
    templates::Registry,
    utils::{block_with_signals, finally, register_signals, run_command, spawn_command, temp_dir},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{
    ffi::CString,
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    os::unix::{ffi::OsStrExt, io::AsRawFd},
    path::PathBuf,
    process::Command,
};
use tempfile::{tempdir_in, TempDir};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

impl BmpCmd {
    /// Runs the `bmp` command.
    pub fn run(&self, shell: &mut StandardStream) -> Result<()> {
        let Self { bmp_sub_cmd } = self;
        let signals = register_signals()?;
        let registry = Registry::new()?;
        let config = config::Config::read_from_current_dir()?;
        match bmp_sub_cmd {
            BmpSubCmd::Reset(cmd) => cmd.run(&signals, &registry, &config),
            BmpSubCmd::Flash(cmd) => cmd.run(&signals, &registry, &config),
            BmpSubCmd::Gdb(cmd) => cmd.run(&signals, &registry, &config),
            BmpSubCmd::Itm(cmd) => cmd.run(&signals, &registry, &config, shell),
        }
    }
}

impl BmpResetCmd {
    /// Runs the `bmp reset` command.
    pub fn run(
        &self,
        signals: &Signals,
        registry: &Registry,
        config: &config::Config,
    ) -> Result<()> {
        let Self {} = self;
        let script = registry.bmp_reset(&config)?;
        let mut gdb = Command::new(&config.bmp()?.gdb_command);
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, || run_command(gdb))
    }
}

impl BmpFlashCmd {
    /// Runs the `bmp flash` command.
    pub fn run(
        &self,
        signals: &Signals,
        registry: &Registry,
        config: &config::Config,
    ) -> Result<()> {
        let Self { firmware } = self;
        let script = registry.bmp_flash(&config)?;
        let mut gdb = Command::new(&config.bmp()?.gdb_command);
        gdb.arg(firmware);
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, || run_command(gdb))
    }
}

impl BmpGdbCmd {
    /// Runs the `bmp gdb` command.
    pub fn run(
        &self,
        signals: &Signals,
        registry: &Registry,
        config: &config::Config,
    ) -> Result<()> {
        let Self { firmware, reset } = self;
        let script = registry.bmp_gdb(&config, *reset)?;
        let mut gdb = Command::new(&config.bmp()?.gdb_command);
        if let Some(firmware) = firmware {
            gdb.arg(firmware);
        }
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, || run_command(gdb))
    }
}

impl BmpItmCmd {
    /// Runs the `bmp itm` command.
    pub fn run(
        &self,
        signals: &Signals,
        registry: &Registry,
        config: &config::Config,
        shell: &mut StandardStream,
    ) -> Result<()> {
        let Self {
            ports,
            firmware,
            reset,
            itmsink_args,
        } = self;
        let config_bmp = config.bmp()?;

        let mut stty = Command::new("stty");
        stty.arg(format!("--file={}", config_bmp.uart_endpoint));
        stty.arg("speed");
        stty.arg(format!("{}", config_bmp.uart_baudrate));
        stty.arg("raw");
        block_with_signals(&signals, || run_command(stty))?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir)?;
        let script = registry.bmp_itm(&config, ports, *reset, &pipe)?;
        let mut gdb = Command::new(&config.bmp()?.gdb_command);
        if let Some(firmware) = firmware {
            gdb.arg(firmware);
        }
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        let mut gdb = spawn_command(gdb)?;

        let (pipe, packet) = block_with_signals(&signals, move || {
            let mut packet = [0];
            OpenOptions::new()
                .read(true)
                .open(&pipe)?
                .read_exact(&mut packet)?;
            Ok((pipe, packet))
        })?;

        exhaust_fifo(&config.bmp()?.uart_endpoint)?;
        let mut itmsink = Command::new("itmsink");
        itmsink.arg("--input").arg(&config.bmp()?.uart_endpoint);
        itmsink.args(itmsink_args);
        let mut itmsink = spawn_command(itmsink)?;
        let _itmsink = finally(|| itmsink.kill().expect("itmsink wasn't running"));

        block_with_signals(&signals, move || {
            OpenOptions::new()
                .write(true)
                .open(&pipe)?
                .write_all(&packet)?;
            Ok(())
        })?;

        shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Cyan)))?;
        writeln!(shell)?;
        writeln!(shell, "{:=^80}", " ITM OUTPUT ")?;
        shell.reset()?;

        block_with_signals(&signals, move || {
            gdb.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

fn exhaust_fifo(path: &str) -> Result<()> {
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

fn make_fifo(dir: &TempDir) -> Result<PathBuf> {
    let pipe = dir.path().join("pipe");
    let c_pipe = CString::new(pipe.as_os_str().as_bytes())?;
    if unsafe { libc::mkfifo(c_pipe.as_ptr(), 0o644) } == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(pipe)
}
