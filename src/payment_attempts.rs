//! Failed payment attempt tracking.

use crate::TenantDb;
use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

use crate::error::ShopsterError;
use crate::postgresql::dbpaymentattempt::{DbPaymentAttempt, InsertableDbPaymentAttempt};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

/// A record of a failed or declined payment attempt.
pub struct PaymentAttempt {
    pub id: i64,
    /// The Stripe `payment_intent.id`. Unique — re-delivery of the same webhook is idempotent.
    pub payment_intent_id: String,
    /// Human-readable failure reason (e.g. `"card_declined"`).
    pub reason: String,
    /// The basket the payment was for, if known.
    pub basket_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub created_at: NaiveDateTime,
}

impl From<DbPaymentAttempt> for PaymentAttempt {
    fn from(db: DbPaymentAttempt) -> Self {
        PaymentAttempt {
            id: db.id,
            payment_intent_id: db.payment_intent_id,
            reason: db.reason,
            basket_id: db.basket_id,
            amount: db.amount,
            currency: db.currency,
            created_at: db.created_at,
        }
    }
}

/// Handler for payment attempt operations.
pub struct PaymentAttempts {
    tenant: TenantDb,
}

impl PaymentAttempts {
    pub fn new(tenant: TenantDb) -> Self {
        PaymentAttempts { tenant }
    }

    /// Returns all recorded payment attempts, most recent first.
    pub async fn get_all(&self) -> Result<Vec<PaymentAttempt>, ShopsterError> {
        let attempts = DbPaymentAttempt::get_all(&self.tenant).await?;
        Ok(attempts.into_iter().map(PaymentAttempt::from).collect())
    }

    /// Looks up a payment attempt by its Stripe payment intent id.
    pub async fn find_by_payment_intent_id(&self, payment_intent_id: &str) -> Result<Option<PaymentAttempt>, ShopsterError> {
        let attempt = DbPaymentAttempt::find_by_payment_intent_id(&self.tenant, payment_intent_id).await?;
        Ok(attempt.map(PaymentAttempt::from))
    }

    /// Records a failed payment attempt. Idempotent: a second call with the same
    /// `payment_intent_id` returns `Ok(None)` without modifying the database.
    pub async fn insert(&self, payment_intent_id: String, reason: String, basket_id: Option<Uuid>, amount: i64, currency: String) -> Result<Option<PaymentAttempt>, ShopsterError> {
        let insertable = InsertableDbPaymentAttempt {
            payment_intent_id,
            reason,
            basket_id,
            amount,
            currency,
            created_at: Utc::now().naive_utc(),
        };

        match DbPaymentAttempt::create(&self.tenant, insertable).await {
            Ok(attempt) => Ok(Some(PaymentAttempt::from(attempt))),
            Err(ShopsterError::DatabaseError(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _))) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
