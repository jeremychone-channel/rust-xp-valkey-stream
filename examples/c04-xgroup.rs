use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1:6379")?;

	let stream_name = "stream-c04";

	// -- Writer Task
	let mut con_writer = client.get_multiplexed_async_connection().await?;
	let writer_handle = tokio::spawn(async move {
		println!("WRITER - started");
		for i in 0..10 {
			let id: String = con_writer
				.xadd(stream_name, "*", &[("val", &i.to_string())])
				.await
				.expect("XADD Fail");
			println!("WRITER - sent 'val: {i}' with id: {id}");
			sleep(Duration::from_millis(200)).await;
		}
	});

	// -- Create a group
	let group_01 = "group_01";
	let mut con_group_01 = client.get_multiplexed_async_connection().await?;
	let group_create_res: Result<(), _> = con_group_01.xgroup_create_mkstream(stream_name, group_01, "0").await;
	if let Err(err) = group_create_res {
		if let Some("BUSYGROUP") = err.code() {
			println!("XGROUP - group '{group_01}' ALREADY created");
		} else {
			return Err(err.into());
		}
	} else {
		println!("XGROUP - group '{group_01}' created");
	}

	// -- Reader Task (group_01, consumer_g01_a)
	let mut con_reader = client.get_multiplexed_async_connection().await?;
	let reader_handle = tokio::spawn(async move {
		let consumer = "consumer_g01_a";

		println!("READER - started ({consumer})");
		let options = StreamReadOptions::default().count(1).block(2000).group(group_01, consumer);

		loop {
			let res: Option<StreamReadReply> = con_reader
				.xread_options(&[stream_name], &[">"], &options)
				.await
				.expect("Fail to xread");

			if let Some(reply) = res {
				for stream_key in reply.keys {
					for stream_id in stream_key.ids {
						println!(
							"READER - {group_01} - {consumer} - read: id: {} - fields: {:?}",
							stream_id.id, stream_id.map
						);
						println!("READER - SLEEP 400ms");
						sleep(Duration::from_millis(400)).await;
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
		println!("READER - finished");
	});

	// -- Wait for the tasks
	writer_handle.await?;
	reader_handle.await?;

	// -- Clean up the stream
	let mut con = client.get_multiplexed_async_connection().await?;
	let count: i32 = con.del(stream_name).await?;
	println!("Stream '{stream_name}' deleted ({count} key).");

	Ok(())
}
