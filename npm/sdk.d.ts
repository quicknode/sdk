// sdk.d.ts
import {
  QuickNodeSdk as _QuickNodeSdk,
  SdkFullConfig,
  WebhookAttributes,
  S3Attributes,
  AzureAttributes,
  PostgresAttributes,
  MysqlAttributes,
  MongoAttributes,
  ClickhouseAttributes,
  SnowflakeAttributes,
  KafkaAttributes,
  RedisAttributes,
  WebhookTemplateId,
  EvmWalletFilterTemplate,
  EvmContractEventsTemplate,
  EvmAbiFilterTemplate,
  SolanaWalletFilterTemplate,
  BitcoinWalletFilterTemplate,
  XrplWalletFilterTemplate,
  HyperliquidWalletEventsFilterTemplate,
  StellarWalletTransactionsFilterTemplate,
} from "./index";

// Stream destination attributes (input). The inner key is `attributes` rather
// than the wire's `destination_attributes` to avoid
// `destinationAttributes.destinationAttributes.url`; the Node binding renames
// it back before the request.
export type StreamDestinationAttributesInput =
  | { destination: "webhook"; attributes: WebhookAttributes }
  | { destination: "s3"; attributes: S3Attributes }
  | { destination: "azure"; attributes: AzureAttributes }
  | { destination: "postgres"; attributes: PostgresAttributes }
  | { destination: "mysql"; attributes: MysqlAttributes }
  | { destination: "mongo"; attributes: MongoAttributes }
  | { destination: "clickhouse"; attributes: ClickhouseAttributes }
  | { destination: "snowflake"; attributes: SnowflakeAttributes }
  | { destination: "kafka"; attributes: KafkaAttributes }
  | { destination: "redis"; attributes: RedisAttributes };

// Stream destination attributes (response) — matches the wire shape.
export type StreamDestinationAttributesResponse =
  | { destination: "webhook"; destination_attributes: WebhookAttributes }
  | { destination: "s3"; destination_attributes: S3Attributes }
  | { destination: "azure"; destination_attributes: AzureAttributes }
  | { destination: "postgres"; destination_attributes: PostgresAttributes }
  | { destination: "mysql"; destination_attributes: MysqlAttributes }
  | { destination: "mongo"; destination_attributes: MongoAttributes }
  | { destination: "clickhouse"; destination_attributes: ClickhouseAttributes }
  | { destination: "snowflake"; destination_attributes: SnowflakeAttributes }
  | { destination: "kafka"; destination_attributes: KafkaAttributes }
  | { destination: "redis"; destination_attributes: RedisAttributes };

// Replace the napi-generated JSON-blob destinationAttributes with typed unions.
type _CreateStreamParamsNode = import("./index").CreateStreamParamsNode;
type _UpdateStreamParamsNode = import("./index").UpdateStreamParamsNode;
type _StreamNode = import("./index").StreamNode;
type _ListStreamsResponseNode = import("./index").ListStreamsResponseNode;

export type CreateStreamParams =
  Omit<_CreateStreamParamsNode, "destinationAttributes"> & {
    destinationAttributes: StreamDestinationAttributesInput;
  };

export type UpdateStreamParams =
  Omit<_UpdateStreamParamsNode, "destinationAttributes"> & {
    destinationAttributes?: StreamDestinationAttributesInput;
  };

export type Stream = Omit<_StreamNode, "destinationAttributes"> & {
  destinationAttributes?: StreamDestinationAttributesResponse;
};

export type ListStreamsResponse = Omit<_ListStreamsResponseNode, "data"> & {
  data: Stream[];
};

export type {
  SdkFullConfig,
  HttpConfig,
  AdminConfig,
  StreamsConfig,
  // streams
  ListStreamsParams,
  PageInfo,
  TestFilterParams,
  TestFilterResponse,
  EnabledCountResponse,
  WebhookAttributes,
  S3Attributes,
  AzureAttributes,
  PostgresAttributes,
  MysqlAttributes,
  MongoAttributes,
  ClickhouseAttributes,
  SnowflakeAttributes,
  KafkaAttributes,
  RedisAttributes,
  AddressBookConfig,
  StreamsApiClient,
  // billing
  InvoiceLine,
  Invoice,
  ListInvoicesResponse,
  ListInvoicesData,
  Payment,
  ListPaymentsResponse,
  ListPaymentsData,
  // chains
  ChainNetwork,
  Chain,
  ListChainsResponse,
  // metrics
  GetEndpointMetricsRequest,
  GetAccountMetricsRequest,
  EndpointMetric,
  GetEndpointMetricsResponse,
  GetAccountMetricsResponse,
  // rate limits
  MethodRateLimiter,
  GetMethodRateLimitsData,
  GetMethodRateLimitsResponse,
  CreateMethodRateLimitRequest,
  CreateMethodRateLimitResponse,
  UpdateMethodRateLimitRequest,
  UpdateMethodRateLimitResponse,
  RateLimitSettings,
  UpdateRateLimitsRequest,
  // security options
  SecurityOption,
  GetSecurityOptionsResponse,
  SecurityOptionsUpdate,
  UpdateSecurityOptionsRequest,
  UpdateSecurityOptionsResponse,
  CreateReferrerRequest,
  CreateIpRequest,
  CreateDomainMaskRequest,
  CreateJwtRequest,
  CreateRequestFilterRequest,
  CreateRequestFilterResponse,
  CreateRequestFilterData,
  UpdateRequestFilterRequest,
  CreateOrUpdateIpCustomHeaderRequest,
  IpCustomHeaderData,
  CreateOrUpdateIpCustomHeaderResponse,
  DeleteBoolResponse,
  // endpoints
  GetEndpointsRequest,
  GetEndpointsResponse,
  Endpoint,
  EndpointTag,
  CreateEndpointRequest,
  CreateEndpointResponse,
  SingleEndpoint,
  EndpointRateLimits,
  EndpointSecurity,
  EndpointSecurityOptions,
  EndpointIpCustomHeaderOption,
  EndpointToken,
  EndpointJwt,
  EndpointReferrer,
  EndpointDomainMask,
  EndpointIp,
  EndpointRequestFilter,
  ShowEndpointResponse,
  UpdateEndpointRequest,
  UpdateEndpointStatusRequest,
  UpdateEndpointStatusResponse,
  CreateTagRequest,
  // logs
  GetEndpointLogsRequest,
  LogDetails,
  EndpointLog,
  GetEndpointLogsResponse,
  GetLogDetailsResponse,
  // teams
  TeamUser,
  TeamSummary,
  TeamDetail,
  ListTeamsResponse,
  CreateTeamRequest,
  CreateTeamData,
  CreateTeamResponse,
  GetTeamResponse,
  DeleteTeamData,
  DeleteTeamResponse,
  TeamEndpoint,
  ListTeamEndpointsResponse,
  UpdateTeamEndpointsRequest,
  UpdateTeamEndpointsData,
  UpdateTeamEndpointsResponse,
  InviteTeamMemberRequest,
  InviteTeamMemberResponse,
  RemoveTeamMemberRequest,
  TeamMessageData,
  RemoveTeamMemberResponse,
  ResendTeamInviteResponse,
  // usage
  GetUsageRequest,
  UsageData,
  GetUsageResponse,
  EndpointUsage,
  MethodUsage,
  ChainUsage,
  UsageByEndpointData,
  GetUsageByEndpointResponse,
  UsageByMethodData,
  GetUsageByMethodResponse,
  UsageByChainData,
  GetUsageByChainResponse,
  // webhooks
  WebhooksConfig,
  GetWebhooksParams,
  UpdateWebhookParams,
  Webhook,
  ListWebhooksResponse,
  WebhookEnabledCountResponse,
  WebhookDestinationAttributes,
  ActivateWebhookParams,
  CreateWebhookFromTemplateParams,
  UpdateWebhookTemplateParams,
  EvmWalletFilterTemplate,
  EvmContractEventsTemplate,
  EvmAbiFilterTemplate,
  SolanaWalletFilterTemplate,
  BitcoinWalletFilterTemplate,
  XrplWalletFilterTemplate,
  HyperliquidWalletEventsFilterTemplate,
  StellarWalletTransactionsFilterTemplate,
  WebhooksApiClient,
  // kvstore
  KvStoreConfig,
  KvStoreApiClient,
  KvSetEntry,
  CreateSetParams,
  GetSetsParams,
  GetSetsResponse,
  GetSetResponse,
  BulkSetsParams,
  CreateListParams,
  GetListsParams,
  GetListsData,
  GetListsResponse,
  GetListParams,
  GetListData,
  GetListResponse,
  UpdateListParams,
  AddListItemParams,
  ListContainsItemResponse,
} from "./index";

// const enums must use `export` (not `export type`) so they are usable as values
export {
  StreamRegion,
  StreamDataset,
  StreamDestination,
  FilterLanguage,
  StreamMetadataLocation,
  ProductType,
  StreamStatus,
  WebhookTemplateId,
  WebhookStartFrom,
} from "./index";

// Retypes napi's `any` destination_attributes to the discriminated unions.
export interface StreamsApiClientTyped {
  createStream(params: CreateStreamParams): Promise<Stream>;
  listStreams(params?: import("./index").ListStreamsParams | undefined | null): Promise<ListStreamsResponse>;
  deleteAllStreams(): Promise<void>;
  getStream(id: string): Promise<Stream>;
  updateStream(id: string, params: UpdateStreamParams): Promise<Stream>;
  deleteStream(id: string): Promise<void>;
  activateStream(id: string): Promise<void>;
  pauseStream(id: string): Promise<void>;
  testFilter(params: import("./index").TestFilterParams): Promise<import("./index").TestFilterResponse>;
  getEnabledCount(streamType?: string | undefined | null): Promise<import("./index").EnabledCountResponse>;
}

export class QuickNodeSdk {
  constructor(config: SdkFullConfig);
  static fromEnv(): QuickNodeSdk;
  admin: _QuickNodeSdk["admin"];
  streams: StreamsApiClientTyped;
  webhooks: _QuickNodeSdk["webhooks"];
  kvstore: _QuickNodeSdk["kvstore"];
}

export class TemplateArgs {
  templateId: WebhookTemplateId;
  value: string;
  static evmWalletFilter(attrs: EvmWalletFilterTemplate): TemplateArgs;
  static evmContractEvents(attrs: EvmContractEventsTemplate): TemplateArgs;
  static evmAbiFilter(attrs: EvmAbiFilterTemplate): TemplateArgs;
  static solanaWalletFilter(attrs: SolanaWalletFilterTemplate): TemplateArgs;
  static bitcoinWalletFilter(attrs: BitcoinWalletFilterTemplate): TemplateArgs;
  static xrplWalletFilter(attrs: XrplWalletFilterTemplate): TemplateArgs;
  static hyperliquidWalletEventsFilter(attrs: HyperliquidWalletEventsFilterTemplate): TemplateArgs;
  static stellarWalletTransactionsFilter(attrs: StellarWalletTransactionsFilterTemplate): TemplateArgs;
}
