use std::fmt;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::error::ShopsterError;
use crate::baskets::Baskets;
use crate::postgresql::dborder::DbOrder;
use crate::postgresql::dborder::DbOrderItem;
use crate::postgresql::dborder::DbOrderStatus;
use crate::warehouse::Warehouse;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    InProgress,
    ReadyToShip,
    Shipping,
    Done,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<DbOrderStatus> for OrderStatus {
    fn from(status: DbOrderStatus) -> Self {
        match status {
            DbOrderStatus::New => OrderStatus::New,
            DbOrderStatus::InProgress => OrderStatus::InProgress,
            DbOrderStatus::ReadyToShip => OrderStatus::ReadyToShip,
            DbOrderStatus::Shipping => OrderStatus::Shipping,
            DbOrderStatus::Done => OrderStatus::Done,
        }
    }
}

impl From<OrderStatus> for DbOrderStatus {
    fn from(status: OrderStatus) -> Self {
        match status {
            OrderStatus::New => DbOrderStatus::New,
            OrderStatus::InProgress => DbOrderStatus::InProgress,
            OrderStatus::ReadyToShip => DbOrderStatus::ReadyToShip,
            OrderStatus::Shipping => DbOrderStatus::Shipping,
            OrderStatus::Done => DbOrderStatus::Done,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderItemPrice {
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderItemSnapshot {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub article_number: String,
    pub gtin: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub tags: Vec<String>,
    pub title_image: String,
    pub additional_images: Vec<String>,
    pub price: OrderItemPrice,
    pub weight: i64,
}

impl From<&DbOrderItem> for OrderItemSnapshot {
    fn from(db_item: &DbOrderItem) -> Self {
        let tags = db_item
            .tags
            .split('|')
            .filter(|tag| !tag.is_empty())
            .map(String::from)
            .collect();
        let additional_images = db_item
            .additional_images
            .split('|')
            .filter(|image| !image.is_empty())
            .map(String::from)
            .collect();

        OrderItemSnapshot {
            id: db_item.id,
            product_id: db_item.product_id,
            quantity: db_item.quantity,
            article_number: db_item.article_number.clone(),
            gtin: db_item.gtin.clone(),
            title: db_item.title.clone(),
            short_description: db_item.short_description.clone(),
            description: db_item.description.clone(),
            tags,
            title_image: db_item.title_image.clone(),
            additional_images,
            price: OrderItemPrice {
                amount: db_item.price,
                currency: db_item.currency.clone(),
            },
            weight: db_item.weight as i64,
        }
    }
}

impl From<&OrderItemSnapshot> for DbOrderItem {
    fn from(item: &OrderItemSnapshot) -> Self {
        DbOrderItem {
            id: item.id,
            order_id: 0,
            product_id: item.product_id,
            quantity: item.quantity,
            article_number: item.article_number.clone(),
            gtin: item.gtin.clone(),
            title: item.title.clone(),
            short_description: item.short_description.clone(),
            description: item.description.clone(),
            tags: item.tags.join("|"),
            title_image: item.title_image.clone(),
            additional_images: item.additional_images.join("|"),
            price: item.price.amount,
            currency: item.price.currency.clone(),
            weight: item.weight as i32,
            created_at: Utc::now().naive_utc(),
        }
    }
}

pub struct Order {
    pub id: i64,
    pub customer_id: Option<Uuid>,
    pub status: OrderStatus,
    pub delivery_address: String,
    pub billing_address: String,
    pub items: Vec<OrderItemSnapshot>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<&Order> for DbOrder {
    fn from(order: &Order) -> Self {
        DbOrder {
            id: order.id,
            customer_id: order.customer_id,
            status: order.status.into(),
            delivery_address: order.delivery_address.clone(),
            billing_address: order.billing_address.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc())
        }
    }
}


pub struct Orders { 
    tenant_id: Uuid
}

impl Orders {
    pub fn new(tenant_id: Uuid) -> Self {
        Orders { tenant_id }
    }

    fn is_reserving_status(status: OrderStatus) -> bool {
        matches!(status, OrderStatus::New | OrderStatus::InProgress | OrderStatus::ReadyToShip)
    }

    fn apply_reserved_delta(&self, items: &[OrderItemSnapshot], delta: i64) -> Result<(), ShopsterError> {
        let warehouse = Warehouse::new(self.tenant_id);

        for item in items {
            let item_delta = item.quantity * delta;
            warehouse.apply_reserved_delta(item.product_id, item_delta)?;
        }

        Ok(())
    }
    
    pub fn get_all(&self) -> Result<Vec<Order>, ShopsterError> {
        let db_orders = DbOrder::get_all(self.tenant_id)?;
        let mut orders = Vec::new();

        for db_order in db_orders {
            let db_items = DbOrderItem::get_for_order(self.tenant_id, db_order.id)?;
            let items = db_items.iter().map(OrderItemSnapshot::from).collect();

            orders.push(Order {
                id: db_order.id,
                customer_id: db_order.customer_id,
                status: db_order.status.into(),
                delivery_address: db_order.delivery_address,
                billing_address: db_order.billing_address,
                items,
                created_at: db_order.created_at,
                updated_at: db_order.updated_at,
            });
        }

        Ok(orders)
    }
    
    pub fn get_by_id(&self, order_id: i64) -> Result<Order, ShopsterError> {
        let db_order = DbOrder::find(self.tenant_id, order_id)?;
        let db_items = DbOrderItem::get_for_order(self.tenant_id, db_order.id)?;
        let items = db_items.iter().map(OrderItemSnapshot::from).collect();

        Ok(Order {
            id: db_order.id,
            customer_id: db_order.customer_id,
            status: db_order.status.into(),
            delivery_address: db_order.delivery_address,
            billing_address: db_order.billing_address,
            items,
            created_at: db_order.created_at,
            updated_at: db_order.updated_at,
        })
    }
    
    pub fn insert(&self, order: &Order) -> Result<Order, ShopsterError> {
        let db_order = DbOrder::from(order);
        let created_order = DbOrder::create(self.tenant_id, db_order)?;

        let mut db_items: Vec<DbOrderItem> = order.items.iter().map(DbOrderItem::from).collect();
        for db_item in &mut db_items {
            db_item.order_id = created_order.id;
        }
        let created_items = DbOrderItem::create_for_order(self.tenant_id, db_items)?;
        let items: Vec<OrderItemSnapshot> = created_items.iter().map(OrderItemSnapshot::from).collect();

        let created_status: OrderStatus = created_order.status.into();
        if Self::is_reserving_status(created_status) {
            self.apply_reserved_delta(&items, 1)?;
        }

        Ok(Order {
            id: created_order.id,
            customer_id: created_order.customer_id,
            status: created_order.status.into(),
            delivery_address: created_order.delivery_address,
            billing_address: created_order.billing_address,
            items,
            created_at: created_order.created_at,
            updated_at: created_order.updated_at,
        })
    }
    
    pub fn update(&self, order: &Order) -> Result<Order, ShopsterError> {
        let existing_order = DbOrder::find(self.tenant_id, order.id)?;
        let existing_items = DbOrderItem::get_for_order(self.tenant_id, order.id)?;
        let previous_status: OrderStatus = existing_order.status.into();
        let next_status: OrderStatus = order.status;

        let db_order = DbOrder::from(order);
        let updated_order = DbOrder::update(self.tenant_id, order.id, db_order)?;

        let db_items = DbOrderItem::get_for_order(self.tenant_id, updated_order.id)?;
        let items = db_items.iter().map(OrderItemSnapshot::from).collect();

        let previous_reserving = Self::is_reserving_status(previous_status);
        let next_reserving = Self::is_reserving_status(next_status);
        if previous_reserving != next_reserving {
            let delta = if next_reserving { 1 } else { -1 };
            let previous_snapshots: Vec<OrderItemSnapshot> = existing_items.iter().map(OrderItemSnapshot::from).collect();
            self.apply_reserved_delta(&previous_snapshots, delta)?;
        }

        Ok(Order {
            id: updated_order.id,
            customer_id: updated_order.customer_id,
            status: updated_order.status.into(),
            delivery_address: updated_order.delivery_address,
            billing_address: updated_order.billing_address,
            items,
            created_at: updated_order.created_at,
            updated_at: updated_order.updated_at,
        })
    }
    
    pub fn remove(&self, order_id: i64) -> Result<bool, ShopsterError> {
        let existing_order = DbOrder::find(self.tenant_id, order_id)?;
        let existing_items = DbOrderItem::get_for_order(self.tenant_id, order_id)?;
        let existing_status: OrderStatus = existing_order.status.into();
        if Self::is_reserving_status(existing_status) {
            let existing_snapshots: Vec<OrderItemSnapshot> = existing_items.iter().map(OrderItemSnapshot::from).collect();
            self.apply_reserved_delta(&existing_snapshots, -1)?;
        }

        let result = DbOrder::delete(self.tenant_id, order_id)?;
        Ok(result > 0)
    }

    pub fn create_from_basket(&self, basket_id: Uuid, delivery_address: String, billing_address: String) -> Result<Order, ShopsterError> {
        let baskets = Baskets::new(self.tenant_id);
        let basket_items = baskets.get_products_with_details(basket_id)?;

        let mut items = Vec::new();
        for basket_item in basket_items {
            let product = basket_item.product;
            let price = product.price.ok_or_else(|| {
                ShopsterError::InvalidOperationError("Product price missing".to_string())
            })?;

            items.push(OrderItemSnapshot {
                id: 0,
                product_id: product.id,
                quantity: basket_item.quantity,
                article_number: product.article_number,
                gtin: product.gtin,
                title: product.title,
                short_description: product.short_description,
                description: product.description,
                tags: product.tags,
                title_image: product.image_url,
                additional_images: product.additional_images,
                price: OrderItemPrice {
                    amount: price.amount,
                    currency: price.currency,
                },
                weight: product.weight,
            });
        }

        let order = Order {
            id: 0,
            customer_id: None,
            status: OrderStatus::New,
            delivery_address,
            billing_address,
            items,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        };

        self.insert(&order)
    }
}
