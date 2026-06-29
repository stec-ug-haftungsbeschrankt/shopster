//! Product catalog management.

use crate::error::ShopsterError;
use crate::postgresql::dbproduct::DbProduct;
use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use uuid::Uuid;

/// Product pricing information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Price {
    pub amount: i64,
    pub currency: String
}

/// A product in the catalog.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Product {
    pub id: i64,
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image_url: String,
    pub additional_images: Vec<String>,
    pub price: Option<Price>,
    pub weight: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<&DbProduct> for Product {
    fn from(db_product: &DbProduct) -> Self {
        let additional_images = db_product.additional_images.split('|').map(String::from).collect();
        let tags = db_product.tags.split('|').map(String::from).collect();

        Product {
            id: db_product.id,
            title: db_product.title.clone(),
            gtin: db_product.gtin.clone(),
            article_number: db_product.article_number.clone(),
            short_description: db_product.short_description.clone(),
            description: db_product.description.clone(),
            image_url: db_product.title_image.clone(),
            additional_images,
            tags,
            price: Some(Price {
                amount: db_product.price,
                currency: db_product.currency.clone()
            }),
            weight: db_product.weight as i64,
            created_at: db_product.created_at,
            updated_at: db_product.updated_at
        }
    }
}


impl TryFrom<&Product> for DbProduct {
    type Error = ShopsterError;

    fn try_from(product: &Product) -> Result<Self, ShopsterError> {
        let price = product.price.as_ref()
            .ok_or_else(|| ShopsterError::InvalidOperationError(
                "Product price is required".to_string()
            ))?;

        Ok(DbProduct {
            id: product.id,
            title: product.title.clone(),
            gtin: product.gtin.clone(),
            article_number: product.article_number.clone(),
            short_description: product.short_description.clone(),
            description: product.description.clone(),
            price: price.amount,
            currency: price.currency.clone(),
            tags: product.tags.join("|"),
            title_image: product.image_url.clone(),
            additional_images: product.additional_images.join("|"),
            weight: product.weight as i32,
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        })
    }
}


/// Handler for product management operations.
pub struct Products {
    tenant_id: Uuid
}

impl Products {
    pub fn new(tenant_id: Uuid) -> Self {
        Products { tenant_id }
    }

    pub async fn get_all(&self) -> Result<Vec<Product>, ShopsterError> {
        let db_products = DbProduct::get_all(self.tenant_id).await?;
        let products = db_products.iter().map(Product::from).collect();
        Ok(products)
    }

    pub async fn get(&self, product_id: i64) -> Result<Product, ShopsterError> {
        let db_product = DbProduct::find(self.tenant_id, product_id).await?;
        let product = Product::from(&db_product);
        Ok(product)
    }

    pub async fn insert(&self, product: &Product) -> Result<Product, ShopsterError> {
        if product.title.trim().is_empty() {
            return Err(ShopsterError::InvalidOperationError(
                "Product title cannot be empty".to_string(),
            ));
        }
        if let Some(price) = &product.price {
            if price.amount < 0 {
                return Err(ShopsterError::InvalidOperationError(
                    "Product price cannot be negative".to_string(),
                ));
            }
        }
        let db_product = DbProduct::try_from(product)?;
        let created_product = DbProduct::create(self.tenant_id, db_product).await?;

        let reply = Product::from(&created_product);
        Ok(reply)
    }

    pub async fn update(&self, product: &Product) -> Result<Product, ShopsterError> {
        if product.title.trim().is_empty() {
            return Err(ShopsterError::InvalidOperationError(
                "Product title cannot be empty".to_string(),
            ));
        }
        if let Some(price) = &product.price {
            if price.amount < 0 {
                return Err(ShopsterError::InvalidOperationError(
                    "Product price cannot be negative".to_string(),
                ));
            }
        }
        let db_product = DbProduct::try_from(product)?;
        let updated_product = DbProduct::update(self.tenant_id, product.id, db_product).await?;

        let reply = Product::from(&updated_product);
        Ok(reply)
    }

    pub async fn remove(&self, product_id: i64) -> Result<bool, ShopsterError> {
        let result = DbProduct::delete(self.tenant_id, product_id).await?;
        Ok(result > 0)
    }
}
