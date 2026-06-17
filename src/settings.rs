//! Shop configuration and settings management.
//!
//! This module manages key-value settings for shop configuration such as
//! shop name, currency, shipping options, and other configurable parameters.

use uuid::Uuid;

use crate::postgresql::dbsettings::DbSetting;
use crate::ShopsterError;

/// A single shop setting.
///
/// Settings are stored as key-value pairs with type information for validation.
pub struct Setting {
    /// Unique setting ID
    pub id: i32,
    /// Setting key/name
    pub title: String,
    /// Data type (e.g., "string", "integer", "boolean")
    pub datatype: String,
    /// Setting value as string
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

/// Handler for shop settings and configuration.
///
/// Manages application-wide settings for the tenant.
pub struct Settings {
    /// The tenant ID for tenant isolation
    tenant_id: Uuid
}


impl Settings {
    /// Creates a new Settings handler for a tenant.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    pub fn new(tenant_id: Uuid) -> Self {
        Settings { tenant_id }
    }

    /// Retrieves all settings for the tenant.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<Setting>)` - All configured settings
    /// `Err(ShopsterError)` - If database error occurs
    pub fn get_all(&self) -> Result<Vec<Setting>, ShopsterError> {
        let db_settings = DbSetting::get_all(self.tenant_id)?;
        Ok(db_settings.iter().map(Setting::from).collect())
    }

    /// Retrieves a setting by its key/title.
    ///
    /// # Arguments
    ///
    /// * `title` - The setting key to retrieve
    ///
    /// # Returns
    ///
    /// `Ok(Setting)` - The setting value
    /// `Err(ShopsterError)` - If not found or database error
    pub fn get_by_title(&self, title: String) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find_by_title(self.tenant_id, title)?;
        Ok(Setting::from(&setting))
    }

    /// Retrieves a setting by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The setting ID
    ///
    /// # Returns
    ///
    /// `Ok(Setting)` - The setting
    /// `Err(ShopsterError)` - If not found or database error
    pub fn get_by_id(&self, id: i32) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find(self.tenant_id, id)?;
        Ok(Setting::from(&setting))
    }

    /// Creates a new setting.
    ///
    /// # Arguments
    ///
    /// * `title` - Setting key
    /// * `datatype` - Data type for validation
    /// * `value` - Initial value
    ///
    /// # Returns
    ///
    /// `Ok(Setting)` - The created setting
    /// `Err(ShopsterError)` - If creation fails
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

    /// Updates a setting by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The setting ID
    /// * `value` - The new value
    ///
    /// # Returns
    ///
    /// `Ok(Setting)` - The updated setting
    /// `Err(ShopsterError)` - If update fails
    pub fn update_by_id(&self, id: i32, value: String) -> Result<Setting, ShopsterError> {
        let mut db_setting = DbSetting::find(self.tenant_id, id)?;
        db_setting.value = value;
        let updated_setting = DbSetting::update(self.tenant_id, id, db_setting)?;

        let reply = Setting::from(&updated_setting);
        Ok(reply)     
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
