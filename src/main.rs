use clap::{Parser, Subcommand};
use serde::{Deserialize,Serialize};
use serde_json::json;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

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

async fn get() -> io::Result<String> {
	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let mut tcp_reader = BufReader::new(tcp_stream);
	let mut tcp_payload = String::new();

	let payload: GetPayload;
	let mut brackets = 0;

	loop {
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
						return Ok(payload.layer.name);
					}
				},
				_ => {},
			}
		}
	}
}

async fn set(layer: String) -> io::Result<String> {
	let mut tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;

	let message = json!({ "ChangeLayer": { "new": layer } }).to_string();
	if let Err(e) = tcp_stream.write_all(message.as_bytes()).await {
		eprintln!("Failed to send data to server: {}", e);
	}

	Ok(layer)
}

async fn watch() -> io::Result<String> {
	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let mut tcp_reader = BufReader::new(tcp_stream);
	let mut tcp_payload = String::new();

	let mut payload: GetPayload;
	let mut brackets = 0;

	loop {
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
	}
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let cli = Cli::parse();

	match cli.command.unwrap() {
		Commands::Get {} => {
			let response = get().await?;
			println!("{}", response);
		},
		Commands::Set { layer } => {
			let response = set(layer).await?;
			println!("{}", response);
		},
		Commands::Watch {} => {
			watch().await?;
		},
	}

	Ok(())
}
