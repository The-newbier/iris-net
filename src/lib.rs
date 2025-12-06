mod config;

use std::io::Write;
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
    mut message_handel: NetHandler,
    content: T,
) -> Result<(), String> {
    let config = bincode::config::standard().with_big_endian();
    let encoded_content = match bincode::encode_to_vec(content, config) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to encode Content: {:?}", e)),
    };
    let total_len = encoded_content.len() as u32;
    let len_bytes = total_len.to_be_bytes();

    if let Some(stream) = message_handel.stream.as_mut() {
        match stream.write_all(&len_bytes) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to write to socket: {:?}", e)),
        };
        match stream.write_all(&encoded_content) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to write to socket: {:?}", e)),
        };
    } else {
        return Err("Kein Stream vorhanden".into());
    }

    Ok(())
}

/*pub fn reading_message<'a, T: bincode::Decode<Context<'a>>>(net_handler: NetHandler) -> Result<T, String> {

}*/
pub fn run_fn_manage_data_on_server<'a, T: bincode::Decode<Context<'a>>>(
    f: fn(T),
    content: T,
    net_handler: NetHandler,
) -> Result<(), String> {
    for stream_result in net_handler
        .listener
        .expect("Failed reading acessind handler")
        .incoming()
    {
        match stream_result {
            Ok(stream) => {
                // Spawn für jede Verbindung
                let net_handler_stream = NetHandler {
                    stream: Option::from(stream),
                    listener: None,
                };
                std::thread::spawn(move || loop {
                    
                });
            }
            Err(e) => eprintln!("Accept failed: {e}"),
        }
    }
    Ok(())
}
