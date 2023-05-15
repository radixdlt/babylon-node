import {
  StatusApi,
  NetworkConfigurationResponse,
  NetworkStatusResponse,
} from "../generated";

/**
 * Wraps the lower-level `StatusApi` - which can be accessed with `innerClient` for advanced use cases.
 */
export class Status {
  constructor(
    public innerClient: StatusApi,
    public logicalNetworkName: string
  ) {}

  public async getNetworkConfiguration(): Promise<NetworkConfigurationResponse> {
    return this.innerClient.statusNetworkConfigurationPost();
  }

  public async getNetworkStatus(): Promise<NetworkStatusResponse> {
    return this.innerClient.statusNetworkStatusPost({
      networkStatusRequest: {
        network: this.logicalNetworkName,
      },
    });
  }
}
