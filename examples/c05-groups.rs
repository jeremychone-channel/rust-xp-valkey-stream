use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = redis::Client::open("redis://127.0.0.1:6379")?;

	let stream_name = "stream-c05";

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

	// -- Create groups
	let group_01 = "group_01";
	create_group(&client, stream_name, group_01).await?;
	let group_02 = "group_02";
	create_group(&client, stream_name, group_02).await?;

	// -- Run consumers
	let consumer_g01_a_handle = run_consumer(&client, stream_name, group_01, "consumer_g01_a").await?;
	let consumer_g01_b_handle = run_consumer(&client, stream_name, group_01, "consumer_g01_b").await?;

	let consumer_g02_a_handle = run_consumer(&client, stream_name, group_02, "consumer_g02_a").await?;

	// -- Wait for the tasks
	writer_handle.await?;
	consumer_g01_a_handle.await?;
	consumer_g01_b_handle.await?;
	consumer_g02_a_handle.await?;

	// -- Clean up the stream
	let mut con = client.get_multiplexed_async_connection().await?;
	let count: i32 = con.del(stream_name).await?;
	println!("Stream '{stream_name}' deleted ({count} key).");

	Ok(())
}

async fn run_consumer(
	client: &Client,
	stream_name: &'static str,
	group_name: &'static str,
	consumer_name: &'static str,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
	let mut con_reader = client.get_multiplexed_async_connection().await?;
	let reader_handle = tokio::spawn(async move {
		let consumer = consumer_name;

		println!("READER - started ({consumer})");
		let options = StreamReadOptions::default().count(1).block(2000).group(group_name, consumer);

		loop {
			let res: Option<StreamReadReply> = con_reader
				.xread_options(&[stream_name], &[">"], &options)
				.await
				.expect("Fail to xread");

			if let Some(reply) = res {
				for stream_key in reply.keys {
					for stream_id in stream_key.ids {
						println!(
							"READER - {group_name} - {consumer} - read: id: {} - fields: {:?}",
							stream_id.id, stream_id.map
						);
						println!("READER - SLEEP 400ms");
						sleep(Duration::from_millis(400)).await;
						let res: Result<(), _> = con_reader.xack(stream_name, group_name, &[stream_id.id]).await;
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

	Ok(reader_handle)
}

async fn create_group(client: &Client, stream_name: &str, group_name: &str) -> Result<(), Box<dyn std::error::Error>> {
	let mut con = client.get_multiplexed_async_connection().await?;
	let group_create_res: Result<(), _> = con.xgroup_create_mkstream(stream_name, group_name, "0").await;
	if let Err(err) = group_create_res {
		if let Some("BUSYGROUP") = err.code() {
			println!("XGROUP - group '{group_name}' ALREADY created");
		} else {
			return Err(err.into());
		}
	} else {
		println!("XGROUP - group '{group_name}' created");
	}
	Ok(())
}
