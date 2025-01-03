> [!IMPORTANT]
>
> * Please read our [Contributing Guidelines](/CONTRIBUTING.md) before opening a PR.
> * Before creating your PR, please ensure you read the [branching strategy](/docs/branching-strategy.md). The end result after completing the merge actions should be that `main <= release/XXX <= develop`, where `XXX` is the latest released protocol version. This ensures that we minimise merge conflicts, and that work doesn't go missing.
> * As per the branching strategy, **you must ensure you select the _correct base branch_**, both for branching from, and in the PR UI above. The following process can be used to decide the base branch:
>   * For code changes which can wait until the next protocol update to be released, use `develop`. This should be the default for code changes.
>   * For code changes which need to go out as a fully-interoperable update to the node at the current protocol version, use `release/XXX`.
>     * Such changes must be tested and reviewed more carefully to mitigate the risk of regression.
>     * Once the change is merged, it is the merger's responsibility to ensure `release/XXX` is merged into the `develop` branch.
>   * For github workflow changes, use `main`.
>     * Once the change is merged, it is the merger's responsibility to ensure `main` is merged into both `release/XXX` and `develop`, so that the changes also apply to hotfixes, and to current development.
>   * For changes to README files, use `main`.
>     * Once the change is merged, it is the merger's responsibility to ensure `main` is merged into both `release/XXX` and `develop`, so that the changes also apply on these branches.
> 
> _Please remove this section once you confirm you follow its guidance._

## Summary

<!--
> [!TIP]
> 
> Start with the context of your PR. Why are you making this change? What does it address? Link back to an issue if relevant.
> 
> Then summarise the changes that were made.
> * Bullet points are fine.
> * Feel free to add additional subheadings (using ###) with more information if required.
-->

## Testing

<!--
> [!TIP]
> 
> Explain what testing / verification is done, including manual testing or automated testing.
-->
