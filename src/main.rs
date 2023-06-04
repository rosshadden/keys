use std::borrow::Cow;
use std::fs;

use clap::{Parser, Subcommand};
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

	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	println!("tcp server listening on {}", TCP_ADDRESS);

	handle_tcp_connection(tcp_stream).await;

	loop {
		let (socket, _) = unix_listener.accept().await?;
		handle_unix_connection(socket).await;
	}
}

async fn handle_tcp_connection(mut stream: TcpStream) {
	// Handle TCP connection
	// ...

	// Example: Send a message and receive a response
	let message = r#"{"ChangeLayer":{"new":"dvorak"}}"#;
	if let Err(e) = stream.write_all(message.as_bytes()).await {
		eprintln!("Failed to send data to server: {}", e);
		return;
	}

	let mut response = vec![0u8; 1024];
	if let Err(e) = stream.read_exact(&mut response).await {
		eprintln!("Failed to receive response from server: {}", e);
		return;
	}

	let response = String::from_utf8_lossy(&response);
	println!("Received response from server: {}", response);
}

async fn handle_unix_connection(mut socket: UnixStream) {
	// tokio::spawn(async move {
	let mut buffer = [0u8; 1024];

	// read data from the client
	let bytes_read = socket.read(&mut buffer).await.unwrap();
	let request = String::from_utf8_lossy(&buffer[..bytes_read]);

	let response = match request {
		Cow::Borrowed("/GET") => {
			println!("get");
			"<LAYER>"
		},
		_ => {
			println!("set {}", request);
			"<LAYER>"
		}
	};

	socket.write_all(response.as_bytes()).await.unwrap();
// });
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
