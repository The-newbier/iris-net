use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct NetHandler {
    pub stream: Option<TcpStream>,
    pub listener: Option<TcpListener>,
}

impl Default for NetHandler {
    fn default() -> Self {
        NetHandler {
            stream: None,
            listener: None,
        }
    }
}

pub fn new_client(url: impl Into<String>) -> Result<NetHandler, String> {
    let stream = match TcpStream::connect(url.into()) {
        Ok(stream) => stream,
        Err(e) => return Err(e.to_string()),
    };
    Ok(NetHandler {
        stream: Option::from(stream),
        ..Default::default()
    })
}
pub fn new_server(url: impl Into<String>) -> Result<NetHandler, String> {
    let bind = match TcpListener::bind(url.into()) {
        Ok(stream) => stream,
        Err(e) => return Err("Failed to bind Url".to_string() + &*e.to_string()),
    };
    Ok(NetHandler {
        listener: Option::from(bind),
        ..Default::default()
    })
}
pub fn send_message<T: bincode::Encode>(
    handler: &mut NetHandler,
    content: T,
) -> Result<(), String> {
    let config = bincode::config::standard().with_big_endian();
    let encoded =
        bincode::encode_to_vec(content, config).map_err(|e| format!("encode error: {:?}", e))?;

    let len = (encoded.len() as u32).to_be_bytes();

    let stream = handler.stream.as_mut().ok_or("no stream")?;

    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(&encoded).map_err(|e| e.to_string())?;
    match stream.flush() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to Flush{}", e.to_string())),
    }?;
    Ok(())
}

pub fn read_message<'a, T: bincode::Decode<()>>(net_handler: &mut NetHandler) -> Result<T, String> {
    let stream = net_handler.stream.as_mut().ok_or("no stream available")?;

    let mut buf = [0u8; 4];
    let mut read = 0;

    while read < 4 {
        match stream.read(&mut buf[read..]) {
            Ok(0) => return Err("connection closed while reading length".into()),
            Ok(n) => read += n,
            Err(e) => return Err(format!("io error while reading length: {}", e)),
        }
    }
    let expected = u32::from_be_bytes(buf) as usize;
    let mut buf = vec![0u8; expected];
    let mut read = 0;

    while read < expected {
        match stream.read(&mut buf[read..]) {
            Ok(0) => return Err("connection closed while reading payload".into()),
            Ok(n) => read += n,
            Err(e) => return Err(format!("io error while reading payload: {}", e)),
        }
    }

    let config = bincode::config::standard().with_big_endian();
    let (decoded, _bytes_read) =
        bincode::decode_from_slice(&buf, config).map_err(|e| format!("Decode error: {:?}", e))?;

    Ok(decoded)
}
pub fn registered_fn_manage_data_on_server<T: bincode::Decode<()> + bincode::Encode + 'static>(
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

pub fn close_net_handel(handler: &mut NetHandler) -> Result<NetHandler, String> {
    let stream = handler.stream.as_mut().ok_or("No stream available")?;

    stream.flush().map_err(|e| format!("Flush failed: {}", e))?;
    stream
        .shutdown(std::net::Shutdown::Both)
        .map_err(|e| format!("Shutdown failed: {}", e))?;
    Ok(NetHandler {
        ..Default::default()
    })
}
