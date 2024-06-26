use crate::config::{
    transfer::send::{SendArgs, SendCommand, SendIpArgs},
    Config,
};
#[cfg(feature = "mdns")]
use crate::{config::transfer::send::SendMdnsArgs, mdns::resolve::resolve_mdns_hostname};

use anyhow::Result;
use client::run_client;

pub mod client;
pub mod util;

pub fn handle_send_cmd(cmd: &SendArgs, cfg: &Config) -> Result<()> {
    match cmd.subcmd {
        SendCommand::Ip(SendIpArgs {
            ref ip,
            port,
            ref message,
            compression,
        }) => run_client(
            ip.parse()?,
            port,
            message.as_deref(),
            cmd.mmap,
            cmd.file.as_deref(),
            cmd.prealloc,
            compression,
        )?,
        #[cfg(feature = "mdns")]
        SendCommand::Mdns(SendMdnsArgs {
            ref hostname,
            timeout_ms,
            ip_version,
            port,
            ref message,
            compression,
        }) => {
            if let Some(resolved_info) = resolve_mdns_hostname(hostname, timeout_ms, true)? {
                if let Some(ip) = resolved_info.get_ip(ip_version) {
                    run_client(
                        *ip,
                        port,
                        message.as_deref(),
                        cmd.mmap,
                        cmd.file.as_deref(),
                        cmd.prealloc,
                        compression,
                    )?;
                }
            }
        }
        #[cfg(feature = "ssh")]
        SendCommand::Ssh(ref args) => {
            crate::ssh::handle_send_ssh(cfg, args, cmd.file.as_deref(), cmd.prealloc, cmd.mmap)?
        }
    }
    Ok(())
}
