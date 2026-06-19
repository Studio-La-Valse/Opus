use serde::Deserialize;
use crate::bounding_box::BoundingBox;

#[derive(Debug, Deserialize)]
pub struct BoundingBoxMetadata {
    #[serde(rename = "bBoxNE")]
    pub ne: [f32; 2],
    #[serde(rename = "bBoxSW")]
    pub sw: [f32; 2],
}

impl Into<BoundingBox> for &BoundingBoxMetadata {
    fn into(self) -> BoundingBox {
        BoundingBox {
            x_min: self.sw[0],
            y_min: self.sw[1],
            x_max: self.ne[0],
            y_max: self.ne[1],
        }
    }
}
