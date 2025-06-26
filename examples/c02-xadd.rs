use redis::streams::{StreamMaxlen, StreamReadOptions, StreamReadReply};
use redis::{Client, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1:6379")?;

	let mut con = client.get_connection()?;

	let stream_name = "stream-c02";

	// -- Add an entry
	let id: String = con.xadd(stream_name, "*", &[("name", "Jen"), ("surname", "Donavan")])?;
	println!("XADD - id: {id}");

	// -- Read
	let res: StreamReadReply = con.xread(&[stream_name], &["0"])?;
	println!("Entries:\n{res:#?}");

	// -- Read only one
	let options = StreamReadOptions::default().count(1);
	let res: StreamReadReply = con.xread_options(&[stream_name], &["0"], &options)?;
	println!("Single Entry:\n{res:#?}");

	// -- XTrim
	// Delete the entries
	let res: u64 = con.xtrim(stream_name, StreamMaxlen::Equals(0))?;
	println!("xtrim count: {res}");

	// -- Del
	// Delete the key
	let res: u64 = con.del(stream_name)?;
	println!("del count: {res}"); // always be 1 (from above)

	Ok(())
}
