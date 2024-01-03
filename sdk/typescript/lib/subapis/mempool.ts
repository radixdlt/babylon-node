import { MempoolApi } from "../generated";

/**
 * Wraps the lower-level `MempoolApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class Mempool {
  constructor(public innerClient: MempoolApi) {}
}
