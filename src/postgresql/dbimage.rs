use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DbImage {
    pub id: i32,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}



impl From<&String> for DbImage {
    fn from(data: &String) -> Self {
        serde_json::from_str(data).expect("Unable to deserialize DbImage")
    }
}

impl From<&DbImage> for String {
    fn from(data: &DbImage) -> Self {
        serde_json::to_string(data).expect("Unable to serialize DbImage")
    }
}
