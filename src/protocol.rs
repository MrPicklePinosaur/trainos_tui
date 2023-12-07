use serde::{Deserialize, Serialize};

pub enum MsgType {
    Sensor = 1,
    TrainSpeed = 1 << 8,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SensorMsg {
    train: u64,
    sensor_id: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TrainSpeedMsg {
    train: u64,
    speed: u64,
}
