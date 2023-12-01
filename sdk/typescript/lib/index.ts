import {
  Configuration,
  ConfigurationParameters,
  LTSApi,
  MempoolApi,
  RequestContext,
  StateApi,
  StatusApi,
  StreamApi,
  TransactionApi,
} from "./generated";
import { LTS, Mempool, State, Status, Stream, Transaction } from "./subapis";
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
  public LTS: LTS;
  public lts: LTS;
  public mempool: Mempool;
  public state: State;
  public status: Status;
  public stream: Stream;
  public transaction: Transaction;
  public lowLevel: {
    lts: LTSApi;
    mempool: MempoolApi;
    state: StateApi;
    status: StatusApi;
    stream: StreamApi;
    transaction: TransactionApi;
  };

  private constructor(
    configuration: Configuration,
    public logicalNetworkName: string
  ) {
    this.lowLevel = {
      lts: new LTSApi(configuration),
      mempool: new MempoolApi(configuration),
      state: new StateApi(configuration),
      status: new StatusApi(configuration),
      stream: new StreamApi(configuration),
      transaction: new TransactionApi(configuration),
    };
    this.lts = new LTS(this.lowLevel.lts, logicalNetworkName);
    this.LTS = this.lts; // NOTE: this is to keep backwards compatibility
    this.mempool = new Mempool(this.lowLevel.mempool);
    this.state = new State(this.lowLevel.state);
    this.status = new Status(this.lowLevel.status, logicalNetworkName);
    this.stream = new Stream(this.lowLevel.stream);
    this.transaction = new Transaction(this.lowLevel.transaction);
  }

  private static constructConfiguration(
    settings: CoreApiClientSettings
  ): Configuration {
    // Left for backward compatibility
    if (settings.httpAgent || settings.httpsAgent) {
      const agentSelector = (parsedUrl: any) => {
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
