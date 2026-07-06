//! ID generation utilities.

use uuid::Uuid;

/// Generate a time-sortable UUID v7 (preferred for new rows).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// Generate a random idempotency key.
pub fn new_idempotency_key() -> String {
    Uuid::now_v7().to_string()
}
