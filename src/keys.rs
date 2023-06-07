use std::io::Result;

use async_stream::try_stream;
use serde::{Deserialize,Serialize};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_stream::Stream;

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
}

impl Keys {
	pub fn read(&self, stream: TcpStream) -> impl Stream<Item = Result<String>> {
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
}
