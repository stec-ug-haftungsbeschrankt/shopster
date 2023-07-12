
use uuid::Uuid;

use crate::postgresql::dbsettings::DbSetting;
use crate::DbConnection;
use crate::ShopsterError;

pub struct Setting {
    id: i32,
    title: String,
    datatype: String,
    value: String
}

impl From<&DbSetting> for Setting {
    fn from(db_setting: &DbSetting) -> Self {
        Setting {
            id: db_setting.id,
            title: db_setting.title.clone(),
            datatype: db_setting.datatype.clone(),
            value: db_setting.value.clone()
        }
    }
}

pub struct Settings {
    tenant_id: Uuid
}


impl Settings {
    pub fn new(tenant_id: Uuid) -> Self {
        Settings { tenant_id }
    }

    pub fn get_all(&self) -> Result<Vec<Setting>, ShopsterError> {
        let db_settings = DbSetting::get_all(self.tenant_id)?;
        Ok(db_settings.iter().map(Setting::from).collect())
    }

    pub fn get_by_title(&self, title: String) -> Setting {
        todo!()
    }

    pub fn get_by_id(&self, id: Uuid) -> Setting {
        todo!()
    }

    pub fn insert(&self, title: String, datatype: String, value: String) -> Setting {
        todo!()
    }

    pub fn update_by_id(&self, id: Uuid, value: String) -> Setting {
        todo!()
    }


}