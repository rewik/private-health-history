use super::measurements;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PerUserData {
    measurements: HashMap<measurements::MeasurementDescription, Vec<measurements::Measurement>>,
}

pub struct InMemoryStorage {
    save_file_name: String,
    data: tokio::sync::RwLock<HashMap<u32,PerUserData>>,
}

impl InMemoryStorage {
    pub fn try_from(filepath: &str) -> Result<Self, &'static str> {
        let mut data = tokio::sync::RwLock::new(HashMap::new());
        //TODO: read from file
        Ok(InMemoryStorage {
            save_file_name: filepath.to_string(),
            data,
        })
    }

    pub async fn store_data(&self, _user: u32, _data: PerUserData) {
        todo!();
    }

    pub async fn retrieve_data(&self, user: u32) -> PerUserData {
        let mut retv = HashMap::new();
        {
            let data = self.data.read().await;
            if let Some(userdata) = data.get(&user) {
                for (k, v) in userdata.measurements.iter() {
                    retv.insert(k.clone(), v.to_vec());
                }
            }
        }
        PerUserData {
            measurements: retv,
        }
    }
}
