pub mod config;

use crate::config::{Endian, IrisNetworkConfig, SizeType};
use bincode::{Decode, Encode};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct NetHandler {
    pub stream: Option<TcpStream>,
    pub listener: Option<TcpListener>,
    pub config: IrisNetworkConfig,
}

impl NetHandler {
    pub fn new_client(config: IrisNetworkConfig, url: impl Into<String>) -> Result<NetHandler, String> {
        let stream = match TcpStream::connect(url.into()) {
            Ok(stream) => stream,
            Err(e) => return Err(e.to_string()),
        };
        Ok(Self {
            stream: Option::from(stream),
            listener: None,
            config,
        })
    }
    pub fn new_server(config: IrisNetworkConfig, url: impl Into<String>) -> Result<NetHandler, String> {
        let bind = match TcpListener::bind(url.into()) {
            Ok(stream) => stream,
            Err(e) => return Err("Failed to bind Url".to_string() + &*e.to_string()),
        };
        Ok(Self {
            stream: None,
            listener: Option::from(bind),
            config,
        })
    }
    pub fn close_handel(net_handler: &mut Self) -> Result<(), String> {
        let stream = net_handler.stream.as_mut().ok_or("No stream available")?;

        stream.flush().map_err(|e| format!("Flush failed: {}", e))?;
        stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| format!("Shutdown failed: {}", e))?;
        Ok(())
    }
}

pub fn send_message<T: bincode::Encode + Clone>(
    net_handler: &mut NetHandler,
    content: T,
) -> Result<(), String> {
    let encoded = match net_handler.config.endian.clone() {
        Endian::Big => {
            let config = bincode::config::standard().with_big_endian();
            let encoded = bincode::encode_to_vec(content, config)
                .map_err(|e| format!("encode error: {:?}", e))?;
            encoded
        }
        Endian::Little => {
            let config = bincode::config::standard().with_little_endian();
            let encoded = bincode::encode_to_vec(content, config)
                .map_err(|e| format!("encode error: {:?}", e))?;
            encoded
        }
    };

    let len = match (net_handler.config.endian, net_handler.config.size) {
        (Endian::Big,    SizeType::U16) => (encoded.len() as u16).to_be_bytes(),
        (Endian::Little, SizeType::U16) => (encoded.len() as u16).to_le_bytes(),

        (Endian::Big,    SizeType::U32) => (encoded.len() as u32).to_be_bytes(),
        (Endian::Little, SizeType::U32) => (encoded.len() as u32).to_le_bytes(),

        (Endian::Big,    SizeType::U64) => (encoded.len() as u64).to_be_bytes(),
        (Endian::Little, SizeType::U64) => (encoded.len() as u64).to_le_bytes(),
    };


    let stream = net_handler.stream.as_mut().ok_or("no stream")?;

    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(&encoded).map_err(|e| e.to_string())?;
    match stream.flush() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to Flush{}", e.to_string())),
    }?;
    Ok(())
}

pub fn read_message<'a, T: bincode::Decode<()>>(net_handler: &mut NetHandler) -> Result<T, String> {
    let stream = net_handler.stream.as_mut().ok_or("No Stream available")?;
    let mut buf_u16 = [0u8; 2];
    let mut buf_u32 = [0u8; 4];
    let mut buf_u64 = [0u8; 8];

    let mut read = 0;

    while read < 4 {
        let buf = match net_handler.config.size {
            SizeType::U16 => &mut buf_u16[..],
            SizeType::U32 => &mut buf_u32[..],
            SizeType::U64 => &mut buf_u64[..],
        };

        match stream.read(&mut buf[read..]) {
            Ok(0) => return Err("Connection closed while reading length".into()),
            Ok(n) => read += n,
            Err(e) => return Err(format!("IO Error while reading length: {}", e)),
        }
    }
    let expected = match (net_handler.config.endian, net_handler.config.size) {
        (Endian::Big,    SizeType::U16) => u16::from_be_bytes(buf_u16) as usize,
        (Endian::Big,    SizeType::U32) => u32::from_be_bytes(buf_u32) as usize,
        (Endian::Big,    SizeType::U64) => u64::from_be_bytes(buf_u64) as usize,

        (Endian::Little, SizeType::U16) => u16::from_le_bytes(buf_u16) as usize,
        (Endian::Little, SizeType::U32) => u32::from_le_bytes(buf_u32) as usize,
        (Endian::Little, SizeType::U64) => u64::from_le_bytes(buf_u64) as usize,
    };

    let mut buf = vec![0u8; expected];
    let mut read = 0;

    while read < expected {
        match stream.read(&mut buf[read..]) {
            Ok(0) => return Err("Connection closed while reading payload".into()),
            Ok(n) => read += n,
            Err(e) => return Err(format!("IO error while reading payload: {}", e)),
        }
    }

    let decoded: T = match net_handler.config.endian.clone() {
        Endian::Big => {
            let config = bincode::config::standard().with_big_endian();
            let (decoded, _) = bincode::decode_from_slice(&buf, config)
                .map_err(|e| format!("Decode error: {:?}", e))?;
            decoded
        }
        Endian::Little => {
            let config = bincode::config::standard().with_little_endian();
            let (decoded, _) = bincode::decode_from_slice(&buf, config)
                .map_err(|e| format!("Decode error: {:?}", e))?;
            decoded
        }
    };

    Ok(decoded)
}
pub fn registered_fn_manage_data_on_server<
    T: bincode::Decode<()> + bincode::Encode + 'static + Clone,
>(
    f: fn(T) -> T,
    mut handler: NetHandler,
) -> Result<(), String> {
    let listener = handler.listener.take().ok_or("no listener")?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("New connection: {}", s.peer_addr().unwrap());
                std::thread::spawn(move || {
                    let mut h = NetHandler {
                        stream: Some(s),
                        listener: None,
                        config: handler.config,
                    };

                    loop {
                        let msg = match read_message::<T>(&mut h) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!("Read error: {}", e);
                                break;
                            }
                        };

                        let reply = f(msg);

                        if let Err(e) = send_message(&mut h, reply) {
                            eprintln!("Send error: {}", e);
                            break;
                        }
                    }
                });
            }
            Err(e) => eprintln!("Accept failed: {}", e),
        }
    }

    Ok(())
}