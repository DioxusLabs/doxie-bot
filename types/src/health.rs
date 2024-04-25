use std::collections::HashMap;

pub struct Fullstats {
    /// A map of branch to its stat history
    ///
    /// This is designed to allow us to have diverging stats for a stable and a nightly version
    ///
    /// For instance, you'd be able to compare v0.5 to v0.4 and main to 0.4 using tatgs
    pub stats: HashMap<String, BranchStats>,

    /// when was this stats bundle last updated?
    pub last_updated: String,
}

pub struct BranchStats {
    /// The name of the branch
    pub name: String,

    /// A history of commit stats for this branch
    ///
    /// This is not necessarily completely populated - a commit might be missing here
    /// This is populated when PRs are merged so direct commits bypass this cache
    ///
    /// Also we don't populate this for the entire history of dioxus.
    ///
    /// Ultimately though, we should have one rebased commit per PR, though it's not a hard requirement.
    /// This should roughly be a list of commits found by `git log --first-parent`
    ///
    /// `git log --reverse --first-parent --ancestry-path <branch>`
    pub stats: Vec<CommitStats>,
}

/// Stats about a single commit
///
/// This is always saved to a PR when a new commit is made, allowing us to poke at individual PRs
///
/// The hashmaps are used to map a particular benchmark to the stats for that benchmark
///
/// This lets us add new benchmarks over time to test different things
pub struct CommitStats {
    pub pr_name: String,

    pub sha: String,

    pub perf: HashMap<String, PerfStats>,

    /// How big is the .wasm bundle, the .apps, the full bundle, etc.
    pub compile_size: HashMap<String, CompileSizeStats>,

    pub compile_time: HashMap<String, CompiletimeStats>,
}

pub struct PerfStats {
    pub name: String,

    pub raw_walltime: f64,

    /// Not every runnner is made equally, so we attempt to normalize the walltime by running with the
    /// previous main commit and then the current one.
    pub normalized_walltime: f64,
}

pub struct CompileSizeStats {
    pub name: String,

    /// Cargo build --target wasm32 --release
    pub raw_debug_wasm_size: u64,

    pub optimized_wasm_size: u64,
    //
}

pub struct CompiletimeStats {
    pub name: String,

    /// The .html page generated by --timings
    pub timing_page: String,
}
