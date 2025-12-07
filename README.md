# The Iris-Network Crate
This Network provides some Network Functions to ease the use of networks in Rust.
It's also things like Compression or multithreading for optimization.
## Example
### Server
````rust
use iris_net::*;

fn main() {
    let net_handler = new_server("ws://127.0.0.1");
    registered_fn_manage_data_on_server(manage_data, net_handler);
}

struct Message {
    text: String,
}

fn manage_data(msg: Message) {
    println!("Client Response: {:?}", msg);
    return Message { text: "Pong".to_string() };
}
````
### Client
````rust
use iris_net::*;

fn main() {
    let net_handler = new_client("ws://127.0.0.1");
    send_message(Message { text: "Ping".to_string()});
    loop {
        sleep(Duration::from_millis(10)); // optional, um nicht 100% CPU zu ziehen

        match reading_message::<Message>(&mut net_handler) {
            Ok(msg) => {
                println!("Server responded with: {}", msg.text);
                break;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

struct Message {
    text: String,
}

````