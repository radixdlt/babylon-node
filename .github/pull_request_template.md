> [!IMPORTANT]
>
> * Please read our [Contributing Guidelines](https://github.com/radixdlt/babylon-node/blob/main/CONTRIBUTING.md) before opening a PR.
> * Before creating your PR, please ensure you read the [branching strategy](https://github.com/radixdlt/babylon-node/blob/main/docs/branching-strategy.md). The end result after completing the merge actions should be that `main <= release/XXX <= develop`, where `XXX` is the latest released protocol version. This ensures that we minimise merge conflicts, and that work doesn't go missing.
> * As per the branching strategy, **you must ensure you select the _correct base branch_**, both for branching from, and in the PR UI above. The following process can be used to decide your branch:
>   * For most code changes, choose `develop`. Code against the `develop` branch will be released at the next protocol update. 
>   * For code changes which need to go out as a fully-interoperable update to the node at the current protocol version, use the current `release/XXX` branch.
>     * Such changes will be reviewed more carefully to mitigate the risk of regression.
>     * Once the change is merged, it is the merger's responsibility to ensure `release/XXX` is merged into the `develop` branch.
>   * For github workflow changes, use the current `main` branch.
>     * Once the change is merged, it is the merger's responsibility to ensure `main` is merged into both `release/XXX` and `develop`, so that the changes also apply to hotfixes, and to current development.
>   * For changes to README files, use the `main` base branch.
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
