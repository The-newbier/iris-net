

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::task::Context;

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
    let encoded = bincode::encode_to_vec(content, config)
        .map_err(|e| format!("encode error: {:?}", e))?;

    let len = (encoded.len() as u32).to_be_bytes();

    let stream = handler.stream.as_mut().ok_or("no stream")?;

    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(&encoded).map_err(|e| e.to_string())?;

    Ok(())
}


/*pub fn reading_message<'a, T: bincode::Decode<Context<'a>>>(net_handler: NetHandler) -> Result<T, String> {

}*/
pub fn run_fn_manage_data_on_server<T: for<'a> bincode::Decode<Context>>(
    f: fn(T),
    handler: NetHandler,
) -> Result<(), String> {
    let listener = handler.listener.ok_or("no listener")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                std::thread::spawn(move || {
                    loop {
                        // ---- 1. Länge lesen ----
                        let mut len_buf = [0u8; 4];
                        if let Err(e) = s.read_exact(&mut len_buf) {
                            eprintln!("Failed to read len: {}", e);
                            break;
                        }
                        let needed = u32::from_be_bytes(len_buf) as usize;

                        // ---- 2. Payload lesen ----
                        let mut buf = vec![0u8; needed];
                        if let Err(e) = s.read_exact(&mut buf) {
                            eprintln!("Failed to read payload: {}", e);
                            break;
                        }

                        // ---- 3. Dekodieren ----
                        let config = bincode::config::standard().with_big_endian();
                        let decoded: T = match bincode::decode_from_slice(&buf, config) {
                            Ok((value, _)) => value,
                            Err(e) => {
                                eprintln!("Decode failed: {:?}", e);
                                continue;
                            }
                        };

                        // ---- 4. Callback ausführen ----
                        f(decoded);
                    }
                });
            }
            Err(e) => eprintln!("Accept failed: {}", e),
        }
    }

    Ok(())
}
