import {
  Configuration,
  ConfigurationParameters,
  LTSApi,
  MempoolApi,
  RequestContext,
  StatusApi,
  StreamApi,
  TransactionApi,
} from "./generated";
import { LTS, Status } from "./subapis";
export * from "./subapis";
export * from "./generated";

interface CoreApiClientSettings {
  basePath: string;
  logicalNetworkName?: string;
  /** On Node.JS < 18, this will need to be provided by a library such as `node-fetch` */
  fetch?: any;
  /** DEPRECATED: Use `advanced.agent`. For use with node-fetch */
  httpAgent?: any;
  /** DEPRECATED: Use `advanced.agent`. For use with node-fetch */
  httpsAgent?: any;
  advanced?: ConfigurationParameters;
}

export class CoreApiClient {
  public Status: Status;
  public LTS: LTS;
  public LowLevel: {
    Status: StatusApi;
    LTS: LTSApi;
    Transaction: TransactionApi;
    Mempool: MempoolApi;
    Stream: StreamApi;
  };

  private constructor(
    configuration: Configuration,
    public logicalNetworkName: string
  ) {
    this.LowLevel = {
      Status: new StatusApi(configuration),
      LTS: new LTSApi(configuration),
      Transaction: new TransactionApi(configuration),
      Mempool: new MempoolApi(configuration),
      Stream: new StreamApi(configuration),
    };
    this.Status = new Status(this.LowLevel.Status, logicalNetworkName);
    this.LTS = new LTS(this.LowLevel.LTS, logicalNetworkName);
  }

  private static constructConfiguration(
    settings: CoreApiClientSettings
  ): Configuration {
    // Left for backward compatibility
    if (settings.httpAgent || settings.httpsAgent) {
      const agentSelector = (parsedUrl: any) => {
        console.log(parsedUrl);
        if (parsedUrl.protocol === "https:") {
          return settings.httpsAgent || settings.httpAgent;
        }
        return settings.httpAgent;
      };
      settings.advanced = {
        ...(settings.advanced || {}),
        agent: agentSelector,
      };
    }

    const parameters: ConfigurationParameters = {
      ...(settings.advanced || {}),
      basePath: settings.basePath,
      fetchApi: settings.fetch,
    };
    return new Configuration(parameters);
  }

  /**
   * Creates a CoreAPIClient.
   * Before returning, this method connects to the Core API to validate the connection and configured network name.
   */
  public static async initialize(
    settings: CoreApiClientSettings
  ): Promise<CoreApiClient> {
    const configuration = CoreApiClient.constructConfiguration(settings);
    const innerStatusApi = new StatusApi(configuration);
    const configurationResponse =
      await innerStatusApi.statusNetworkConfigurationPost();
    let logicalNetworkName = settings.logicalNetworkName;
    if (!logicalNetworkName) {
      logicalNetworkName = configurationResponse.network;
    } else {
      if (configurationResponse.network != logicalNetworkName) {
        throw new Error(
          `Connected to wrong network: expected ${logicalNetworkName} but was ${configurationResponse.network}`
        );
      }
    }
    return new CoreApiClient(configuration, logicalNetworkName);
  }

  /**
   * Creates a CoreAPIClient, without checking that a connection can be established.
   * Requires a logical network name to be provided in the settings.
   */
  public static initializeUnchecked(
    settings: CoreApiClientSettings & { logicalNetworkName: string }
  ): CoreApiClient {
    const configuration = CoreApiClient.constructConfiguration(settings);
    return new CoreApiClient(configuration, settings.logicalNetworkName);
  }
}
