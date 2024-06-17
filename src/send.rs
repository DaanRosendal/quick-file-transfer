use crate::{
    config::{Config, SendCommand, SendIpArgs, SendMdnsArgs},
    mdns::resolve_mdns_hostname,
};
use anyhow::Result;
use client::run_client;

mod client;

pub fn handle_send_cmd(cmd: SendCommand, cfg: &Config) -> Result<()> {
    match cmd {
        SendCommand::Ip(SendIpArgs { ip, port }) => run_client(ip.parse()?, port, cfg)?,
        SendCommand::Mdns(SendMdnsArgs {
            hostname,
            timeout_ms,
            ip_version,
            port,
        }) => {
            if let Some(resolved_info) = resolve_mdns_hostname(&hostname, timeout_ms)? {
                if let Some(ip) = resolved_info.get_ip(ip_version) {
                    run_client(*ip, port, cfg)?;
                }
            }
        }
    }
    Ok(())
}
