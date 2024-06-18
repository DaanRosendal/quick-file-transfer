use std::{
    fs::File,
    io::{self, Write},
    net::{IpAddr, TcpStream},
};

use crate::{
    config::{self, ContentTransferArgs},
    mmap_reader::MemoryMappedReader,
    send::util::{file_with_bufreader, stdin_bufreader, tcp_bufwriter},
    util::{format_data_size, incremental_rw},
    TCP_STREAM_BUFSIZE,
};
use anyhow::Result;
use flate2::{read::GzEncoder, Compression};

pub fn run_client(
    ip: IpAddr,
    port: u16,
    message: Option<&str>,
    use_mmap: bool,
    content_transfer_args: &ContentTransferArgs,
) -> Result<()> {
    let socket_addr = (ip, port);
    let mut tcp_stream = TcpStream::connect(socket_addr)?;
    if content_transfer_args.prealloc() {
        let file_size = File::open(content_transfer_args.file().unwrap())?
            .metadata()?
            .len();
        log::debug!(
            "Requesting preallocation of file of size {} [{file_size} B]",
            format_data_size(file_size)
        );
        tcp_stream.write_all(&file_size.to_be_bytes())?;
    }
    let mut buf_tcp_stream = tcp_bufwriter(&tcp_stream);

    log::info!("Connecting to: {ip}:{port}");
    if let Some(msg) = message {
        let res = buf_tcp_stream.write_all(msg.as_bytes());
        log::debug!("Wrote message: {msg}");
        log::debug!("TCP write result: {res:?}");
    }

    // On-stack dynamic dispatch
    let (mut stdin_read, mut file_read, mut mmap_read);
    let bufreader: &mut dyn io::Read = match content_transfer_args.file() {
        Some(p) if use_mmap => {
            log::debug!("Opening file in memory map mode");
            mmap_read = MemoryMappedReader::new(p)?;
            &mut mmap_read
        }
        Some(p) => {
            log::debug!("Opening file in buffered reading mode");
            file_read = file_with_bufreader(p)?;
            &mut file_read
        }
        None => {
            log::debug!("Reading from stdin");
            stdin_read = stdin_bufreader();
            &mut stdin_read
        }
    };

    let compression_mode = content_transfer_args.compression().unwrap_or_default();
    log::debug!("Compression mode: {compression_mode}");
    let transferred_bytes = match content_transfer_args.compression().unwrap_or_default() {
        config::Compression::Lz4 => {
            let mut lz4_writer = lz4_flex::frame::FrameEncoder::new(&mut buf_tcp_stream);
            let total_read = incremental_rw::<TCP_STREAM_BUFSIZE>(&mut lz4_writer, bufreader)?;
            lz4_writer.finish()?;
            total_read
        }
        config::Compression::Gzip => {
            let mut encoder = GzEncoder::new(bufreader, Compression::fast());
            incremental_rw::<TCP_STREAM_BUFSIZE>(&mut buf_tcp_stream, &mut encoder)?
        }
        config::Compression::Bzip2 => todo!(),
        config::Compression::Xz => todo!(),
        config::Compression::None => {
            incremental_rw::<TCP_STREAM_BUFSIZE>(&mut buf_tcp_stream, bufreader)?
        }
    };
    log::info!(
        "Sent {} [{transferred_bytes} B]",
        format_data_size(transferred_bytes)
    );

    Ok(())
}
