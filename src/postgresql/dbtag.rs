use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DbTag {
    pub id: i32,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}



impl From<&String> for DbTag {
    fn from(data: &String) -> Self {
        serde_json::from_str(data).expect("Unable to deserialize DbTag")
    }
}

impl From<&DbTag> for String {
    fn from(data: &DbTag) -> Self {
        serde_json::to_string(data).expect("Unable to serialize DbTag")
    }
}
