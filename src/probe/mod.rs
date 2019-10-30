//! Debug probe interface.

pub mod bmp;
pub mod openocd;

use crate::{
    cli::{ProbeCmd, ProbeSubCmd},
    templates::Registry,
    utils::{block_with_signals, register_signals, run_command},
};
use anyhow::{anyhow, bail, Result};
use drone_config as config;
use signal_hook::iterator::Signals;
use std::process::Command;
use termcolor::StandardStream;

impl ProbeCmd {
    /// Runs the `drone probe` command.
    #[allow(clippy::too_many_lines)]
    pub fn run(&self, shell: &mut StandardStream) -> Result<()> {
        let Self { probe_sub_cmd } = self;
        let signals = register_signals()?;
        let registry = Registry::new()?;
        let config = &config::Config::read_from_current_dir()?;
        let config_probe = config
            .probe
            .as_ref()
            .ok_or_else(|| anyhow!("Missing `probe` section in `{}`", config::CONFIG_NAME))?;
        match probe_sub_cmd {
            ProbeSubCmd::Reset(cmd) => {
                if config_probe.bmp.is_some() {
                    return bmp::ResetCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                    }
                    .run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::ResetCmd {
                        cmd,
                        signals,
                        registry,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
            ProbeSubCmd::Flash(cmd) => {
                if config_probe.bmp.is_some() {
                    return bmp::FlashCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                    }
                    .run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::FlashCmd {
                        cmd,
                        signals,
                        registry,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
            ProbeSubCmd::Gdb(cmd) => {
                if config_probe.bmp.is_some() {
                    return bmp::GdbCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                    }
                    .run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::GdbCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
            ProbeSubCmd::Itm(cmd) => {
                let config_probe_itm = config_probe.itm.as_ref().ok_or_else(|| {
                    anyhow!("Missing `probe.itm` section in `{}`", config::CONFIG_NAME)
                })?;
                if config_probe.bmp.is_some() {
                    return bmp::ItmCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_itm,
                        shell,
                    }
                    .run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::ItmCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe_itm,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
        }
        bail!(
            "Suitable debug probe configuration is not found in `{}`",
            config::CONFIG_NAME
        );
    }
}

/// Configures the endpoint with `stty` command.
pub fn setup_uart_endpoint(signals: &Signals, endpoint: &str, baud_rate: u32) -> Result<()> {
    let mut stty = Command::new("stty");
    stty.arg(format!("--file={}", endpoint));
    stty.arg("speed");
    stty.arg(format!("{}", baud_rate));
    stty.arg("raw");
    block_with_signals(signals, true, || run_command(stty))
}
