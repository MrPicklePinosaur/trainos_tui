use serde::{Deserialize, Serialize};

pub enum MsgType {
    Sensor = 1 << 0,
    Switch = 1 << 1,

    SetTrainSpeed = 1 << 8,
    SetSwitch = 1 << 9,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SensorMsg {
    train: u64,
    sensor_id: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SwitchMsg {
    state: [u16; 5],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SetTrainSpeedMsg {
    train: u64,
    speed: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SetSwitchMsg {
    switch_id: u64,
    state: bool,
}
