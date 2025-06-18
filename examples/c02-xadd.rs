use redis::streams::{StreamMaxlen, StreamReadReply};
use redis::{Commands, Value};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
	println!("Hello, world!");

	let stream_id = "stream-01";

	let client = redis::Client::open("redis://127.0.0.1:6379")?;
	let mut con = client.get_connection()?;

	let _: () = con.xadd(stream_id, "*", &[("name", "Mike"), ("surname", "Donavan")])?;

	let res: StreamReadReply = con.xread(&["stream-01"], &["0"])?;

	println!("->> {res:#?}");

	let num: usize = con.xtrim(stream_id, StreamMaxlen::Equals(0))?;
	println!("->> number of records trimmed: {num:#?}");

	Ok(())
}
