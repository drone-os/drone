use super::{bmp, jlink, openocd, ProbeConfig};
use crate::{cli::ProbeLogCmd, templates::Registry};
use anyhow::{anyhow, Error, Result};
use drone_config as config;
use serde::{Deserialize, Serialize};
use signal_hook::iterator::Signals;
use std::convert::TryFrom;
use termcolor::StandardStream;

/// An `enum` of all supported debug loggers.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Log {
    /// Listening to ARM® SWO through the debug probe.
    SwoProbe,
    /// Listening to ARM® SWO through the USB-serial adapter.
    SwoSerial,
    /// Listening to Drone Serial Output through the USB-serial adapter.
    DsoSerial,
}

enum ProbeLogConfig<'a> {
    Swo(&'a config::ProbeSwo),
    Dso(&'a config::ProbeDso),
}

impl<'a> TryFrom<&'a config::Probe> for ProbeLogConfig<'a> {
    type Error = Error;

    fn try_from(config_probe: &'a config::Probe) -> Result<Self> {
        if let Some(config_probe_swo) = &config_probe.swo {
            Ok(Self::Swo(config_probe_swo))
        } else if let Some(config_probe_dso) = &config_probe.dso {
            Ok(Self::Dso(config_probe_dso))
        } else {
            Err(anyhow!(
                "Missing one of `probe.swo`, `probe.dso` sections in `{}`",
                config::CONFIG_NAME
            ))
        }
    }
}

pub(super) fn run(
    cmd: &ProbeLogCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: &config::Config,
    config_probe: &config::Probe,
    probe_config: &ProbeConfig<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    match (probe_config, ProbeLogConfig::try_from(config_probe)?) {
        (ProbeConfig::Bmp(_), ProbeLogConfig::Swo(config_probe_swo)) => {
            bmp::LogSwoCmd { cmd, signals, registry, config, config_probe, config_probe_swo, shell }
                .run()
        }
        (ProbeConfig::Jlink(_), ProbeLogConfig::Swo(_)) => {
            unimplemented!("SWO capture with J-Link");
        }
        (ProbeConfig::Openocd(config_probe_openocd), ProbeLogConfig::Swo(config_probe_swo)) => {
            openocd::LogSwoCmd {
                cmd,
                signals,
                registry,
                config,
                config_probe,
                config_probe_openocd,
                config_probe_swo,
                shell,
            }
            .run()
        }
        (ProbeConfig::Jlink(config_probe_jlink), ProbeLogConfig::Dso(config_probe_dso)) => {
            jlink::LogDsoCmd {
                cmd,
                signals,
                registry,
                config,
                config_probe,
                config_probe_jlink,
                config_probe_dso,
                shell,
            }
            .run()
        }
        (ProbeConfig::Bmp(_), ProbeLogConfig::Dso(_))
        | (ProbeConfig::Openocd(_), ProbeLogConfig::Dso(_)) => todo!(),
    }
}
