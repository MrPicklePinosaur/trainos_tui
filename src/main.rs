mod protocol;

use std::{
    io::{self, Write},
    time::Duration,
};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::protocol::*;

const START_MSG_BYTES: usize = 2; // magic bytes 0x6969
const LENGTH_BYTES: usize = 4; // using u32 for msg length
const TYPE_BYTES: usize = 4; // using u32 for msg type

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let (read_serial_tx, mut read_serial_rx) = mpsc::channel::<SerialMessage>(32);
    let (write_serial_tx, mut write_serial_rx) = mpsc::channel::<SerialMessage>(32);

    let addr = std::env::var("SERVER_WS").expect("could not get env var for SERVER_WS");

    let (ws_stream, _) = connect_async(addr).await.expect("Failed to connect");
    let (mut ws_tx, ws_rx) = ws_stream.split();

    tokio::spawn(serial_conn(read_serial_tx, write_serial_rx));

    tokio::spawn(async move {
        ws_rx
            .for_each(|msg| async {
                let data = msg.unwrap().into_data();
                let str_data = std::str::from_utf8(&data).unwrap();
                let value: serde_json::Value = serde_json::from_str(str_data).unwrap();
                let msg_type = value.get("type").unwrap().as_u64().unwrap() as u32;
                println!("from server msg_type = {msg_type}, data = {str_data:?}");
                if msg_type == MsgType::TrainSpeed as u32 {
                    let train_speed_msg: TrainSpeedMsg =
                        serde_json::from_value(value.get("data").unwrap().clone()).unwrap();
                    println!("parsed {train_speed_msg:?}");

                    let serial_msg = SerialMessage {
                        data: bincode::serialize(&train_speed_msg).unwrap(),
                        msg_type,
                        msg_len: std::mem::size_of::<TrainSpeedMsg>() as u32,
                    };

                    write_serial_tx.send(serial_msg).await.unwrap();
                } else {
                    eprintln!("invalid msg type from server {msg_type}");
                }
            })
            .await;
    });

    while let Some(serial_msg) = read_serial_rx.recv().await {
        // deserialize the binary data
        if serial_msg.msg_type == MsgType::Sensor as u32 {
            let sensor: SensorMsg = bincode::deserialize(&serial_msg.data).unwrap();
            let json_data = serde_json::to_string(&sensor).unwrap();
            println!("{json_data:?}");
            ws_tx.send(json_data.into()).await.unwrap();
        } else {
            eprintln!("invalid type {}", serial_msg.msg_type);
        }
    }
}

#[derive(Debug)]
pub struct SerialMessage {
    data: Vec<u8>,
    msg_len: u32,
    msg_type: u32,
}

async fn serial_conn(read: mpsc::Sender<SerialMessage>, mut write: mpsc::Receiver<SerialMessage>) {
    let mut port = serialport::new("/dev/ttyUSB0", 115_200)
        .timeout(Duration::ZERO)
        .open()
        .expect("Failed to open port");

    println!("{port:?}");
    let mut serial_buf: Vec<u8> = vec![0; 1000];
    let mut msg: Vec<u8> = vec![];
    loop {
        if let Ok(send_data) = write.try_recv() {
            println!("send data {send_data:?}");

            let mut raw_data: Vec<u8> = vec![69, 69];
            raw_data.extend_from_slice(&send_data.msg_len.to_le_bytes());
            raw_data.extend_from_slice(&send_data.msg_type.to_le_bytes());
            raw_data.extend_from_slice(&send_data.data);

            println!("send raw {raw_data:?}");

            for i in 0..raw_data.len() {
                port.write(&[raw_data[i]]).unwrap();
                tokio::time::sleep(Duration::from_millis(5)).await;
            }

            println!("done sending");
        }

        // TODO busy polling, perhaps can seperate read and write to seperate tasks?
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                if let Ok(data) = std::str::from_utf8(&serial_buf[..t]) {
                    print!("{data}");
                }

                /*
                msg.extend_from_slice(&serial_buf[..t]);

                // wait for msg header to be read
                if msg.len() < START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES {
                    continue;
                }

                // ignore if not magic bytes
                if msg[0] != 0x69 || msg[1] != 0x69 {
                    panic!("invalid magic bytes");
                }

                let msg_len: [u8; 4] = msg[START_MSG_BYTES..START_MSG_BYTES + LENGTH_BYTES]
                    .try_into()
                    .unwrap();
                let msg_len = u32::from_le_bytes(msg_len);

                let msg_type: [u8; 4] = msg
                    [START_MSG_BYTES + LENGTH_BYTES..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES]
                    .try_into()
                    .unwrap();
                let msg_type = u32::from_le_bytes(msg_type);

                if msg.len() < START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len as usize {
                    continue;
                }

                // let out = msg[START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES
                //     ..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len]
                //     .iter()
                //     .map(|ch| format!("{ch:x}"))
                //     .collect::<Vec<_>>()
                //     .join(" ");
                // println!("len = {msg_len}, type = {msg_type}, msg = {out}");

                let data: Vec<u8> = msg[START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES
                    ..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len as usize]
                    .to_vec();

                let serial_msg = SerialMessage {
                    data,
                    msg_len,
                    msg_type,
                };
                read.send(serial_msg).await.unwrap();

                // finished processing
                msg.drain(..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len as usize);
                */
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
