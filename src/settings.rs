//! Shop configuration and settings management.
//!
//! Settings flagged as *secret* (API keys, webhook signing secrets) are handled separately:
//! they are encrypted at rest (see [`crate::secrets`]), never returned by [`Settings::get_all`],
//! and their `value` is always empty in a [`Setting`]. Use [`Settings::get_secret`] and
//! [`Settings::set_secret`] to read and write them.

use crate::TenantDb;
use uuid::Uuid;

use crate::postgresql::dbsettings::DbSetting;
use crate::ShopsterError;
use crate::secrets;
use log::warn;

/// A single shop setting.
///
/// For secret settings (`is_secret == true`) `value` is always empty; see the module docs.
#[derive(Clone, PartialEq)]
pub struct Setting {
    pub id: i32,
    pub title: String,
    pub datatype: String,
    pub value: String,
    pub is_secret: bool
}

impl std::fmt::Debug for Setting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Setting")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("datatype", &self.datatype)
            .field("value", if self.is_secret { &"***" } else { &self.value })
            .field("is_secret", &self.is_secret)
            .finish()
    }
}

impl From<&DbSetting> for Setting {
    fn from(db_setting: &DbSetting) -> Self {
        Setting {
            id: db_setting.id,
            title: db_setting.title.clone(),
            datatype: db_setting.datatype.clone(),
            value: if db_setting.is_secret { String::new() } else { db_setting.value.clone() },
            is_secret: db_setting.is_secret
        }
    }
}

/// Handler for shop settings and configuration.
pub struct Settings {
    tenant: TenantDb
}


impl Settings {
    pub fn new(tenant: TenantDb) -> Self {
        Settings { tenant }
    }

    /// Returns all non-secret settings. Secret settings are never listed.
    pub async fn get_all(&self) -> Result<Vec<Setting>, ShopsterError> {
        let db_settings = DbSetting::get_all(&self.tenant).await?;
        Ok(db_settings.iter().filter(|s| !s.is_secret).map(Setting::from).collect())
    }

    pub async fn get_by_title(&self, title: String) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find_by_title(&self.tenant, title).await?;
        Ok(Setting::from(&setting))
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Setting, ShopsterError> {
        let setting = DbSetting::find(&self.tenant, id).await?;
        Ok(Setting::from(&setting))
    }

    pub async fn insert(&self, title: String, datatype: String, value: String) -> Result<Setting, ShopsterError> {
        let setting = DbSetting {
            id: 0,
            title,
            datatype,
            value,
            is_secret: false
        };
        let created_setting = DbSetting::create(&self.tenant, setting).await?;
        Ok(Setting::from(&created_setting))
    }

    pub async fn update_by_id(&self, id: i32, value: String) -> Result<Setting, ShopsterError> {
        let mut db_setting = DbSetting::find(&self.tenant, id).await?;
        if db_setting.is_secret {
            return Err(ShopsterError::InvalidOperationError(
                format!("Setting '{}' is secret: use Settings::set_secret", db_setting.title)
            ));
        }
        db_setting.value = value;
        let updated_setting = DbSetting::update(&self.tenant, id, db_setting).await?;

        let reply = Setting::from(&updated_setting);
        Ok(reply)
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<bool, ShopsterError> {
        let result = DbSetting::delete(&self.tenant, id).await?;
        Ok(result > 0)
    }

    pub async fn delete_by_title(&self, title: String) -> Result<bool, ShopsterError> {
        let result = DbSetting::delete_by_title(&self.tenant, &title).await?;
        Ok(result > 0)
    }

    /// Reads and decrypts a secret setting.
    ///
    /// Returns an empty string for a secret that has not been set yet. Fails if the setting
    /// is not flagged as secret, or if no key is configured / the key is wrong.
    pub async fn get_secret(&self, title: &str) -> Result<String, ShopsterError> {
        let key = self.tenant.secret_key()?;
        let db_setting = DbSetting::find_by_title(&self.tenant, title.to_string()).await?;
        if !db_setting.is_secret {
            return Err(ShopsterError::InvalidOperationError(format!("Setting '{}' is not a secret", title)));
        }

        if db_setting.value.is_empty() {
            return Ok(String::new());
        }
        if !secrets::is_encrypted(&db_setting.value) {
            warn!("Secret setting '{}' is stored unencrypted; call Settings::encrypt_legacy_secrets", title);
            return Ok(db_setting.value);
        }
        secrets::decrypt(key, &db_setting.title, &db_setting.value)
    }

    /// Encrypts and stores a secret setting, creating it (flagged secret, datatype `String`)
    /// if it does not exist yet. An existing non-secret setting of that title is rejected.
    pub async fn set_secret(&self, title: &str, value: &str) -> Result<(), ShopsterError> {
        let key = self.tenant.secret_key()?;
        let encrypted = secrets::encrypt(key, title, value)?;

        match DbSetting::find_by_title(&self.tenant, title.to_string()).await {
            Ok(mut existing) => {
                if !existing.is_secret {
                    return Err(ShopsterError::InvalidOperationError(format!("Setting '{}' is not a secret", title)));
                }
                existing.value = encrypted;
                let id = existing.id;
                DbSetting::update(&self.tenant, id, existing).await?;
            }
            Err(ShopsterError::DatabaseError(diesel::result::Error::NotFound)) => {
                let setting = DbSetting {
                    id: 0,
                    title: title.to_string(),
                    datatype: "String".to_string(),
                    value: encrypted,
                    is_secret: true
                };
                DbSetting::create(&self.tenant, setting).await?;
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    /// Encrypts secret settings that are still stored in plaintext (rows created before
    /// encryption existed). Returns the number of rows converted. Idempotent.
    pub async fn encrypt_legacy_secrets(&self) -> Result<usize, ShopsterError> {
        let key = self.tenant.secret_key()?;
        let mut converted = 0;
        for mut db_setting in DbSetting::get_all(&self.tenant).await? {
            if db_setting.is_secret && !db_setting.value.is_empty() && !secrets::is_encrypted(&db_setting.value) {
                db_setting.value = secrets::encrypt(key, &db_setting.title, &db_setting.value)?;
                let id = db_setting.id;
                DbSetting::update(&self.tenant, id, db_setting).await?;
                converted += 1;
            }
        }
        Ok(converted)
    }
}
