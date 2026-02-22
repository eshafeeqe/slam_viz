use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPose {
    pub timestamp: f64,
    pub position: [f32; 3],
    pub orientation: [f32; 4], // XYZW quaternion
}
