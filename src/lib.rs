use std::net::{TcpListener, TcpStream};
pub struct NetHandler {
    pub client: Option<TcpStream>,
    pub server: Option<TcpListener>,
}

impl Default for NetHandler {
    fn default() -> Self {
        NetHandler {
            client: None,
            server: None,
        }
    }
}

impl NetHandler {
    pub fn new_client(url: impl Into<String>) -> Result<Self, String> {
        let stream = match TcpStream::connect(url.into()) {
            Ok(stream) => stream,
            Err(e) => return Err(e.to_string()),
        };
        Ok(Self {
            client: Option::from(stream),
            ..Default::default()
        })
    }
    pub fn new_server(url: impl Into<String>) -> Result<Self, String> {
        let bind = match TcpListener::bind(url.into()) {
            Ok(stream) => stream,
            Err(e) => return Err("Failed to bind Url".to_string() + &*e.to_string()),
        };
        Ok(Self {
            server: Option::from(bind),
            ..Default::default()
        })
    }
    pub fn send_message_server<T: bincode::Encode>(message_handel: Self, content: T) -> Result<(), String> {
        let config = bincode::config::standard().with_big_endian();
        let encoded_content = match bincode::encode_to_vec(content, config) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to encode Content: {:?}", e)),
        };
        let total_len = encoded_content.len() as u32;
        let len_bytes = total_len.to_be_bytes();

        // Erst Gesamtlänge
        message_handel.server
            .write_all(&len_bytes);

        // Dann alles auf einmal
        stream
            .write_all(&encoded_content)

        Ok(())
    }
}
