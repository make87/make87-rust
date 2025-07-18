use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::{ConfiguredQueryable, ZenohInterface};
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;
use std::error::Error;

macro_rules! recv_and_reply {
    ($prv:expr) => {{
        let provider = $prv;
        let message_encoder = ProtobufEncoder::<PlainText>::new();
        while let Ok(query) = provider.recv_async().await {
            let payload = query.payload().ok_or("No payload to decode")?;
            let message_decoded = message_encoder.decode(&payload.to_bytes());
            match message_decoded {
                Ok(msg) => {
                    println!("Received: {:?}", msg);
                    let reply = PlainText {
                        header: Header {
                            timestamp: Timestamp::get_current_time().into(),
                            entity_path: msg.header.as_ref().unwrap().entity_path.clone(),
                            reference_id: msg.header.as_ref().unwrap().reference_id,
                        }
                        .into(),
                        body: msg.body.chars().rev().collect(),
                        ..Default::default()
                    };
                    let reply_encoded = message_encoder.encode(&reply)?;
                    query
                        .reply(&query.key_expr().clone(), &reply_encoded)
                        .await?;
                }
                Err(e) => eprintln!("Decode error: {e}"),
            }
        }
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let configured_queryable = zenoh_interface
        .get_queryable(&session, "HELLO_WORLD_MESSAGE")
        .await?;

    match configured_queryable {
        ConfiguredQueryable::Fifo(prv) => recv_and_reply!(prv)?,
        ConfiguredQueryable::Ring(prv) => recv_and_reply!(prv)?,
    }

    Ok(())
}
