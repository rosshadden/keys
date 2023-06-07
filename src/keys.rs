use async_stream::try_stream;
use futures_util::pin_mut;
use serde::{Deserialize,Serialize};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_stream::{Stream, StreamExt};

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

pub struct Keys {
	addr: &'static str,
}

impl Keys {
	pub fn new(addr: &'static str) -> Self {
		Keys {
			addr,
		}
	}

	pub fn read(&self, stream: TcpStream) -> impl Stream<Item = io::Result<String>> {
		let mut tcp_reader = BufReader::new(stream);
		let mut tcp_payload = String::new();
		let mut brackets = 0;

		try_stream! {
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
								yield layer;
								tcp_payload.clear();
							}
						},
						_ => {},
					}
				}
			}
		}
	}

	pub async fn get(&self) -> io::Result<String> {
		let stream = TcpStream::connect(self.addr).await?;
		let s = self.read(stream);
		pin_mut!(s);

		while let Some(Ok(layer)) = s.next().await {
			return Ok(layer);
		}

		Ok(String::new())
	}

	pub async fn set(&self, layer: String) -> io::Result<String> {
		let mut stream = TcpStream::connect(self.addr).await?;

		let payload = Payload::ChangeLayer { layer: layer.clone() };
		if let Ok(json) = serde_json::to_string(&payload) {
			stream.write_all(json.as_bytes()).await.expect("failed to send data");
		}

		Ok(layer)
	}

	pub async fn watch(&self) -> io::Result<()> {
		let stream = TcpStream::connect(self.addr).await?;
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

		let result = self.set(next.to_string()).await.unwrap();
		Ok(result)
	}
}
