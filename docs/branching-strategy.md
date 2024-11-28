
# Branching Strategy

Once you have read the [contributing guide](../CONTRIBUTING.md), if you want to start development, you will need to know which branches to use.

> [!NOTE]
> The currently supported base branches are:
> * `develop` - The primary development branch for features and enhancements to be released at the next protocol version.
> * `release/XXX` = `release/cuttlefish` - The lasest published protocol version.
> * `main` - A pointer to the code in the latest release.
> 
> When clicking merge on a PR to one of these base branches `B`, it is your duty to ensure that a new PR is raised from `B` to all base branches above `B` in the list.

## Summary of approach

### The base branches

We use a variant of `git-flow`, where there are three types of base branches: the `main`, `develop`, and `release/*` branches.

* The `develop` branch is the primary integration branch, for work targeting the next protocol version.
* The `release/*` branches are for all named protocol versions. Only the latest released `release/XXX` branch is **currently supported**. In exceptional scenarios, we may release a hot-fix to the previous release branch.
* The `main` branch is the public-facing base branch and **represents the last official release**. It's also used for docs.

> [!IMPORTANT]
> The development/merge process aims to ensure that there is a strict ordering on the supported base branches under the "contains all changes" partial order. This ensures that work ends up in the right place, we minimise merge conflicts, and that work doesn't go missing.
>
> The order (from latest/most downstream to earliest/most upstream) is as follows:
> * `develop`
> * The currently supported `release/XXX` branch
> * `main`

### Development/Merge process

When working on changes:
* You will first need to select the correct base branch to create your feature branch from. For some epics, it is acceptable to choose a long running `feature/*` or `epic/*` branch as your base, to break up the work into separate reviews.
* Your branch should start `feature/*`, or variants on naming such as `hotfix/*`, `tweak/*`, `docs/*` are permitted. The specific name should be prefixed by a JIRA ticket or Github issue reference where appropriate, e.g. `feature/NODE-123-my-feature` for JIRA tickets or `feature/gh-1235-my-feature` for github issues.
* When you raise a PR, you will need to ensure you select the appropriate base branch before creating the PR. **The default, `main` is typically not correct!**

> [!IMPORTANT]
> 
> Finally, when a PR is merged, it is **the PR merger's responsibility** to ensure that the _base branch_ that was merged into is then merged into _all downstream base branches_ (ideally one by one, like a waterfall).
>
> If there is a merge conflict, this should be handled by creating a special `conflict/X-into-Y-DATE` branch (for branches `X`, `Y` and `DATE`) from `X`, and putting in a PR with a merge target of `Y`.
>
> But if this process is properly followed, such merge conflicts will be rare. 

## Which base branch should I use for my change?

### Code changes

For most code changes, choose `develop`. Code against the `develop` branch will be released at the next protocol update.

For code changes which need to go out as a fully-interoperable update to the node at the current protocol version, use the current `release/XXX` branch. Such changes will be reviewed more carefully to mitigate the risk of regression. Once the change is merged, it is the merger's responsibility to ensure `release/XXX` is merged into the develop branch.

### Stand-alone README changes

Public facing docs change unrelated to another ticket should use a base branch of `main` - as this is the branch which is first visible when someone looks at the repository. Once the change is merged, it is the merger's responsibility to ensure `main` is merged into both `release/XXX` and `develop` branches to avoid merge conflicts or confusion.


### Workflow / CI changes

For github workflow changes, start by branching off of and merging to the current `main` branch.

Once the change is merged, it is the merger's responsibility to ensure `main` is merged into both `release/XXX` and `develop`, so that the changes also apply for current development, and for any hotfixes which need to be built and release.

## Merge or Rebase/Cherry-pick?

This strategy relies on the fact that we always merge.

We avoid rebases after publicly pushing a branch / seeking a review because:

* Rebases cause potential conflicts with other people's work on the same branches, overwrite the history of the project and overwrite any GPG signed commits from other developers
* Rebases result in more merge conflicts
* Various other benefits discussed in the below section.

We acknowledge the weakness of merging that this can make the git history messier to display.

At merge time, it is acceptable but not recommended to squash-merge. We encourage developers to instead squash commits before asking for a review. This results in a better record of the review / iteration process.

## Diagram

The following demonstrates a possible branch structure under this strategy, under the hypothetical scenario where `bottlenose` is the current live protocol version, and `cuttlefish` is being prepared but not yet live.

Admittedly, this isn't particularly easy to follow. The key with this strategy is following the rules. If the rules are followed, you don't need to visualize the structure.

![Diagram summarising the branching strategy](./branching-diagram.png)

## Why do we follow this model?

In order to support a network built upon deterministic execution of the radix engine, we need to have a very clear policy of what is compatible with what. This is where the protocol version strategy comes in. And this maps to git via the `release/*` branch strategy.

The rest of the strategy is motivated by the following benefits:
* We only have a single PR to review changes on
* We avoid clashes where conflicting PRs are merged into develop and main at the same time (this used to keep happening, particularly with devops workflow changes, and was a pain to resolve)
* We only have one commit for a given change in the commit tree.
* It makes it trivial to avoid regressions due to forgetting to merge branches - because we can simply check that all base branches X before Y are entirely merged into Y when a release is prepared.

