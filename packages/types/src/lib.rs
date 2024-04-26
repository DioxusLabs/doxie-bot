//! Types split out so the docsite can import these without bringing in all the of the deps in the main
//! crate.

mod health;
pub use health::*;

mod git_results;
pub use git_results::*;

pub use octocrab_models;
