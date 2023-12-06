use std::{
    io::{self, Write},
    time::Duration,
};

const START_MSG_BYTES: usize = 2; // magic bytes 0x6969
const LENGTH_BYTES: usize = 4; // using u32 for msg length

fn main() {
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
                // let out = msg.iter().map(|ch| format!("{ch:x}")).collect::<Vec<_>>().join(" ");
                // println!("cur msg: {out}");

                if msg.len() < START_MSG_BYTES + LENGTH_BYTES {
                    // wait for msg header to be read
                    continue;
                }

                // ignore if not magic bytes
                if msg[0] != 0x69 || msg[1] != 0x69 {
                    panic!("invalid magic bytes");
                }

                let expected_msg_len: [u8; 4] = msg
                    [START_MSG_BYTES..START_MSG_BYTES + LENGTH_BYTES]
                    .try_into()
                    .unwrap();
                let expected_msg_len = u32::from_le_bytes(expected_msg_len) as usize;

                if msg.len() < START_MSG_BYTES + LENGTH_BYTES + expected_msg_len {
                    continue;
                }

                let out = msg[START_MSG_BYTES + LENGTH_BYTES
                    ..START_MSG_BYTES + LENGTH_BYTES + expected_msg_len]
                    .iter()
                    .map(|ch| format!("{ch:x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("done: {out}");
                msg.drain(..START_MSG_BYTES + LENGTH_BYTES + expected_msg_len); // finished processing
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
