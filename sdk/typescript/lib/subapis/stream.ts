import { StreamApi } from "../generated";

/**
 * Wraps the lower-level `StreamApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class Stream {
  constructor(public innerClient: StreamApi) {}
}
