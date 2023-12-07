use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SensorMsg {
    train: u64,
    sensor_id: u64,
}
