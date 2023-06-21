use std::path::PathBuf;
use std::process::ExitStatus;

use async_stream::try_stream;
use futures_util::pin_mut;
use serde::{Deserialize,Serialize};
use shellexpand::tilde;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Deserialize, Serialize)]
enum Payload {
	ChangeLayer {
		#[serde(rename = "new")]
		layer: String,
	},
	LayerChange {
		#[serde(rename = "new")]
		layer: String,
	},
}

pub struct Keys {
	addr: String,
	port: u16,
}

impl Keys {
	pub fn new(port: u16) -> Self {
		Keys {
			addr: format!("127.0.0.1:{}", port),
			port,
		}
	}

	fn read(&self, stream: TcpStream) -> impl Stream<Item = io::Result<String>> {
		let mut reader = BufReader::new(stream);
		let mut payload = String::new();
		let mut brackets = 0;

		try_stream! {
			loop {
				if let Ok(byte) = reader.read_u8().await {
					let char = char::from_u32(byte.into()).unwrap();
					payload.push(char);

					match char {
						'{' => {
							brackets += 1;
						},
						'}' => {
							brackets -= 1;
							if brackets != 0 { continue; }

							if let Payload::LayerChange { layer } = serde_json::from_str(&payload)? {
								yield layer;
								payload.clear();
							}
						},
						_ => {},
					}
				}
			}
		}
	}

	pub async fn start(&self) -> io::Result<ExitStatus> {
		Command::new("kanata")
			.arg("--port")
			.arg(self.port.to_string())
			.spawn()
			.unwrap()
			.wait()
			.await
	}

	pub async fn get(&self) -> io::Result<String> {
		let stream = TcpStream::connect(self.addr.to_owned()).await?;
		stream.readable().await?;

		let s = self.read(stream);
		pin_mut!(s);

		while let Some(Ok(layer)) = s.next().await {
			return Ok(layer)
		}

		Ok(String::new())
	}

	pub async fn set(&self, layer: String) -> io::Result<String> {
		let mut stream = TcpStream::connect(self.addr.to_owned()).await?;
		stream.writable().await?;

		let payload = Payload::ChangeLayer { layer: layer.clone() };
		if let Ok(json) = serde_json::to_string(&payload) {
			stream.write_all(json.as_bytes()).await?;
		}
		stream.flush().await?;
		stream.shutdown().await?;

		// NOTE: this is necessary to avoid panicking on disconnect
		// ref: https://stackoverflow.com/questions/76347638/connection-reset-by-peer-error-for-simple-tcp-server-with-mio-under-minor-load
		self.get().await?;

		Ok(layer)
	}

	pub async fn watch(&self) -> io::Result<()> {
		let stream = TcpStream::connect(self.addr.to_owned()).await?;
		stream.readable().await?;

		let s = self.read(stream);
		pin_mut!(s);

		while let Some(Ok(layer)) = s.next().await {
			println!("{}", layer);
		}

		Ok(())
	}

	pub async fn toggle(&self, layers: Vec<String>) -> io::Result<String> {
		let current = self.get().await.unwrap();
		let mut next = "";

		let mut c = 0;
		for layer in &layers {
			if &current == layer {
				let n = (c + 1) % layers.len();
				next = &layers[n];
				break;
			}
			c += 1;
		}

		let result = self.set(next.to_string()).await?;
		Ok(result)
	}
}
