use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Commands};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
	println!("\n--- c03-async ---");

	let stream_key = "stream-c03";
	let client = redis::Client::open("redis://127.0.0.1:6379")?;
	let con_orig = client.get_multiplexed_async_connection().await?;

	// -- Setup: clean stream
	let mut con = con_orig.clone();
	{
		let _: () = con.del(stream_key).await?;
	}

	// -- Task 1: XADD
	let handle_1 = tokio::spawn(async move {
		// give time for the reader to start
		sleep(Duration::from_millis(100)).await;

		println!("->> TASK-1 - xadd");
		let id: String = con.xadd(stream_key, "*", &[("name", "Mike")]).await?;
		println!("->> TASK-1 - xadd record id: {id}");

		Ok::<(), redis::RedisError>(())
	});

	// -- Task 2: XREAD
	let mut con = con_orig.clone();
	let handle_2 = tokio::spawn(async move {
		println!("->> TASK-2 - xread block");
		let opts = StreamReadOptions::default().count(1).block(2000); // block up to 2 sec.
		let res: Option<StreamReadReply> = con.xread_options(&[stream_key], &["$"], &opts).await?;
		println!("->> TASK-2 - xread response: {res:#?}");

		Ok::<(), redis::RedisError>(())
	});

	// -- Wait for tasks to finish
	let (res1, res2) = tokio::try_join!(handle_1, handle_2)?;
	res1?;
	res2?;

	// -- Cleanup
	let mut con = con_orig.clone();
	{
		let num: usize = con.xtrim(stream_key, redis::streams::StreamMaxlen::Equals(0)).await?;
		println!("->> number of records trimmed: {num:#?}");
	}

	println!("--- c03-async ---\n");
	Ok(())
}
