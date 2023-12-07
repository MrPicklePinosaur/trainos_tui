mod protocol;

use std::{
    io::{self, Write},
    time::Duration,
};

use bincode::config::Options;
use serde::{Deserialize, Serialize};

use crate::protocol::*;

const START_MSG_BYTES: usize = 2; // magic bytes 0x6969
const LENGTH_BYTES: usize = 4; // using u32 for msg length
const TYPE_BYTES: usize = 4; // using u32 for msg type

fn main() {
    //let bincode_config = bincode::config::DefaultOptions::new().with_big_endian();

    let mut port = serialport::new("/dev/ttyUSB0", 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    println!("{port:?}");
    let mut serial_buf: Vec<u8> = vec![0; 1000];
    let mut msg: Vec<u8> = vec![];
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
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
                let msg_len = u32::from_le_bytes(msg_len) as usize;

                let msg_type: [u8; 4] = msg
                    [START_MSG_BYTES + LENGTH_BYTES..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES]
                    .try_into()
                    .unwrap();
                let msg_type = u32::from_le_bytes(msg_type) as usize;

                if msg.len() < START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len {
                    continue;
                }

                // let out = msg[START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES
                //     ..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len]
                //     .iter()
                //     .map(|ch| format!("{ch:x}"))
                //     .collect::<Vec<_>>()
                //     .join(" ");
                // println!("len = {msg_len}, type = {msg_type}, msg = {out}");

                let out = &msg[START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES
                    ..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len];

                // deserialize the binary data
                match msg_type {
                    1 => {
                        let sensor: SensorMsg = bincode::deserialize(&out).unwrap();
                        println!("{sensor:?}");
                    },
                    _ => {
                        eprintln!("invalid type {msg_type}");
                    },
                }

                // finished processing
                msg.drain(..START_MSG_BYTES + LENGTH_BYTES + TYPE_BYTES + msg_len);
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
