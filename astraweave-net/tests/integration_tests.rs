//! Integration tests entry point for astraweave-net

#![cfg(test)]

#[path = "common/mod.rs"]
mod common;

#[path = "integration/auth_tests.rs"]
mod auth_tests;

#[path = "integration/sync_tests.rs"]
mod sync_tests;

#[path = "integration/packet_loss_tests.rs"]
mod packet_loss_tests;
