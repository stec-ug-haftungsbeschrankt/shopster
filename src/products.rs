//! Product catalog management.

use crate::TenantDb;
use crate::error::ShopsterError;
use crate::postgresql::dbproduct::DbProduct;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use uuid::Uuid;

/// Sort order for product search results.
#[derive(Debug, Clone, Copy, Default)]
pub enum ProductSort {
    /// Newest products first (default).
    #[default]
    CreatedAtDesc,
    TitleAsc,
    TitleDesc,
    PriceAsc,
    PriceDesc,
}

/// Parameters for [`Products::search`].
#[derive(Debug, Clone, Default)]
pub struct ProductSearchParams {
    /// Case-insensitive substring match against title, descriptions, and article_number.
    pub query: Option<String>,
    /// Filter to products that have ALL of these tags.
    pub tags: Vec<String>,
    pub sort: ProductSort,
    /// Maximum number of results to return (default: 50).
    pub limit: i64,
    /// Number of results to skip for pagination (default: 0).
    pub offset: i64,
}

/// A page of results with a total count for pagination.
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
}

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


fn map_product_unique_violation(e: ShopsterError) -> ShopsterError {
    if let ShopsterError::DatabaseError(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info)) = e {
        let constraint = info.constraint_name().unwrap_or("");
        if constraint.contains("article_number") {
            return ShopsterError::ConflictError("Article number already exists".to_string());
        }
        if constraint.contains("gtin") {
            return ShopsterError::ConflictError("GTIN already exists".to_string());
        }
    }
    e
}

/// Handler for product management operations.
pub struct Products {
    tenant: TenantDb
}

impl Products {
    pub fn new(tenant: TenantDb) -> Self {
        Products { tenant }
    }

    /// Search and filter products with pagination.
    pub async fn search(&self, params: ProductSearchParams) -> Result<PagedResult<Product>, ShopsterError> {
        let limit = if params.limit <= 0 { 50 } else { params.limit };
        let offset = if params.offset < 0 { 0 } else { params.offset };

        let (db_products, total) = DbProduct::search(&self.tenant, &params.query, &params.tags, params.sort, limit, offset).await?;
        let items = db_products.iter().map(Product::from).collect();
        Ok(PagedResult { items, total })
    }

    pub async fn get_all(&self) -> Result<Vec<Product>, ShopsterError> {
        let db_products = DbProduct::get_all(&self.tenant).await?;
        let products = db_products.iter().map(Product::from).collect();
        Ok(products)
    }

    pub async fn get(&self, product_id: i64) -> Result<Product, ShopsterError> {
        let db_product = DbProduct::find(&self.tenant, product_id).await?;
        let product = Product::from(&db_product);
        Ok(product)
    }

    pub async fn find_by_article_number(&self, article_number: &str) -> Result<Option<Product>, ShopsterError> {
        let db_product = DbProduct::find_by_article_number(&self.tenant, article_number).await?;
        Ok(db_product.as_ref().map(Product::from))
    }

    pub async fn find_by_gtin(&self, gtin: &str) -> Result<Option<Product>, ShopsterError> {
        let db_product = DbProduct::find_by_gtin(&self.tenant, gtin).await?;
        Ok(db_product.as_ref().map(Product::from))
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
        let created_product = DbProduct::create(&self.tenant, db_product).await
            .map_err(map_product_unique_violation)?;

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
        let updated_product = DbProduct::update(&self.tenant, product.id, db_product).await
            .map_err(map_product_unique_violation)?;

        let reply = Product::from(&updated_product);
        Ok(reply)
    }

    pub async fn remove(&self, product_id: i64) -> Result<bool, ShopsterError> {
        let result = DbProduct::delete(&self.tenant, product_id).await?;
        Ok(result > 0)
    }
}
