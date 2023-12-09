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
    pub train: u64,
    pub sensor_id: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SwitchMsg {
    pub state: [u16; 5],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SetTrainSpeedMsg {
    pub train: u64,
    pub speed: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SetSwitchMsg {
    pub switch_id: u64,
    pub state: bool,
}
