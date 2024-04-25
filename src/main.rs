//! A single entrypoint, but with different actions depending on input

use std::{collections::HashSet, path::PathBuf};

use doxie_types::*;
use git2::{Commit, Oid, Repository, Revwalk};
use tokio::process::Command;

mod workflow;

#[tokio::main]
async fn main() {
    // For now, just write to the stats cache as the default
    save_stats_as_artifact().await;
}

async fn bot_loop() {}

/// When a new PR is created, this workflow function will comment on the PR to say hello
///
/// This includes a checklist:
/// - A link to the discord
/// - Request to link an issue
/// - Some info about how this bot works
/// - A link to the contributing guide / code of conduct
/// - A link to the status page that shows the health of the repo, using data spit out by write_status_bloba
///
/// If the information is available:
/// - Warnings about performance changes
/// - Warnings about semver / breaking changes
///
/// This is structured in such a way that we completely overwrite a comment, so it needs to be
/// deterministic with maybe an "updated time" to show that it's been updated.
async fn write_status_comment() {}

/// A single page that shows all the changes for the current tip of main
///
/// This includes benchmarks, closed PRs and issues, milestone information, and a diff of the changed
/// crates for both stable and nightly.
///
/// Would like this to either be on dioxuslabs.com and we query this info or on a living PR that we keep
/// open. We're trying to emulate the cloudflare slipboard with this, so a page on dioxuslabs.com might
/// be nice. Simply dumping the data as json and doing a query against a file might be good enough!
///
/// We don't want to hit the GH API for every request, so running this status page on every commit to main
/// or just every commit to an open PR and then caching that might be the best strategy.
///
/// This includes:
/// - Performance of a particular set of benchmarks in a table format
/// - Binary size of a particular crate compiled to wasm with different lines for opt levels, compression, etc
async fn write_status_blob() {}

/// Run all the benchmarks, tests, etc and save their output as a single json blob
///
/// This should be executing various `cargo make xyz` things and capturing their outputs.
async fn collect_stats() {}

/// Build the wasm examples and optimize them, and then save the
async fn collect_size(name: String) -> CompileSizeStats {
    // Build without optimizations
    let cmd = Command::new("dx").args(["build", "--release", "--no-opt"]);
    let raw_debug_wasm_size = 0;

    // Build with optimizations
    let cmd = Command::new("dx").args(["build", "--release"]);
    let optimized_wasm_size = 0;

    CompileSizeStats {
        name,
        raw_debug_wasm_size,
        optimized_wasm_size,
    }
}

/// Whenever a PR is merged, we kick off the workflow to build the stats for the new main branch and then
/// save that as a single unified blob to the repo.
///
/// This lets the dioxuslabs.com status page read the blob and display stats dynamically without running
/// into API rate limits.
///
/// We should also try to implement some sort of caching/versioning CDN-like mechanism so we don't
/// run into issues. GH gives us 12.5k req/hr which could add up in DDOS scenario
async fn save_stats_as_artifact() {}

/// collect the prs from the main repo
///
/// Lists info about them:
/// - lines changed
/// - connected issue
/// - backported
/// - patch version (ie in between 0.5.1 and 0.5.2)
/// - semver compatibility
/// - should this pr be backported?
async fn collect_prs() {}

fn sha_from_tag(repo: &Repository, target_tag: &str) -> Oid {
    let mut id = None;

    _ = repo.tag_foreach(|this_id, bytes| {
        let name = std::str::from_utf8(bytes).unwrap();

        if name.ends_with(target_tag) {
            let obj = repo.revparse_single(name).unwrap();

            if let Some(tag) = obj.as_tag() {
                id = Some(tag.target_id());
            }

            if let Some(commit) = obj.as_commit() {
                id = Some(commit.id());
            }
        }

        true
    });

    id.unwrap()
}

#[derive(Debug)]
struct Pr<'a> {
    commit: Commit<'a>,

    /// The `#id` of the PR - so you can go to github.com/dioxuslabs/dioxus/pull/id
    id: Option<usize>,

    changed_files: HashSet<PathBuf>,
}

impl Pr<'_> {
    //
    pub fn changed_crates(&self) -> Vec<PathBuf> {
        let mut packages = vec![];

        for file in self.changed_files.iter() {
            if file.starts_with("packages") {
                packages.push(file.clone());
            }
        }

        packages
    }

    // get the name of the package that changed
    // just parse the first folder after "packages/"
    pub fn changed_packages(&self) -> Vec<String> {
        self.changed_crates()
            .iter()
            .map(|path| {
                path.iter()
                    .skip(1)
                    .take(1)
                    .map(|os_str| os_str.to_str().unwrap().to_string())
                    .collect()
            })
            .collect()
    }
}

/// Manually look at the shas between versions
async fn collect_changed_crates(repo: PathBuf) {
    // Run through each tag starting at 0.5.0 and increment the patch version
    // We're trying to get a sense of which crates changed between versions
    // We want to link these to PRs by checking the git log for "merged" PR logs
    // git log --first-parent/ / to get shas for every PR in the past

    let repo = Repository::open(repo).unwrap();
    let start_id = sha_from_tag(&repo, "v0.5.0");
    let end_id = sha_from_tag(&repo, "v0.5.1");

    let mut prs = vec![];

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push(end_id).unwrap();
    revwalk.simplify_first_parent();

    let mut commits = vec![];

    // note that this actually walks backwards
    for commit in revwalk {
        let commit_id = commit.unwrap();
        let commit = repo.find_commit(commit_id).unwrap();
        let commit_id = commit.id();

        commits.push(commit);

        if commit_id == start_id {
            break; // Stop when reaching the starting commit
        }
    }

    // Now walk the merge commits and list out the files changed by that commit
    // Diff that commit with its parent
    for commit in commits.iter() {
        let parent = commit.parent(0).unwrap();
        let diff = repo
            .diff_tree_to_tree(
                Some(&parent.tree().unwrap()),
                Some(&commit.tree().unwrap()),
                None,
            )
            .unwrap();

        let mut pr = Pr {
            commit: commit.clone(),
            changed_files: HashSet::new(),
            id: None,
        };

        for delta in diff.deltas() {
            let old_file = delta.old_file();
            let new_file = delta.new_file();

            pr.changed_files
                .insert(PathBuf::from(old_file.path().unwrap()));
            pr.changed_files
                .insert(PathBuf::from(new_file.path().unwrap()));
        }

        let summary = commit.summary().unwrap();

        // usually a merge commit will contain the ID in the form of (#id)
        // try and find the first match (from the end of the string) that matches that
        // Use a regex, I guess?
        let regex = regex::Regex::new(r"\(#(\d+)\)").unwrap();
        let id = regex
            .captures(summary)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().parse().unwrap());
        pr.id = id;

        prs.push(pr);
    }

    for pr in prs.iter() {
        println!(
            "{}: {:?}",
            pr.commit.summary().unwrap(),
            pr.changed_packages()
        );
    }
}

#[tokio::test]
async fn changed_crates_between() {
    collect_changed_crates("/Users/jonkelley/Development/dioxus".parse().unwrap()).await;
}

struct CrateVersion {
    major: usize,
    minor: usize,
    patch: usize,
    pre_id: Option<usize>,
}

/// Collect the list of crates that changed
///
/// checks out the repo and moves between tags using the last since tag
async fn changed_crates(start: CrateVersion, end: CrateVersion) {}
