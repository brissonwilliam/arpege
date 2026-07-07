// mod duplicata_test;
mod myquery;
mod norm;
mod search;
mod storage;
mod write;

pub use search::{FindMetaCriteria, MetaMatch};
pub use storage::{new_pool, Storage};
