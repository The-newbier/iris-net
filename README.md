# The Iris-Network Crate
This Network provides some Network Functions to ease the use of networks in Rust.
It's also does things like Compression or multithreading for optimization.
## Install
```shell
cargo add iris_net bincode
```
## Github
[https://github.com/The-newbier/iris-net](https://github.com/The-newbier/iris-net)

## Example
### Server
````rust
use iris_net::*;
fn main() {
    let net_handler = new_server("127.0.0.1:5000").expect("Failed to create server");
    registered_fn_manage_data_on_server(manage_data, net_handler).expect("Failed to register data manager");
}
#[derive(bincode::Encode, bincode::Decode, Debug)]
struct Message {
    text: String,
}

fn manage_data(msg: Message) -> Message {
    println!("Client Response: {:?}", msg);
    Message { text: "Pong".to_string() }
}
````
### Client
````rust
use iris_net::*;

fn main() {
    let mut net_handler = new_client("127.0.0.1:5000").expect("Failed to connect to server");
    send_message(
        &mut net_handler,
        Message {
            text: "Ping".to_string(),
        },
    ).expect("Failed to send message");
    loop {

        match read_message::<Message>(&mut net_handler) {
            Ok(msg) => {
                println!("Server responded with: {}", msg.text);
                close_net_handel(&mut net_handler);
                break;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}
#[derive(bincode::Encode, bincode::Decode)]
struct Message {
    text: String,
}
````