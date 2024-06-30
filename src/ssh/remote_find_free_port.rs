use ssh::SessionBroker;

use crate::util::{IANA_RECOMMEND_DYNAMIC_PORT_RANGE_END, IANA_RECOMMEND_DYNAMIC_PORT_RANGE_START};

const GET_FREE_PORT_CMD_PREFIX: &str = "qft get-free-port";
const START_PORT_OPTION: &str = "--start-port";
const END_PORT_OPTION: &str = "--end-port";

pub fn remote_find_free_port(
    session: &mut SessionBroker,
    start_port: u16,
    end_port: u16,
) -> anyhow::Result<u16> {
    if start_port < IANA_RECOMMEND_DYNAMIC_PORT_RANGE_START {
        log::warn!("Specified start port range of {start_port} is outside of the IANA recommended range for dynamic ports ({IANA_RECOMMEND_DYNAMIC_PORT_RANGE_START}-{IANA_RECOMMEND_DYNAMIC_PORT_RANGE_END})");
    }
    let get_free_port_cmd = format!("{GET_FREE_PORT_CMD_PREFIX} {START_PORT_OPTION} {start_port} {END_PORT_OPTION} {end_port} -q");
    log::debug!(
        "No TCP port specified, querying remote for a free port with '{get_free_port_cmd}'"
    );
    let mut exec = session.open_exec()?;
    exec.send_command(&get_free_port_cmd)?;
    let exit_status = exec.exit_status()?;
    let terminate_msg = exec.terminate_msg()?;
    log::debug!("Exit status: {exit_status}");
    if !terminate_msg.is_empty() {
        log::debug!("Terminate message: {exit_status}");
    }
    let raw_out = exec.get_result()?;
    log::trace!("Receivied raw output {raw_out:?}");
    log::trace!(
        "Receivied output as lossy utf8:{}",
        String::from_utf8_lossy(&raw_out)
    );
    // Take the first N-bytes that are ascii digits and parse them to u16
    let free_port = raw_out
        .iter()
        .take_while(|&&byte| byte.is_ascii_digit())
        .fold(String::new(), |mut acc, &byte| {
            acc.push(byte as char);
            acc
        })
        .parse::<u16>()
        .expect("Failed to parse u16");
    log::trace!(
        "'{get_free_port_cmd}' output as utf8: {}",
        String::from_utf8_lossy(&raw_out)
    );
    Ok(free_port)
}
