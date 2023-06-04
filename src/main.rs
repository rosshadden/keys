use std::borrow::Cow;
use std::fs;

use clap::{Parser, Subcommand};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
// use tokio::select;

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
}

async fn start() -> io::Result<()> {
	// remove socket if it exists
	if fs::metadata(SOCKET_ADDRESS).is_ok() {
		fs::remove_file(SOCKET_ADDRESS).ok();
	}

	let unix_listener = UnixListener::bind(SOCKET_ADDRESS)?;
	println!("listening on unix socket {}", SOCKET_ADDRESS);

	let mut tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	println!("tcp server listening on {}", TCP_ADDRESS);

	loop {
		let (mut socket, _) = unix_listener.accept().await?;

		// read data from the client
		let mut buffer = [0u8; 1024];
		let bytes_read = socket.read(&mut buffer).await.unwrap();
		let request = String::from_utf8_lossy(&buffer[..bytes_read]);

		let response = match request {
			Cow::Borrowed("/GET") => {
				println!("get");
				let mut response = vec![0u8; 64];
				if let Err(e) = tcp_stream.read_exact(&mut response).await {
					eprintln!("Failed to receive response from server: {}", e);
				}

				let response = String::from_utf8_lossy(&response);
				println!("Received response from server: {}", response);
				"<LAYER>"
			},
			_ => {
				println!("set {}", request);
				let message = json!({ "ChangeLayer": { "new": request } }).to_string();
				if let Err(e) = tcp_stream.write_all(message.as_bytes()).await {
					eprintln!("Failed to send data to server: {}", e);
				}

				"<LAYER>"
			}
		};

		socket.write_all(response.as_bytes()).await.unwrap();
	}
}

async fn get() -> io::Result<String> {
	let mut socket = UnixStream::connect(SOCKET_ADDRESS).await?;
	socket.write_all("/GET".as_bytes()).await?;

	let mut buffer = [0u8; 1024];
	let bytes_read = socket.read(&mut buffer).await?;
	let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

	Ok(response)
}

async fn set(layer: String) -> io::Result<String> {
	let mut socket = UnixStream::connect(SOCKET_ADDRESS).await?;
	socket.write_all(layer.as_bytes()).await?;

	let mut buffer = [0u8; 1024];
	let bytes_read = socket.read(&mut buffer).await?;
	let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

	Ok(response)
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
		None => {
			start().await?;
		},
	}

	Ok(())
}
