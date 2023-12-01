import { StateApi } from "../generated";

/**
 * Wraps the lower-level `StateApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class State {
  constructor(public innerClient: StateApi) {}
}
