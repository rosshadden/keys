use clap::{Parser, Subcommand};
use serde::{Deserialize,Serialize};
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
enum Payload {
	LayerChange {
		#[serde(rename = "new")]
		layer: String,
	},
	ChangeLayer {
		#[serde(rename = "new")]
		layer: String,
	},
}

async fn get() -> io::Result<String> {
	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let mut tcp_reader = BufReader::new(tcp_stream);
	let mut tcp_payload = String::new();

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
					if brackets != 0 { continue; }

					if let Payload::LayerChange { layer } = serde_json::from_str(&tcp_payload)? {
						return Ok(layer);
					}
				},
				_ => {},
			}
		}
	}
}

async fn set(layer: String) -> io::Result<String> {
	let mut tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;

	let payload = Payload::ChangeLayer { layer: layer.clone() };
	if let Ok(json) = serde_json::to_string(&payload) {
		tcp_stream.write_all(json.as_bytes()).await.expect("failed to send data");
	}

	Ok(layer)
}

async fn watch() -> io::Result<String> {
	let tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;
	let mut tcp_reader = BufReader::new(tcp_stream);
	let mut tcp_payload = String::new();

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
					if brackets != 0 { continue; }

					if let Payload::LayerChange { layer } = serde_json::from_str(&tcp_payload)? {
						println!("{}", layer);
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
			println!("{}", get().await?);
		},
		Commands::Set { layer } => {
			println!("{}", set(layer).await?);
		},
		Commands::Watch {} => {
			watch().await?;
		},
	}

	Ok(())
}
