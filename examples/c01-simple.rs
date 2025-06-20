use redis::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Connect to Redis
	// or "redis://127.0.0.1:6379" (to specify the port if it is not the default)
	let client = redis::Client::open("redis://127.0.0.1/")?;
	let mut con = client.get_connection()?;

	// Throw away the result; just make sure it does not fail
	let _: () = con.set("my_key", 42)?;
	// Read back the key and return it. Because the return value
	// from the function is a result for an integer, this will automatically
	// convert into one.
	let res: i32 = con.get("my_key")?;

	println!("my_key result: {res}");

	Ok(())
}
