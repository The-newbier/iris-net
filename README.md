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
use iris_net::config::IrisNetworkConfig;

fn main() {
    //Creating new Server
    let config = IrisNetworkConfig::default();
    let net_handler =
        NetHandler::new_server(config, "127.0.0.1:5000").expect("Failed to create server");
    //register manage_data so that it can be multithreaded by the api
    registered_fn_manage_data_on_server(manage_data, net_handler)
        .expect("Failed to register data manager");
}

//Message Format
#[derive(bincode::Encode, bincode::Decode, Debug, Clone)]
struct Message {
    text: String,
}

//Function for managing data. It needs to return the same type as the Function has got
fn manage_data(msg: Message) -> Message {
    println!("Client Response: {:?}", msg);
    Message {
        text: "Pong".to_string(),
    }
}
````
### Client
````rust
use iris_net::config::IrisNetworkConfig;
use iris_net::*;

fn main() {
    //Creating Client
    let config = IrisNetworkConfig::default();
    let mut net_handler =
        NetHandler::new_client(config, "127.0.0.1:5000").expect("Failed to connect to server");
    //Sending Message in the Format-Type `Message`-Struct
    send_message(
        &mut net_handler,
        Message {
            text: "Ping".to_string(),
        },
    )
        .expect("Failed to send message");
    //Waiting for Server Response
    loop {
        match read_message::<Message>(&mut net_handler) {
            Ok(msg) => {
                println!("Server responded with: {}", msg.text);
                //Shutting down Handel
                NetHandler::close_handel(&mut net_handler).expect("Failed to close handel");
                //exiting Loop and program
                break;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

//Message-Format
#[derive(bincode::Encode, bincode::Decode, Clone)]
struct Message {
    text: String,
}

````