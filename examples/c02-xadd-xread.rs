use redis::Commands;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
	println!("Hello, world!");

	let client = redis::Client::open("redis://127.0.0.1:6379")?;
	let mut con = client.get_connection()?;
	// throw away the result, just make sure it does not fail
	let _: () = con.set("my_key", 42)?;

	// read back the key and return it.  Because the return value
	// from the function is a result for integer this will automatically
	// convert into one.
	let res: i32 = con.get("my_key")?;

	println!("->> {res:?}");

	Ok(())
}
