pub mod code_blocks;
pub mod lists;
pub mod metadata;
pub mod tables;

// Re-export extraction functions for convenience
pub use code_blocks::extract as extract_code_blocks;
pub use lists::extract as extract_lists;
pub use metadata::extract as extract_metadata;
pub use tables::extract as extract_tables;
