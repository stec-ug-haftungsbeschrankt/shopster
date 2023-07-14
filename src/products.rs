
use crate::DbConnection;
use crate::postgresql::dbproduct::DbProduct;
use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

pub struct Price {
    pub amount: i64,
    pub currency: String
}

pub struct Product {
    pub id: i64,
    pub article_number: String,
    gtin: String,
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


impl From<&Product> for DbProduct {
    fn from(product: &Product) -> Self {
        let price = product.price.as_ref().unwrap();

        DbProduct {
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
        }
    }
}


pub struct Products {
    tenant_id: Uuid
}

impl Products {
    pub fn new(tenant_id: Uuid) -> Self {
        Products { tenant_id }
    }
}


/*
async fn get_all(&self, _request: Request<Empty>) -> Result<Response<ProductList>, Status> {
        let db_products = DbProduct::get_all()
            .map_err(|e| Status::unavailable(e.to_string()))?;

        let products = db_products.iter().map(Product::from).collect();
        let reply = ProductList { products };
        Ok(Response::new(reply))
    }

    async fn get(&self, request: Request<ProductRequest>) -> Result<Response<Product>, Status> {
        let product_request = request.into_inner();
        let db_product = DbProduct::find(product_request.id)
            .map_err(|e| Status::unavailable(e.to_string()))?;

        let reply = Product::from(&db_product);
        Ok(Response::new(reply))
    }

    async fn insert(&self, request: Request<Product>) -> Result<Response<Product>, Status> {
        let product = request.into_inner();
        let db_product = DbProduct::from(&product);
        let created_product = DbProduct::create(db_product)
            .map_err(|e| Status::already_exists(e.to_string()))?;

        let reply = Product::from(&created_product);
        Ok(Response::new(reply))
    }

    async fn update(&self, request: Request<Product>) -> Result<Response<Product>, Status> {
        let product = request.into_inner();

        let db_product = DbProduct::from(&product);
        let updated_product = DbProduct::update(product.id, db_product)
            .map_err(|e| Status::aborted(e.to_string()))?;

        let reply = Product::from(&updated_product);
        Ok(Response::new(reply))
    }

    async fn remove(&self, request: Request<ProductRequest>) -> Result<Response<ProductReply>, Status> {
        let product_request = request.into_inner();
        let _result = DbProduct::delete(product_request.id)
            .map_err(|e| Status::aborted(e.to_string()))?;
        let reply = ProductReply {
            id: product_request.id,
            success: true
        };
        Ok(Response::new(reply))
    }
}
*/