use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Commands};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
	println!("\n--- c03-async ---");

	let stream_key = "stream-c03";
	let client = redis::Client::open("redis://127.0.0.1:6379")?;

	// -- Setup: clean stream
	{
		let mut con = client.get_multiplexed_async_connection().await?;
		let _: () = con.del(stream_key).await?;
	}

	// -- Task 1: XADD
	let mut con = client.get_multiplexed_async_connection().await?;
	let handle_1 = tokio::spawn(async move {
		println!("->> TASK-XADD PREP");
		sleep(Duration::from_millis(2000)).await;
		println!("->> TASK-XADD START");
		// -- Send 5 stream item
		for i in 0..5 {
			// give time for the reader to start
			sleep(Duration::from_millis(100)).await;

			let name = format!("Mike {i}");
			let id: String = con.xadd(stream_key, "*", &[("name", name.clone())]).await?;
			println!("->> TASK-XADD - {name} (id: {id})");
		}

		Ok::<(), redis::RedisError>(())
	});

	// -- Task 2: XREAD
	let mut con = client.get_multiplexed_async_connection().await?;
	let handle_2 = tokio::spawn(async move {
		println!("->> TASK-XREAD START");
		let mut last_id: String = "$".to_string();
		let mut message_count = 0;

		loop {
			let opts = StreamReadOptions::default().count(1).block(10000); // shorter block time
			println!("->> TASK-XREAD ->> last_id: {last_id}");
			let res: Option<StreamReadReply> = con.xread_options(&[stream_key], &[last_id.to_string()], &opts).await?;

			match res {
				Some(reply) => {
					let Some(first_stream_id) = reply.keys.first().and_then(|s_k| s_k.ids.first()) else {
						println!("->> TASK-XREAD error xread - No first stream key");
						break;
					};

					message_count += 1;
					println!(
						"->> TASK-XREAD - Message {message_count}: {:#?}\n\t->> FAKE PROCESSING",
						first_stream_id
					);
					sleep(Duration::from_millis(1000)).await;

					last_id = first_stream_id.id.to_string();

					// Exit after reading 5 messages
					if message_count >= 5 {
						println!("->> TASK-XREAD END (Read all 5 messages)");
						break;
					}
				}
				None => {
					println!("->> TASK-XREAD - No new messages (timeout)");
					// Continue reading in case more messages arrive
				}
			}
		}

		Ok::<(), redis::RedisError>(())
	});

	// -- Wait for tasks to finish
	let (res1, res2) = tokio::try_join!(handle_1, handle_2)?;
	res1?;
	res2?;

	// -- Cleanup
	{
		let mut con = client.get_multiplexed_async_connection().await?;
		let num: usize = con.xtrim(stream_key, redis::streams::StreamMaxlen::Equals(0)).await?;
		println!("->> number of records trimmed: {num}");
	}

	println!("--- c03-async ---\n");
	Ok(())
}
