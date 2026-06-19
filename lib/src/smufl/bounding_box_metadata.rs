use crate::bounding_box::BoundingBox;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BoundingBoxMetadata {
    #[serde(rename = "bBoxNE")]
    pub ne: [f32; 2],
    #[serde(rename = "bBoxSW")]
    pub sw: [f32; 2],
}

impl From<&BoundingBoxMetadata> for BoundingBox {
    fn from(value: &BoundingBoxMetadata) -> Self {
        BoundingBox {
            x_min: value.sw[0],
            y_min: value.sw[1],
            x_max: value.ne[0],
            y_max: value.ne[1],
        }
    }
}
