//! Loyalty domain types.
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "loyalty_tier", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LoyaltyTier {
    Silver,
    Gold,
    Platinum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub points_balance: i64,
    pub tier: LoyaltyTier,
    pub lifetime_points: i64,
}
