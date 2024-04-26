# Doxie: a helpful robot that automates the dioxus org

## Motivation:

Managing a large github org can take a lot of work. New PRs, comments, milestones, bugs, and features can take a lot of mental overhead to manage.

This project is designed to help specifically with large open source Rust projects (though it could likely work for other projects...) that follow a (currently hardcoded) repository layout. The goal here is to help us ship faster with fewer semver issues.

The pain points we *specifically* are trying to address:

- Accidentally publishing semver-incompatible changes
- Outdated documentation and doc tests marked as ignore
- Keeping track of changes to crates so we know when to release

## Features:
- [ ] Status page with outstanding issues for current milestone
- [ ] Current benchmarks of the current milestone and any open PRs
- [ ] Charts of perf, bundle size, build times, and more over time
- [ ] Helpful commands for backporting, coverage, benchmarks, welcome comments, etc
- [ ] Comments on open PRs
- [x] Shows which crates have changed and which commits have changed which crates
- [ ] Reading docs with LLM and then checking if those still make sense given the actual API
- [ ] Automatically generating doctests with AI based on examples


## Open Questions
does this need to be a bot, could just be a script?

should we host a custom ghpages or can we just write to the comment?

should we have a floating PR, issue, discussion, a dedicated webpage?


## Resources
https://github.com/taiki-e/upload-rust-binary-action
https://github.com/dtolnay/cargo-llvm-lines/
git log --first-parent // to get shas for every PR in the past
https://stackoverflow.com/questions/52812007/git-log-command-to-extract-pull-request-titles
https://medium.com/hostspaceng/triggering-workflows-in-another-repository-with-github-actions-4f581f8e0ceb

## Open qs

- do we use a matrix test to do everything in parallel and then re-merge or just make this is a single workflow?
- where we do store these types so the docsite can pick them up? crates.io? just pull this repo?
- commenting functionality?
- retention - we would like to just keep updating the same stats file. some other approaches involve saving things to a gh-pages repo or dumping the data into a separate repo altogether. if we just dumped an artifact into a separate repo or branch, we could rely on that instead of artifacts. the 90 day limit means that it might disappear.
- okay yes, for retention we dump into a repo. it would be nice to dump into the docsite itself, but that might get really noisy.
- also should we run benchmarks automatically or only when queued? IE "I think this change might have some impact on performance, let's run a benchmark and see if it's worth it"

## Design

- Whenever the main repo runs its benchmarks, we want it to dump its outputs into a folder in a separate repo
- The docsite should be able to read the data from that repo and display it
- This workflow provides a script that runs the benchmarks and dumps the data into a separate repo
- This workflow also provides other functionality like commenting on open PRs

