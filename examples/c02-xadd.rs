use redis::Commands;
use redis::streams::{StreamReadOptions, StreamReadReply};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1:6379")?;
	let mut con = client.get_connection()?;

	let stream_name = "stream-c02";

	// -- Add an entry
	let id: String = con.xadd(stream_name, "*", &[("name", "Mike"), ("surname", "Donavan")])?;
	println!("XADD - id: {id}");

	// -- Read all stream records from the start
	let res: StreamReadReply = con.xread(&[stream_name], &["0"])?;

	println!("All records for {stream_name}: {res:#?} ");

	// -- Read only one record
	let options = StreamReadOptions::default().count(1);
	let res: StreamReadReply = con.xread_options(&[stream_name], &["0"], &options)?;
	println!("Only one record for {stream_name}: {res:?} ");

	// -- Clean up the stream
	// This will delete the stream
	let count: i32 = con.del(stream_name)?;
	println!("Stream '{stream_name}' deleted ({count} key).");

	Ok(())
}
