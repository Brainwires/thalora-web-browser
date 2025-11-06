//! Storage APIs - localStorage, sessionStorage, cookies, IndexedDB

pub mod cookie_store;
pub mod indexed_db;
pub mod storage;
pub mod storage_event;
pub mod storage_manager;

#[cfg(test)]
mod tests;
