pub mod config;

use crate::config::{Endian, IrisNetworkConfig, SizeType};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct IrisNetHandler {
    pub stream: Option<TcpStream>,
    pub listener: Option<TcpListener>,
    pub config: IrisNetworkConfig,
}

impl IrisNetHandler {
    /// creates a new Client and returns a Network handle or an Error
    pub fn new_client(
        config: IrisNetworkConfig,
        url: impl Into<String>,
    ) -> Result<IrisNetHandler, String> {
        let stream = match TcpStream::connect(url.into()) {
            Ok(stream) => stream,
            Err(e) => return Err(format!("Could not connect `404`; {}", e.to_string())),
        };
        Ok(Self {
            stream: Option::from(stream),
            listener: None,
            config,
        })
    }
    /// creates a new Server and returns a Network handle or an Error
    pub fn new_server(
        config: IrisNetworkConfig,
        url: impl Into<String>,
    ) -> Result<IrisNetHandler, String> {
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
    /// Closes the Connection between Server and Client
    pub fn close_handel(net_handler: &mut Self) -> Result<(), String> {
        let stream = net_handler.stream.as_mut().ok_or("No stream available")?;

        stream.flush().map_err(|e| format!("Flush failed: {}", e))?;
        stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| format!("Shutdown failed: {}", e))?;
        Ok(())
    }
}
/// This Function sends Messages
pub fn send_message<T: bincode::Encode + Clone>(
    net_handler: &mut IrisNetHandler,
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

    let len_u16 = match net_handler.config.endian {
        Endian::Big => (encoded.len() as u16).to_be_bytes(),
        Endian::Little => (encoded.len() as u16).to_le_bytes(),
    };
    let len_u32 = match net_handler.config.endian {
        Endian::Big => (encoded.len() as u32).to_be_bytes(),
        Endian::Little => (encoded.len() as u32).to_le_bytes(),
    };
    let len_u64 = match net_handler.config.endian {
        Endian::Big => (encoded.len() as u64).to_be_bytes(),
        Endian::Little => (encoded.len() as u64).to_le_bytes(),
    };

    let stream = net_handler.stream.as_mut().ok_or("no stream")?;
    match net_handler.config.size {
        SizeType::U16 => {
            stream.write_all(&len_u16).map_err(|e| e.to_string())?;
        }
        SizeType::U32 => {
            stream.write_all(&len_u32).map_err(|e| e.to_string())?;
        }
        SizeType::U64 => {
            stream.write_all(&len_u64).map_err(|e| e.to_string())?;
        }
    };

    stream.write_all(&encoded).map_err(|e| e.to_string())?;
    match stream.flush() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to Flush{}", e.to_string())),
    }?;
    Ok(())
}
/// This Function reads if there are any Messages in the Inbox
pub fn read_message<'a, T: bincode::Decode<()>>(
    net_handler: &mut IrisNetHandler,
) -> Result<Option<T>, String> {
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
            Ok(0) => return Ok(None),
            Ok(n) => read += n,
            Err(e) => return Err(format!("IO Error while reading length: {}", e)),
        }
    }
    let expected = match (net_handler.config.endian, net_handler.config.size) {
        (Endian::Big, SizeType::U16) => u16::from_be_bytes(buf_u16) as usize,
        (Endian::Big, SizeType::U32) => u32::from_be_bytes(buf_u32) as usize,
        (Endian::Big, SizeType::U64) => u64::from_be_bytes(buf_u64) as usize,

        (Endian::Little, SizeType::U16) => u16::from_le_bytes(buf_u16) as usize,
        (Endian::Little, SizeType::U32) => u32::from_le_bytes(buf_u32) as usize,
        (Endian::Little, SizeType::U64) => u64::from_le_bytes(buf_u64) as usize,
    };

    let mut buf = vec![0u8; expected];
    let mut read = 0;

    while read < expected {
        match stream.read(&mut buf[read..]) {
            Ok(0) => return Ok(None),
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

    Ok(Some(decoded))
}
/// You register a Function, that will be running for each client in a separate Thread
/// You need also to register the Content-Format you're working with
/// Example:
/// ```
/// use iris_net::*;
/// use iris_net::config::IrisNetworkConfig;
///
/// fn main() {
///     //Creating new Server
///     let config = IrisNetworkConfig::default();
///     let net_handler =
///         IrisNetHandler::new_server(config, "127.0.0.1:5000").expect("Failed to create server");
///     //register manage_data so that it can be multithreaded by the api
///     add_server_data_manager(manage_data, net_handler)
///         .expect("Failed to register data manager");
/// }
///
/// //Message Format
/// #[derive(bincode::Encode, bincode::Decode, Debug, Clone)]
/// struct Message {
///     text: String,
/// }
///
/// //Function for managing data. It needs to return the same type as the Function has got
/// fn manage_data(msg: Message) -> Message {
///     println!("Client Response: {:?}", msg);
///     Message {
///         text: "Pong".to_string(),
///     }
/// }
/// ```
pub fn add_server_data_manager<
    T: bincode::Decode<()> + bincode::Encode + 'static + Clone + PartialEq,
>(
    f: fn(T, IrisNetMetadata) -> T,
    mut handler: IrisNetHandler,
) -> Result<(), String> {
    let listener = handler.listener.take().ok_or("no listener")?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("New connection: {}", s.peer_addr().unwrap());
                std::thread::spawn(move || {
                    let mut h = IrisNetHandler {
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
                        if msg == None {
                            break;
                        }
                        let metadata = IrisNetMetadata {
                            ip: h.stream.as_ref().unwrap().local_addr().unwrap().to_string()
                        };
                        let reply = f(msg.expect("Failed to unwrap"), metadata);

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

pub struct IrisNetMetadata {
    pub ip: String,
}