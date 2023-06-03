use std::borrow::Cow;
use std::fs;

use clap::{Parser, Subcommand};
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

static SOCKET_PATH: &str = "/tmp/keys.sock";

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
	if fs::metadata(SOCKET_PATH).is_ok() {
		fs::remove_file(SOCKET_PATH).ok();
	}

	let listener = UnixListener::bind(SOCKET_PATH)?;
	println!("server listening on {}", SOCKET_PATH);

	loop {
		let (mut socket, _) = listener.accept().await?;

		tokio::spawn(async move {
			let mut buffer = [0u8; 1024];

			// read data from the client
			let bytes_read = socket.read(&mut buffer).await.unwrap();
			let request = String::from_utf8_lossy(&buffer[..bytes_read]);

			let response = match request {
				Cow::Borrowed("/GET") => {
					"<LAYER>"
				},
				_ => {
					"<LAYER>"
				}
			};

			socket.write_all(response.as_bytes()).await.unwrap();
		});
	}
}

async fn get() -> io::Result<String> {
	let mut socket = UnixStream::connect(SOCKET_PATH).await?;
	socket.write_all("/GET".as_bytes()).await?;

	let mut buffer = [0u8; 1024];
	let bytes_read = socket.read(&mut buffer).await?;
	let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

	Ok(response)
}

async fn set(layer: String) -> io::Result<String> {
	let mut socket = UnixStream::connect(SOCKET_PATH).await?;
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
		None => { start().await?; },
	}

	Ok(())
}
