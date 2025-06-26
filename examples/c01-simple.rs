//! Simple Valkey/Redis Get/Set (not stream yet)

use redis::{Client, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// "redis://127.0.0.1/" (for default port 6379)
	let client = Client::open("redis://127.0.0.1:6379")?;

	let mut con = client.get_connection()?;

	let _: () = con.set("my_key", 44)?;
	let res: i32 = con.get("my_key")?;

	println!("my_key result: {res}");

	Ok(())
}
