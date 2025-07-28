use std::borrow::Cow;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SensorDescriptor {
    pub id: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

impl From<myhomelab_sensor_prelude::sensor::SensorDescriptor> for SensorDescriptor {
    fn from(value: myhomelab_sensor_prelude::sensor::SensorDescriptor) -> Self {
        Self {
            id: value.id.into(),
            name: value.name.into(),
            description: value.description.into(),
        }
    }
}
