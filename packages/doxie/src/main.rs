//! A single entrypoint, but with different actions depending on input

use std::{
    collections::{HashMap, HashSet},
    future::Future,
    path::PathBuf,
};

use doxie_types::*;
use git2::{Commit, Oid, Repository, Revwalk};
use tokio::process::Command;

mod workflow;

const OUTPUT_DIR: &str = "data";

#[tokio::main]
async fn main() {
    let root = PathBuf::from("/Users/jonkelley/Development/dioxus");

    // For now, just write to the stats cache as the default
    save_stats_as_artifact(root).await;
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

/// Collect all the open PRs across the various repos
async fn all_open_prs() {
    let repos = [
        "dioxuslabs/dioxus",
        "dioxuslabs/dioxus-template",
        "dioxuslabs/docsite",
        "dioxuslabs/blitz",
        "dioxuslabs/components",
        "dioxuslabs/sdk",
        "dioxuslabs/collect-assets",
        "dioxuslabs/include_mdbook",
        "dioxuslabs/example-projects",
        "dioxuslabs/awesome-dioxus",
        "jkelleyrtp/stylo-dioxus",
    ];

    let mut saved_repos = HashMap::new();
    let octocrab = octocrab::instance();

    for repo in repos {
        let (owner, repo) = repo.split_once('/').unwrap();

        let prs = octocrab
            .pulls(owner, repo)
            .list()
            .per_page(100)
            .state(octocrab::params::State::Open)
            .send()
            .await;

        if let Ok(prs) = prs {
            saved_repos.insert(
                repo.to_string(),
                OpenPrs {
                    repo: repo.to_string(),
                    prs: prs.into_iter().collect(),
                },
            );
        } else {
            eprintln!("Failed to get PRs for {}", repo);
        }
    }

    let blob = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&OpenPrMap { prs: saved_repos }).unwrap()
    } else {
        serde_json::to_string(&OpenPrMap { prs: saved_repos }).unwrap()
    };

    let out_dir = OUTPUT_DIR.parse::<PathBuf>().unwrap();
    std::fs::write(out_dir.join("open_prs.json"), blob).unwrap();
}

#[tokio::test]
async fn collect_open_prs() {
    all_open_prs().await;
}

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

#[tokio::test]
async fn changed_prs__() {
    let octocrab = octocrab::instance();

    changed_on_prs(&octocrab).await;
}

async fn changed_on_prs(api: &octocrab::Octocrab) {
    use octocrab::params::State as PrState;

    // Quickly check if this PR has any artifacts with cached data.
    // This should just let us skip any work we need to do. Eventually all the PRs will have this work
    // done for them automatically in the workflow.
    let prs = api
        .pulls("dioxuslabs", "dioxus")
        .list()
        .state(PrState::Open)
        .per_page(100)
        .send()
        .await
        .unwrap();

    for pr in prs {
        println!(
            "PR [{login}] {title} - {id}\n{body}\n",
            id = pr.id,
            body = pr.body.as_deref().unwrap_or_default(),
            title = pr.title.as_deref().unwrap_or_default(),
            login = &pr.user.as_ref().unwrap().login
        );
    }
}

/// Use the github API to figure out what packages have been changed by a given PR
///
/// This checks out the branch of the given PR, diffs it against HEAD and then saves the changed files
/// We only want the *changed files since HEAD* so we have an idea of what is changing for a given PR.
///
/// Eventually we want to save this for cargo semver checks and then run the checks.
///
/// Alternatively, we could run the diffing here on the PR itself and save it as an artifact which this
/// bot reads.
async fn changed_on_pr(api: &octocrab::Octocrab, pr_id: usize) {}

/// Whenever a PR is merged, we kick off the workflow to build the stats for the new main branch and then
/// save that as a single unified blob to the repo.
///
/// This lets the dioxuslabs.com status page read the blob and display stats dynamically without running
/// into API rate limits.
///
/// We should also try to implement some sort of caching/versioning CDN-like mechanism so we don't
/// run into issues. GH gives us 12.5k req/hr which could add up in DDOS scenario
async fn save_stats_as_artifact(path: PathBuf) {
    // For now, collect all the PRs just for 0.4 and 0.5
    let repo = Repository::open(path).unwrap();
    changed_crates_on_repo(&repo);

    // And then list open PRs
    all_open_prs().await;
}

fn changed_crates_on_repo(repo: &Repository) {
    let changed = ChangedVersions {
        version: vec![
            (4, collect_prs_for_minor_version(repo, 4)),
            (5, collect_prs_for_minor_version(repo, 5)),
        ]
        .into_iter()
        .collect(),
    };

    let blob = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&changed).unwrap()
    } else {
        serde_json::to_string(&changed).unwrap()
    };

    let out_dir = OUTPUT_DIR.parse::<PathBuf>().unwrap();

    std::fs::write(out_dir.join("commits.json"), blob).unwrap();
}

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

fn sha_from_tag(repo: &Repository, target_tag: &str) -> Option<Oid> {
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

    id
}

#[derive(Debug)]
struct Pr<'a> {
    commit: Commit<'a>,

    /// The `#id` of the PR - so you can go to github.com/dioxuslabs/dioxus/pull/id
    id: Option<usize>,

    changed_files: HashSet<PathBuf>,

    idx: usize,
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
        let mut out: HashSet<String> = self
            .changed_crates()
            .iter()
            .map(|path| {
                path.iter()
                    .skip(1)
                    .take(1)
                    .map(|os_str| os_str.to_str().unwrap().to_string())
                    .collect()
            })
            .collect();

        let mut out: Vec<String> = out.into_iter().collect();

        out.sort();

        out
    }
}

/// Manually look at the shas between versions
async fn collect_changed_crates(repo: PathBuf) {
    // Run through each tag starting at 0.5.0 and increment the patch version
    // We're trying to get a sense of which crates changed between versions
    // We want to link these to PRs by checking the git log for "merged" PR logs
    // git log --first-parent/ / to get shas for every PR in the past

    let repo = Repository::open(repo).unwrap();
    let start_id = sha_from_tag(&repo, "v0.5.0").unwrap();
    let end_id = sha_from_tag(&repo, "v0.5.1").unwrap();

    collect_pr_between(&repo, end_id, start_id);
}

/// Walk all the tags between the two minor versions and collect the PRs for each release
///
/// so if we're going from 0.5.0 to 0.5.1, we'll collect all the PRs between those two tags
///
/// minor_version would be 5 in this case
/// Does not cover prereleases - only releases in the form
fn collect_prs_for_minor_version(repo: &Repository, minor_version: usize) -> MinorVersionChanged {
    // we're going to march forward version by version until the tag doesn't show up and then give up
    // Kinda dumb but it keeps the structure simple enough
    let mut patch_version = 0;
    let mut patch_versions = vec![];

    loop {
        let start_tag = format!("v0.{}.{}", minor_version, patch_version);
        let end_tag = format!("v0.{}.{}", minor_version, patch_version + 1);

        let start_id = sha_from_tag(repo, &start_tag).unwrap();
        let end_id = sha_from_tag(repo, &end_tag);

        // If there's no end_id, that was the last tag for this minor version
        // If there's a v0.6.0, then we're done
        // If there's not v0.6.0, then we should accumulate the remaining changes as the "next" version
        if end_id.is_none() {
            println!("No end tag for {}", end_tag);

            // todo: Also handle prereleases in the form of alpha-0
            let next_minor = format!("v0.{}.0", minor_version + 1);
            if sha_from_tag(repo, &next_minor).is_some() {
                break;
            }

            println!("No next minor version found, attempting to collect from HEAD");

            // If there's no next minor version, then the remaining tags are for *this* version
            // The idea being that once breaking changes exist in the form of a new minor version, we
            // stop collecting PRs for the previous version
            //
            // todo: this doesn't work for the workflows where we migrate changes onto stable branches
            // while also simultaneously working on the next version in main
            // I think all we need to do is just mark if this PR was backported and then provide that
            // as a filter option
            let end_id = repo.head().unwrap().target();
            let commits = collect_pr_between(repo, end_id.unwrap(), start_id);
            patch_versions.push(PatchVersionChanged {
                commits,
                version: patch_version,
                published: false,
            });
            break;
        }

        let commits = collect_pr_between(repo, end_id.unwrap(), start_id);
        patch_versions.push(PatchVersionChanged {
            commits,
            version: patch_version,
            published: true,
        });

        patch_version += 1;
    }

    MinorVersionChanged {
        patch_versions,
        version: minor_version,
    }
}

#[test]
fn collects_prs_for_5() {
    let repo = Repository::open(
        "/Users/jonkelley/Development/dioxus"
            .parse::<PathBuf>()
            .unwrap(),
    )
    .unwrap();
    collect_prs_for_minor_version(&repo, 5);
}

fn collect_pr_between(repo: &Repository, end_id: Oid, start_id: Oid) -> Vec<PrCommit> {
    let mut prs = vec![];

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push(end_id).unwrap();
    _ = revwalk.simplify_first_parent();

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
    for (idx, commit) in commits.iter().enumerate() {
        let Ok(parent) = commit.parent(0) else {
            continue;
        };

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
            idx,
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

        // usually a merge commit will contain the ID in the form of (#id) or "Merge pull request #id from user/branch"
        // try and find the first match (from the end of the string) that matches that
        // A bit stupid but look for the `#` character and then try and parse the number after it
        // first one wins
        for part in summary.split_ascii_whitespace().rev() {
            if let Some(id) = part.strip_prefix("(#") {
                pr.id = Some(id.trim_end_matches(")").parse().unwrap());
                break;
            }

            if let Some(id) = part.strip_prefix("#") {
                pr.id = Some(id.parse().unwrap());
                break;
            }
        }

        prs.push(pr);
    }

    prs.iter()
        .map(|pr| PrCommit {
            summary: pr.commit.summary().unwrap().to_string(),
            id: pr.id,
            changed_packages: pr.changed_packages().into_iter().collect(),
            commit_hash: pr.commit.id().to_string(),
            head_index: pr.idx,
        })
        .collect()
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
