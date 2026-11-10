//! Discount coupon management.

use crate::TenantDb;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::error::ShopsterError;
use crate::postgresql::dbcoupon::{DbCoupon, InsertableDbCoupon};

/// A discount coupon that can be applied to an order.
#[derive(Debug, Clone, PartialEq)]
pub struct Coupon {
    pub id: i64,
    pub code: String,
    /// Percentage discount (e.g. 10 means 10% off).
    pub percent_off: Option<i32>,
    /// Fixed amount discount in the smallest currency unit (e.g. cents).
    pub amount_off: Option<i64>,
    pub currency: Option<String>,
    pub valid_from: Option<NaiveDateTime>,
    pub valid_until: Option<NaiveDateTime>,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub created_at: NaiveDateTime,
}

impl From<DbCoupon> for Coupon {
    fn from(db: DbCoupon) -> Self {
        Coupon {
            id: db.id,
            code: db.code,
            percent_off: db.percent_off,
            amount_off: db.amount_off,
            currency: db.currency,
            valid_from: db.valid_from,
            valid_until: db.valid_until,
            max_uses: db.max_uses,
            use_count: db.use_count,
            created_at: db.created_at,
        }
    }
}

/// Handler for coupon management operations.
pub struct Coupons {
    tenant: TenantDb,
}

impl Coupons {
    pub fn new(tenant: TenantDb) -> Self {
        Coupons { tenant }
    }

    pub async fn get_all(&self) -> Result<Vec<Coupon>, ShopsterError> {
        let db_coupons = DbCoupon::get_all(&self.tenant).await?;
        Ok(db_coupons.into_iter().map(Coupon::from).collect())
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<Coupon>, ShopsterError> {
        let coupon = DbCoupon::find_by_code(&self.tenant, code).await?;
        Ok(coupon.map(Coupon::from))
    }

    pub async fn insert(
        &self,
        code: String,
        percent_off: Option<i32>,
        amount_off: Option<i64>,
        currency: Option<String>,
        valid_from: Option<NaiveDateTime>,
        valid_until: Option<NaiveDateTime>,
        max_uses: Option<i32>,
    ) -> Result<Coupon, ShopsterError> {
        let insertable = InsertableDbCoupon {
            code,
            percent_off,
            amount_off,
            currency,
            valid_from,
            valid_until,
            max_uses,
            use_count: 0,
            created_at: chrono::Utc::now().naive_utc(),
        };
        let db_coupon = DbCoupon::create(&self.tenant, insertable).await?;
        Ok(Coupon::from(db_coupon))
    }

    /// Validates a coupon code and returns it if currently usable.
    pub async fn validate(&self, code: &str) -> Result<Option<Coupon>, ShopsterError> {
        let Some(coupon) = DbCoupon::find_by_code(&self.tenant, code).await? else {
            return Ok(None);
        };
        let now = chrono::Utc::now().naive_utc();
        if let Some(valid_from) = coupon.valid_from {
            if now < valid_from {
                return Ok(None);
            }
        }
        if let Some(valid_until) = coupon.valid_until {
            if now > valid_until {
                return Ok(None);
            }
        }
        if let Some(max_uses) = coupon.max_uses {
            if coupon.use_count >= max_uses {
                return Ok(None);
            }
        }
        Ok(Some(Coupon::from(coupon)))
    }

    pub async fn increment_use_count(&self, coupon_id: i64) -> Result<Coupon, ShopsterError> {
        let db_coupon = DbCoupon::increment_use_count(&self.tenant, coupon_id).await?;
        Ok(Coupon::from(db_coupon))
    }

    pub async fn delete(&self, coupon_id: i64) -> Result<bool, ShopsterError> {
        let count = DbCoupon::delete(&self.tenant, coupon_id).await?;
        Ok(count > 0)
    }
}
