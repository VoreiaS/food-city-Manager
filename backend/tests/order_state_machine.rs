//! Unit tests for the order state machine.
//!
//! These don't touch the DB — they validate the `OrderStatus::can_transition_to`
//! method, which is the core domain rule.

use food_city_backend::domain::order::OrderStatus;

#[test]
fn pending_accept_can_transition_to_accepted() {
    assert!(OrderStatus::PendingAccept.can_transition_to(OrderStatus::Accepted));
}

#[test]
fn pending_accept_can_transition_to_canceled() {
    assert!(OrderStatus::PendingAccept.can_transition_to(OrderStatus::Canceled));
}

#[test]
fn pending_accept_can_transition_to_auto_rejected() {
    assert!(OrderStatus::PendingAccept.can_transition_to(OrderStatus::AutoRejected));
}

#[test]
fn accepted_can_transition_to_preparing() {
    assert!(OrderStatus::Accepted.can_transition_to(OrderStatus::Preparing));
}

#[test]
fn accepted_can_transition_to_canceled() {
    assert!(OrderStatus::Accepted.can_transition_to(OrderStatus::Canceled));
}

#[test]
fn preparing_can_transition_to_ready() {
    assert!(OrderStatus::Preparing.can_transition_to(OrderStatus::Ready));
}

#[test]
fn ready_can_transition_to_picked_up() {
    assert!(OrderStatus::Ready.can_transition_to(OrderStatus::PickedUp));
}

#[test]
fn picked_up_can_transition_to_delivering() {
    assert!(OrderStatus::PickedUp.can_transition_to(OrderStatus::Delivering));
}

#[test]
fn delivering_can_transition_to_delivered() {
    assert!(OrderStatus::Delivering.can_transition_to(OrderStatus::Delivered));
}

#[test]
fn delivered_cannot_transition_back() {
    assert!(!OrderStatus::Delivered.can_transition_to(OrderStatus::PendingAccept));
    assert!(!OrderStatus::Delivered.can_transition_to(OrderStatus::Accepted));
    assert!(!OrderStatus::Delivered.can_transition_to(OrderStatus::Canceled));
}

#[test]
fn cannot_skip_states() {
    // Cannot go directly from pending_accept to delivered
    assert!(!OrderStatus::PendingAccept.can_transition_to(OrderStatus::Delivered));
    // Cannot go from accepted to delivered
    assert!(!OrderStatus::Accepted.can_transition_to(OrderStatus::Delivered));
    // Cannot go from preparing to delivered
    assert!(!OrderStatus::Preparing.can_transition_to(OrderStatus::Delivered));
}

#[test]
fn canceled_is_terminal() {
    assert!(!OrderStatus::Canceled.can_transition_to(OrderStatus::Accepted));
    assert!(!OrderStatus::Canceled.can_transition_to(OrderStatus::Delivered));
}
