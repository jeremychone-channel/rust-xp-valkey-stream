use redis::AsyncCommands;
use redis::streams::{StreamReadOptions, StreamReadReply};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1/")?;

	// NOTE: Even if the redis-rs doc say we can reuse/clone multiplexed_async_connection
	//       For read/write on stream, better to have different connection (otherwise, read none)
	let stream_name = "stream-c04";

	println!();

	// -- Writer Task
	let mut con_writer = client.get_multiplexed_async_connection().await?;
	let writer_handle = tokio::spawn(async move {
		println!("WRITER - started");
		for i in 0..10 {
			let id: String = con_writer.xadd(stream_name, "*", &[("val", &i.to_string())]).await.unwrap();
			println!("WRITER - sent 'val: {i}' with id: {id}");
			sleep(Duration::from_millis(200)).await;
		}
		println!("WRITER - finished");
	});

	// -- XREAGROUP group-01
	let group_01 = "group_01";
	let mut con_group_01 = client.get_multiplexed_async_connection().await?;
	let group_create_res: Result<(), _> = con_group_01.xgroup_create_mkstream(stream_name, group_01, "0").await;
	if let Err(err) = group_create_res {
		println!("XGROUP - group '{group_01}' already exists, skipping creation.");
	} else {
		println!("XGROUP - group '{group_01}' created successfully.");
	}

	// Consumer 01
	let reader_handle = tokio::spawn(async move {
		let consumer = "consumer_01";
		let options = StreamReadOptions::default().count(1).block(2000).group(group_01, consumer);
		loop {
			let result: Option<StreamReadReply> = con_group_01
				.xread_options(&[stream_name], &[">"], &options)
				.await
				.expect("Fail to read stream");

			if let Some(reply) = result {
				for stream_key in reply.keys {
					for stream_id in stream_key.ids {
						println!(
							"XREADGROUP - {group_01} - {consumer} - read: id: {} - fields: {:?}",
							stream_id.id, stream_id.map
						);
						println!("XREADGROUP - SLEEP 800ms (sim job)");
						sleep(Duration::from_millis(800)).await;
						let res: Result<(), _> = con_group_01.xack(stream_name, group_01, &[stream_id.id]).await;
						if let Err(res) = res {
							println!("XREADGROUP - ERROR ACK: {res}");
						} else {
							println!("XREADGROUP - ACK OK");
						}
					}
				}
			} else {
				println!("READER - timeout, assuming writer is done.");
				break;
			}
		}
	});

	// -- Wait for tasks to complete
	writer_handle.await?;
	reader_handle.await?;

	println!();

	// -- Clean up the stream
	let mut con = client.get_multiplexed_async_connection().await?;
	let count: i32 = con.del(stream_name).await?;
	println!("Stream '{stream_name}' deleted ({count} key).");

	Ok(())
}
