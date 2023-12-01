import {
  LTSApi,
  LtsCommittedTransactionOutcome,
  LtsStateAccountAllFungibleResourceBalancesRequest,
  LtsStateAccountAllFungibleResourceBalancesResponse,
  LtsStateAccountDepositBehaviourResponse,
  LtsStateAccountFungibleResourceBalanceRequest,
  LtsStateAccountFungibleResourceBalanceResponse,
  LtsStreamAccountTransactionOutcomesRequest,
  LtsStreamAccountTransactionOutcomesResponse,
  LtsStreamTransactionOutcomesRequest,
  LtsStreamTransactionOutcomesResponse,
  LtsTransactionConstructionResponse,
  LtsTransactionStatusRequest,
  LtsTransactionStatusResponse,
  LtsTransactionSubmitPriorityThresholdNotMetErrorDetails,
  LtsTransactionSubmitRejectedErrorDetails,
  LtsTransactionSubmitRequest,
  LtsTransactionSubmitResponse,
  ResponseError,
} from "../generated";

type LTSSubmitResult =
  | { result: "Success"; response: LtsTransactionSubmitResponse }
  | { result: "Error"; message: string; error: ResponseError }
  | {
      result: "Rejected";
      details: LtsTransactionSubmitRejectedErrorDetails;
      error: ResponseError;
    }
  | {
      result: "PriorityThresholdNotMet";
      details: LtsTransactionSubmitPriorityThresholdNotMetErrorDetails;
      error: ResponseError;
    };

/**
 * Wraps the lower-level `LTSApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class LTS {
  constructor(public innerClient: LTSApi, public logicalNetworkName: string) {}

  /**
   * This method returns the current epoch, for use in transaction construction.
   *
   * Unless `acceptableSyncDelaySeconds` is set to `null`,
   * this method will throw an error if the node is not synced up within
   * `acceptableSyncDelaySeconds` (defaults to 120) of the current time.
   *
   * @returns metadata for transaction construction - such as the current epoch
   */
  public async getConstructionMetadata(parameters?: {
    acceptableSyncDelaySeconds: number | null;
  }): Promise<LtsTransactionConstructionResponse> {
    const { acceptableSyncDelaySeconds = 120 } = parameters ?? {};
    const response = await this.innerClient.ltsTransactionConstructionPost({
      ltsTransactionConstructionRequest: {
        network: this.logicalNetworkName,
      },
    });
    if (!!acceptableSyncDelaySeconds) {
      const acceptableSyncDelayMs = acceptableSyncDelaySeconds * 1000;
      if (
        response.ledger_clock.unix_timestamp_ms + acceptableSyncDelayMs <
        Date.now()
      ) {
        throw new Error(
          `Node is currently only synced up till ${response.ledger_clock.date_time}, and not synced within acceptable delay of ${acceptableSyncDelaySeconds} seconds`
        );
      }
    }
    return response;
  }

  /**
   * Checks whether the given account is currently configured to accept the deposits of the given
   * resources - in other words, this method returns whether such `try_deposit()` call would be
   * successful, and why.
   *
   * @returns Details on the account's settings related to accepting deposits.
   */
  public async getAccountDepositBehaviour(
    targetAccountAddress: string,
    depositedResourceAddresses: Array<string>
  ): Promise<LtsStateAccountDepositBehaviourResponse> {
    return this.innerClient.ltsStateAccountDepositBehaviourPost({
      ltsStateAccountDepositBehaviourRequest: {
        network: this.logicalNetworkName,
        account_address: targetAccountAddress,
        resource_addresses: depositedResourceAddresses,
      },
    });
  }

  /**
   * Submits the transaction. Returns a result from the API.
   *
   * @param notarized_transaction_hex - the notarized transaction payload for submission, encoded as hex
   * @returns a union of possible results - match on the `result` field to determine which one:
   * - `Success` - the transaction was submitted successfully
   * - `Error` - the request errored for some reason
   * - `Rejected` - the transaction was rejected by the node for some reason
   * - `MempoolFull` - the transaction was rejected by the node because the mempool is full
   *    and the submitted transaction wasn't able to evict any existing mempool transactions
   */
  public async submitTransaction(
    request: Omit<LtsTransactionSubmitRequest, "network">
  ): Promise<LTSSubmitResult> {
    try {
      const response = await this.innerClient.ltsTransactionSubmitPost({
        ltsTransactionSubmitRequest: {
          network: this.logicalNetworkName,
          ...request,
        },
      });
      return {
        result: "Success",
        response,
      };
    } catch (e) {
      if (
        e instanceof ResponseError &&
        e.errorResponse?.error_type == "LtsTransactionSubmit"
      ) {
        const details = e.errorResponse.details;
        if (!details) {
          return {
            result: "Error",
            message: e.errorResponse.message,
            error: e,
          };
        }
        if (details.type == "PriorityThresholdNotMet") {
          return {
            result: details.type,
            details,
            error: e,
          };
        }
        if (details.type == "Rejected") {
          return {
            result: details.type,
            details,
            error: e,
          };
        }
      }
      throw e;
    }
  }

  public async getTransactionStatus(
    request: Omit<LtsTransactionStatusRequest, "network">
  ): Promise<LtsTransactionStatusResponse> {
    return this.innerClient.ltsTransactionStatusPost({
      ltsTransactionStatusRequest: {
        network: this.logicalNetworkName,
        ...request,
      },
    });
  }

  public async getAccountFungibleResourceBalance(
    request: Omit<LtsStateAccountFungibleResourceBalanceRequest, "network">
  ): Promise<LtsStateAccountFungibleResourceBalanceResponse> {
    return this.innerClient.ltsStateAccountFungibleResourceBalancePost({
      ltsStateAccountFungibleResourceBalanceRequest: {
        network: this.logicalNetworkName,
        ...request,
      },
    });
  }

  public async getAccountAllFungibleResourceBalances(
    request: Omit<LtsStateAccountAllFungibleResourceBalancesRequest, "network">
  ): Promise<LtsStateAccountAllFungibleResourceBalancesResponse> {
    return this.innerClient.ltsStateAccountAllFungibleResourceBalancesPost({
      ltsStateAccountAllFungibleResourceBalancesRequest: {
        network: this.logicalNetworkName,
        ...request,
      },
    });
  }

  public async getTransactionOutcome({
    state_version,
  }: {
    state_version: number;
  }): Promise<LtsCommittedTransactionOutcome | undefined> {
    const response = await this.getTransactionOutcomes({
      from_state_version: state_version,
      limit: 1,
    });
    return response.committed_transaction_outcomes[0];
  }

  public async getTransactionOutcomes(
    request: Omit<LtsStreamTransactionOutcomesRequest, "network">
  ): Promise<LtsStreamTransactionOutcomesResponse> {
    return this.innerClient.ltsStreamTransactionOutcomesPost({
      ltsStreamTransactionOutcomesRequest: {
        network: this.logicalNetworkName,
        ...request,
      },
    });
  }

  public async getAccountTransactionOutcomes(
    request: Omit<LtsStreamAccountTransactionOutcomesRequest, "network">
  ): Promise<LtsStreamAccountTransactionOutcomesResponse> {
    return this.innerClient.ltsStreamAccountTransactionOutcomesPost({
      ltsStreamAccountTransactionOutcomesRequest: {
        network: this.logicalNetworkName,
        ...request,
      },
    });
  }
}
