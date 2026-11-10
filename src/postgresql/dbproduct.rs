use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use diesel::{
    self,
    Queryable,
    QueryableByName,
    Insertable
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::products::ProductSort;
use crate::ShopsterError;
use crate::schema::*;
use crate::TenantDb;




#[derive(Debug, Serialize, Deserialize, Identifiable, PartialEq, Queryable, QueryableByName, Insertable, AsChangeset)]
#[diesel(table_name = products)]
pub struct DbProduct {
    pub id: i64,
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: String,
    pub title_image: String,
    pub additional_images: String,
    pub price: i64,
    pub currency: String,
    pub weight: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = products)]
pub struct InsertableDbProduct {
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: String,
    pub title_image: String,
    pub additional_images: String,
    pub price: i64,
    pub currency: String,
    pub weight: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>
}

impl From<&DbProduct> for InsertableDbProduct {
    fn from(product: &DbProduct) -> Self {
        InsertableDbProduct {
            title: product.title.clone(),
            gtin: product.gtin.clone(),
            article_number: product.article_number.clone(),
            short_description: product.short_description.clone(),
            description: product.description.clone(),
            price: product.price,
            currency: product.currency.clone(),
            tags: product.tags.clone(),
            title_image: product.title_image.clone(),
            additional_images: product.additional_images.clone(),
            weight: product.weight,
            created_at: product.created_at,
            updated_at: product.updated_at
        }
    }
}



impl DbProduct {

    pub async fn find_by_article_number(tenant: &TenantDb, article_number: &str) -> Result<Option<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let product = products::table
            .filter(products::article_number.eq(article_number))
            .first(&mut conn).await
            .optional()?;
        Ok(product)
    }

    pub async fn find_by_gtin(tenant: &TenantDb, gtin: &str) -> Result<Option<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let product = products::table
            .filter(products::gtin.eq(gtin))
            .first(&mut conn).await
            .optional()?;
        Ok(product)
    }

    pub async fn search(
        tenant: &TenantDb,
        query: &Option<String>,
        tags: &[String],
        sort: ProductSort,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Self>, i64), ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        // Build boxed queries for composable filtering.
        let mut data_q = products::table.into_boxed();
        let mut count_q = products::table.into_boxed();

        if let Some(q) = query {
            let pattern = format!("%{}%", q);
            data_q = data_q.filter(
                products::title.ilike(pattern.clone())
                    .or(products::short_description.ilike(pattern.clone()))
                    .or(products::description.ilike(pattern.clone()))
                    .or(products::article_number.ilike(pattern.clone()))
            );
            count_q = count_q.filter(
                products::title.ilike(pattern.clone())
                    .or(products::short_description.ilike(pattern.clone()))
                    .or(products::description.ilike(pattern.clone()))
                    .or(products::article_number.ilike(pattern))
            );
        }

        for tag in tags {
            // Tags stored as pipe-separated e.g. "foo|bar|baz".
            // Matching by padding both sides with pipes for exact tag boundaries.
            let pattern = format!("%|{}|%", tag);
            let padded_col = diesel::dsl::sql::<diesel::sql_types::Text>("('|' || tags || '|')");
            data_q = data_q.filter(padded_col.clone().like(pattern.clone()));
            count_q = count_q.filter(padded_col.like(pattern));
        }

        let total: i64 = count_q.count().get_result(&mut conn).await?;

        data_q = match sort {
            ProductSort::TitleAsc => data_q.order(products::title.asc()),
            ProductSort::TitleDesc => data_q.order(products::title.desc()),
            ProductSort::PriceAsc => data_q.order(products::price.asc()),
            ProductSort::PriceDesc => data_q.order(products::price.desc()),
            ProductSort::CreatedAtDesc => data_q.order(products::created_at.desc()),
        };

        let items: Vec<DbProduct> = data_q.limit(limit).offset(offset).load(&mut conn).await?;

        Ok((items, total))
    }

    pub async fn find(tenant: &TenantDb, id: i64) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let product = products::table
            .filter(products::id.eq(id))
            .first(&mut conn).await?;
        Ok(product)
    }

    pub async fn get_all(tenant: &TenantDb) -> Result<Vec<Self>, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let products = products::table.load(&mut conn).await?;
        Ok(products)
    }

    pub async fn create(tenant: &TenantDb, product: DbProduct) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let insertable = InsertableDbProduct::from(&product);
        let db_product = diesel::insert_into(products::table)
            .values(insertable)
            .get_result(&mut conn).await?;
        Ok(db_product)
    }

    pub async fn update(tenant: &TenantDb, id: i64, product: DbProduct) -> Result<Self, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let db_product = diesel::update(products::table)
            .filter(products::id.eq(id))
            .set(product)
            .get_result(&mut conn).await?;
        Ok(db_product)
    }

    pub async fn delete(tenant: &TenantDb, id: i64) -> Result<usize, ShopsterError> {
        let pool = tenant.pool().await?;
        let mut conn = pool.get().await.map_err(|e| ShopsterError::DatabaseConnectionError(e.to_string()))?;

        let res = diesel::delete(
                products::table
                    .filter(products::id.eq(id))
            )
            .execute(&mut conn).await?;
        Ok(res)
    }
}
