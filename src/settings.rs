
use uuid::Uuid;

use crate::postgresql::dbsettings::DbSetting;
use crate::ShopsterError;

pub struct Setting {
    pub id: i32,
    pub title: String,
    pub datatype: String,
    pub value: String
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

    pub fn get_by_title(&self, title: String) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find_by_title(self.tenant_id, title)?;
        Ok(Setting::from(&setting))
    }

    pub fn get_by_id(&self, id: i32) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find(self.tenant_id, id)?;
        Ok(Setting::from(&setting))
    }

    pub fn insert(&self, title: String, datatype: String, value: String) -> Result<Setting, ShopsterError> {
        let setting = DbSetting {
            id: 0,
            title,
            datatype,
            value
        };
        let created_setting = DbSetting::create(self.tenant_id, setting)?;
        Ok(Setting::from(&created_setting))
    }

    pub fn update_by_id(&self, id: Uuid, value: String) -> Setting {
        todo!()
        
        /*
        let setting = request.into_inner();

        let db_setting = DbSetting::from(&setting);
        let updated_setting = DbSetting::update(setting.id, db_setting)
            .map_err(|e| Status::aborted(e.to_string()))?;

        let reply = Setting::from(&updated_setting);
        Ok(Response::new(reply))     
        */
    }
        
    pub fn delete_by_id(&self, id: i32) -> Result<bool, ShopsterError> {
        let result = DbSetting::delete(self.tenant_id, id)?;
        Ok(result > 0)
    }
    
    pub fn delete_by_title(&self, title: String) -> Result<bool, ShopsterError> {
        let result = DbSetting::delete_by_title(self.tenant_id, &title)?;
        Ok(result > 0)
    }
}
