use std::collections::HashSet;

use dioxus::prelude::*;
use doxie_types::{
    octocrab_models::pulls::PullRequest, ChangedVersions, MinorVersionChanged, OpenPrMap,
    PatchVersionChanged, PrCommit,
};

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        div {
            h1 { "Open PRs across the dioxus org:" }
            OpenPrs {}
        }

        div {
            h1 { "Prs with changes" }
            PrsWithChanges {}
        }
    }
}

fn OpenPrs() -> Element {
    let data = use_signal(|| {
        let raw = include_str!("../../../data/open_prs.json");
        let open_prs: OpenPrMap = serde_json::from_str(raw).expect("failed to parse open_prs.json");

        let mut sorted_pr_list = open_prs.prs.into_values().collect::<Vec<_>>();

        sorted_pr_list.sort_by(|a, b| a.prs.len().cmp(&b.prs.len()).reverse());

        // now also sort the PRs in each repo by last updated time
        for prs in sorted_pr_list.iter_mut() {
            prs.prs
                .sort_by(|a, b| a.updated_at.cmp(&b.updated_at).reverse());
        }

        let render_pr = move |pr: &mut PullRequest| {
            let pr_num = pr.number;

            rsx! {
                li { class: "pr-item",
                    input { r#type: "checkbox" }
                    div {
                        a {
                            class: "pr-title",
                            class: if pr.draft.unwrap_or_default() { "draft" },
                            href: pr.html_url.as_ref().map(|f| f.to_string()).unwrap_or_default(),
                            "#{pr_num} {pr.title.as_deref().unwrap_or_default()}"
                        }
                        div { class: "pr-meta",
                            h3 {
                                "Updated at: {pr.updated_at.map(|f| f.to_string()).unwrap_or_default()}"
                            }
                            h4 {
                                "Author - {pr.user.as_ref().map(|f| f.login.to_string()).unwrap_or_default()}"
                            }
                            pre { "Description: {pr.body.as_deref().unwrap_or_default()}" }
                        }
                    }
                }
            }
        };

        rsx! {
            ul {
                for mut pr in sorted_pr_list {
                    h3 { "{pr.repo}" }
                    for pr in pr.prs.iter_mut() {
                        {render_pr(pr)}
                    }
                }
            }
        }
    });

    rsx! {
        {data()}
    }
}

fn PrsWithChanges() -> Element {
    let data = use_signal(|| {
        let raw = include_str!("../../../data/commits.json");
        let commits: ChangedVersions =
            serde_json::from_str(raw).expect("failed to parse commits.json");

        let mut versions = commits.version.into_values().collect::<Vec<_>>();

        // sort so we get the most recent version
        versions.sort_by(|a, b| a.version.cmp(&b.version).reverse());

        let render_minor = move |version: MinorVersionChanged| {
            let minor_version = version.version;

            let render_patch = move |patch: PatchVersionChanged| {
                let render_commit = move |commit: PrCommit| {
                    // note that we're ignore direct commits to main... could get confusing
                    let id = commit.id?;

                    rsx! {
                        li { class: "pr-item",
                            a { href: "https://github.com/dioxuslabs/dioxus/pull/{id}",
                                "{commit.summary}"
                            }
                        }
                    }
                };

                let changed_packages = patch
                    .commits
                    .iter()
                    .flat_map(|commit| commit.changed_packages.iter())
                    .collect::<HashSet<&String>>();

                let mut changed_packages = changed_packages.into_iter().collect::<Vec<_>>();
                changed_packages.sort();

                rsx! {
                    div {
                        h4 {
                            "v0.{minor_version}.{patch.version}"
                            match patch.published {
                                true => rsx!{ span { class: "published", " - (Published)" } },
                                false => rsx!{ span { class: "unpublished", " - (Unpublished)" } },
                            }
                        }
                        div { class: "changed-packages",
                            div { "Changed packages: " }
                            div { class: "inline-changed-package-list",
                                for package in changed_packages {
                                    a {
                                        href: "https://github.com/dioxuslabs/dioxus/tree/main/packages/{package}",
                                        target: "_blank",
                                        "{package},"
                                    }
                                    " "
                                }
                            }
                        }
                        ul {
                            for commit in patch.commits {
                                {render_commit(commit)}
                            }
                        }
                    }
                }
            };

            rsx! {
                div {
                    for patch in version.patch_versions.into_iter().rev() {
                        {render_patch(patch)}
                    }
                }
            }
        };

        rsx! {
            for version in versions {
                {render_minor(version)}
            }
        }
    });

    rsx! {
        {data()}
    }
}
