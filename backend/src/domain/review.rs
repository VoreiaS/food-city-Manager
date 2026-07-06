//! Review domain types (stub — Phase 6).
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub restaurant_id: Uuid,
    pub rating_food: i16,
    pub rating_delivery: i16,
    pub rating_packaging: i16,
    pub rating_overall: i16,
    pub body: Option<String>,
}
