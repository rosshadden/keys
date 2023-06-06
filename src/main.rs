use std::borrow::Cow;
use std::fs;
use std::str;

use clap::{Parser, Subcommand};
use serde::{Deserialize,Serialize};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::select;

static SOCKET_ADDRESS: &str = "/tmp/keys.sock";
static TCP_ADDRESS: &str = "127.0.0.1:1234";

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	Get {},
	Set { layer: String },
	Watch {},
}

#[derive(Debug, Deserialize, Serialize)]
struct Layer {
	#[serde(rename = "new")]
	name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetPayload {
	#[serde(rename = "LayerChange")]
	layer: Layer,
}

#[derive(Debug, Deserialize, Serialize)]
struct SetPayload {
	#[serde(rename = "ChangeLayer")]
	layer: Layer,
}

async fn run() -> io::Result<()> {
	let mut current_layer = "";

	// remove socket if it exists
	if fs::metadata(SOCKET_ADDRESS).is_ok() {
		fs::remove_file(SOCKET_ADDRESS).ok();
	}

	let unix_listener = UnixListener::bind(SOCKET_ADDRESS)?;
	println!("unix socket: {}", SOCKET_ADDRESS);

	let mut tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let (tcp_read_stream, mut tcp_write_stream) = tcp_stream.split();
	let mut tcp_payload = String::new();
	let mut tcp_reader = BufReader::new(tcp_read_stream);
	println!("tcp server: {}", TCP_ADDRESS);

	let mut brackets = 0;
	let mut payload: GetPayload;

	loop {
		select! {
			Ok((mut socket, _)) = unix_listener.accept() => {
				// read data from the client
				let mut buffer = [0u8; 1024];
				let bytes_read = socket.read(&mut buffer).await.unwrap();
				let request = String::from_utf8_lossy(&buffer[..bytes_read]);

				let response = match request {
					Cow::Borrowed("/GET") => {
						current_layer
					},
					Cow::Borrowed("/WATCH") => {
						current_layer
					},
					_ => {
						let message = json!({ "ChangeLayer": { "new": request } }).to_string();
						if let Err(e) = tcp_write_stream.write_all(message.as_bytes()).await {
							eprintln!("Failed to send data to server: {}", e);
						}

						&request
					}
				};

				socket.write_all(response.as_bytes()).await.unwrap();
			},

			Ok(byte) = tcp_reader.read_u8() => {
				let char = char::from_u32(byte.into()).unwrap();

				tcp_payload.push(char);

				match char {
					'{' => {
						brackets += 1;
					},
					'}' => {
						brackets -= 1;

						if brackets == 0 {
							payload = serde_json::from_str(&tcp_payload)?;
							current_layer = &payload.layer.name;
							tcp_payload.clear();
						}
					},
					_ => {},
				}
			},
		}
	}
}

async fn ipc(cmd: &str) -> io::Result<String> {
	let mut socket = UnixStream::connect(SOCKET_ADDRESS).await?;
	socket.write_all(cmd.as_bytes()).await?;

	let mut buffer = [0u8; 1024];
	let bytes_read = socket.read(&mut buffer).await?;
	let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

	Ok(response)
}

async fn get() -> io::Result<String> {
	ipc("/GET").await
}

async fn set(layer: String) -> io::Result<String> {
	ipc(&layer).await
}

async fn watch() -> io::Result<String> {
	// loop {
	// 	let result = ipc("/WATCH").await;
	// 	println!("{:?}", result);
	// }

	// let mut prev_layer = "";
	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let mut tcp_reader = BufReader::new(tcp_stream);
	let mut tcp_payload = String::new();

	let mut payload: GetPayload;
	let mut brackets = 0;

	loop {
		// tcp_stream.read(&mut [0u8; 64]).await?;
		// if let Ok(aoeu) = ipc("/GET").await {
		// 	println!("{}", aoeu);
		// }

		if let Ok(byte) = tcp_reader.read_u8().await {
			let char = char::from_u32(byte.into()).unwrap();
			tcp_payload.push(char);

			match char {
				'{' => {
					brackets += 1;
				},
				'}' => {
					brackets -= 1;

					if brackets == 0 {
						payload = serde_json::from_str(&tcp_payload)?;
						println!("{}", &payload.layer.name);
						tcp_payload.clear();
					}
				},
				_ => {},
			}
		}

		// tcp_stream.flush().await?;
		// old = res;
		// let len = tcp_reader.fill_buf().await?.len();
		// if len != old_len {
		// 	println!("{}", len);
		// }
		// old_len = len;
	}
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let cli = Cli::parse();

	match cli.command {
		Some(Commands::Get {}) => {
			let response = get().await?;
			println!("{}", response);
		},
		Some(Commands::Set { layer }) => {
			let response = set(layer).await?;
			println!("{}", response);
		},
		Some(Commands::Watch {}) => {
			watch().await?;
		},
		None => {
			run().await?;
		},
	}

	Ok(())
}
