use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ShopsterError;
use crate::postgresql::dbwarehouse::DbWarehouse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseItem {
    pub id: i64,
    pub product_id: i64,
    pub in_stock: i64,
    pub reserved: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl WarehouseItem {
    pub fn available(&self) -> i64 {
        self.in_stock - self.reserved
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseItemDetails {
    pub warehouse_id: i64,
    pub product_id: i64,
    pub article_number: String,
    pub title: String,
    pub in_stock: i64,
    pub reserved: i64,
    pub available: i64,
}

impl From<&DbWarehouse> for WarehouseItem {
    fn from(db_item: &DbWarehouse) -> Self {
        WarehouseItem {
            id: db_item.id,
            product_id: db_item.product_id,
            in_stock: db_item.in_stock,
            reserved: db_item.reserved,
            created_at: db_item.created_at,
            updated_at: db_item.updated_at,
        }
    }
}

impl From<&WarehouseItem> for DbWarehouse {
    fn from(item: &WarehouseItem) -> Self {
        DbWarehouse {
            id: item.id,
            product_id: item.product_id,
            in_stock: item.in_stock,
            reserved: item.reserved,
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

pub struct Warehouse {
    tenant_id: Uuid,
}

impl Warehouse {
    pub fn new(tenant_id: Uuid) -> Self {
        Warehouse { tenant_id }
    }

    pub fn get_all(&self) -> Result<Vec<WarehouseItem>, ShopsterError> {
        let db_items = DbWarehouse::get_all(self.tenant_id)?;
        Ok(db_items.iter().map(WarehouseItem::from).collect())
    }

    pub fn get_all_with_details(&self) -> Result<Vec<WarehouseItemDetails>, ShopsterError> {
        let items = self.get_all()?;
        let products = crate::products::Products::new(self.tenant_id);
        let mut result = Vec::new();

        for item in items {
            let product = products.get(item.product_id)?;
            result.push(WarehouseItemDetails {
                warehouse_id: item.id,
                product_id: item.product_id,
                article_number: product.article_number,
                title: product.title,
                in_stock: item.in_stock,
                reserved: item.reserved,
                available: item.available(),
            });
        }

        Ok(result)
    }

    pub fn get_by_product_id(&self, product_id: i64) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::find_by_product_id(self.tenant_id, product_id)?;
        Ok(WarehouseItem::from(&db_item))
    }

    pub fn insert(&self, item: &WarehouseItem) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::from(item);
        let created_item = DbWarehouse::create(self.tenant_id, db_item)?;
        Ok(WarehouseItem::from(&created_item))
    }

    pub fn update_by_product_id(&self, product_id: i64, item: &WarehouseItem) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::from(item);
        let updated_item = DbWarehouse::update_by_product_id(self.tenant_id, product_id, db_item)?;
        Ok(WarehouseItem::from(&updated_item))
    }

    pub fn remove_by_product_id(&self, product_id: i64) -> Result<bool, ShopsterError> {
        let result = DbWarehouse::delete_by_product_id(self.tenant_id, product_id)?;
        Ok(result > 0)
    }

    pub fn apply_reserved_delta(&self, product_id: i64, delta: i64) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::apply_reserved_delta(self.tenant_id, product_id, delta)?;
        Ok(WarehouseItem::from(&db_item))
    }
}
