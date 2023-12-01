import { TransactionApi } from "../generated";

/**
 * Wraps the lower-level `TransactionApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class Transaction {
  constructor(public innerClient: TransactionApi) {}
}
