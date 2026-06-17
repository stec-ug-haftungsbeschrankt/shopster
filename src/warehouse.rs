//! Inventory and warehouse management.
//!
//! This module handles product inventory tracking with support for stock
//! quantities, reservations, and warehouse operations.

use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ShopsterError;
use crate::postgresql::dbwarehouse::DbWarehouse;

/// A unit of inventory for a product.
///
/// Tracks in-stock quantities and reserved (pending/committed) quantities.
/// Available stock is calculated as: in_stock - reserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseItem {
    /// Item ID
    pub id: i64,
    /// Product ID
    pub product_id: i64,
    /// Units in stock
    pub in_stock: i64,
    /// Units reserved (committed to orders)
    pub reserved: i64,
    /// Creation time
    pub created_at: NaiveDateTime,
    /// Last update time
    pub updated_at: Option<NaiveDateTime>,
}

impl WarehouseItem {
    /// Calculates the available quantity (in_stock - reserved).
    ///
    /// # Returns
    ///
    /// Available units for new orders
    pub fn available(&self) -> i64 {
        self.in_stock - self.reserved
    }
}

/// Warehouse item with full product details.
///
/// Used when displaying warehouse status with product information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseItemDetails {
    /// Warehouse item ID
    pub warehouse_id: i64,
    /// Associated product ID
    pub product_id: i64,
    /// Product article number
    pub article_number: String,
    /// Product title
    pub title: String,
    /// Units in stock
    pub in_stock: i64,
    /// Units reserved
    pub reserved: i64,
    /// Available units (in_stock - reserved)
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

/// Handler for warehouse and inventory management.
///
/// Manages inventory stock levels, reservations, and warehouse operations.
pub struct Warehouse {
    /// The tenant ID for tenant isolation
    tenant_id: Uuid,
}

impl Warehouse {
    /// Creates a new Warehouse handler for a tenant.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - The tenant's UUID
    pub fn new(tenant_id: Uuid) -> Self {
        Warehouse { tenant_id }
    }

    /// Retrieves all warehouse items for the tenant.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<WarehouseItem>)` - All warehouse items
    /// `Err(ShopsterError)` - If database error occurs
    pub fn get_all(&self) -> Result<Vec<WarehouseItem>, ShopsterError> {
        let db_items = DbWarehouse::get_all(self.tenant_id)?;
        Ok(db_items.iter().map(WarehouseItem::from).collect())
    }

    /// Retrieves all warehouse items with full product details.
    ///
    /// # Returns
    ///
    /// `Ok(Vec<WarehouseItemDetails>)` - All warehouse items with product details
    /// `Err(ShopsterError)` - If database error occurs
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

    /// Retrieves a warehouse item by product ID.
    ///
    /// # Returns
    ///
    /// `Ok(WarehouseItem)` - The warehouse item
    /// `Err(ShopsterError)` - If database error occurs
    pub fn get_by_product_id(&self, product_id: i64) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::find_by_product_id(self.tenant_id, product_id)?;
        Ok(WarehouseItem::from(&db_item))
    }

    /// Inserts a new warehouse item.
    ///
    /// # Returns
    ///
    /// `Ok(WarehouseItem)` - The inserted warehouse item
    /// `Err(ShopsterError)` - If database error occurs
    pub fn insert(&self, item: &WarehouseItem) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::from(item);
        let created_item = DbWarehouse::create(self.tenant_id, db_item)?;
        Ok(WarehouseItem::from(&created_item))
    }

    /// Updates a warehouse item by product ID.
    ///
    /// # Returns
    ///
    /// `Ok(WarehouseItem)` - The updated warehouse item
    /// `Err(ShopsterError)` - If database error occurs
    pub fn update_by_product_id(&self, product_id: i64, item: &WarehouseItem) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::from(item);
        let updated_item = DbWarehouse::update_by_product_id(self.tenant_id, product_id, db_item)?;
        Ok(WarehouseItem::from(&updated_item))
    }

    /// Removes a warehouse item by product ID.
    ///
    /// # Returns
    ///
    /// `Ok(bool)` - `true` if the item was removed, `false` otherwise
    /// `Err(ShopsterError)` - If database error occurs
    pub fn remove_by_product_id(&self, product_id: i64) -> Result<bool, ShopsterError> {
        let result = DbWarehouse::delete_by_product_id(self.tenant_id, product_id)?;
        Ok(result > 0)
    }

    /// Applies a reserved delta to a product.
    ///
    /// # Returns
    ///
    /// `Ok(WarehouseItem)` - The updated warehouse item
    /// `Err(ShopsterError)` - If database error occurs
    pub fn apply_reserved_delta(&self, product_id: i64, delta: i64) -> Result<WarehouseItem, ShopsterError> {
        let db_item = DbWarehouse::apply_reserved_delta(self.tenant_id, product_id, delta)?;
        Ok(WarehouseItem::from(&db_item))
    }
}
