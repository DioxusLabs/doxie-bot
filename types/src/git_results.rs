use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangedVersions {
    pub version: HashMap<usize, MinorVersionChanged>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinorVersionChanged {
    pub version: usize,
    pub patch_versions: Vec<PatchVersionChanged>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchVersionChanged {
    pub version: usize,

    pub published: bool,

    pub commits: Vec<PrCommit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrCommit {
    /// The summary of the PR
    pub summary: String,

    /// The `#id` of the PR - so you can go to github.com/dioxuslabs/dioxus/pull/id
    pub id: Option<usize>,

    /// The changed packages of the PR
    /// Determined by walking the diff and saving anything under "packages"
    /// Not guaranteed to be sorted, so you probably wanna sort this when rendering it
    pub changed_packages: HashSet<String>,

    /// The hash of the commit - so you can find it on github.com/dioxuslabs/dioxus/commit/hash
    pub commit_hash: String,

    /// The index of this commit in the log, relative to the base of the release commit
    /// IE the first commit will be "0", the second "1", etc for just this PatchVersionChanged
    ///
    /// This is so we can do things like sort the Patches but retain the order of the commits
    pub head_index: usize,
}
