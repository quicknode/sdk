// sdk.d.ts
import {
  QuicknodeSdk as _QuicknodeSdk,
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

// Stream destination attributes (response). Mirrors the input shape so a
// response can be round-tripped back into an update call without renaming.
// The Node binding renames the wire `destination_attributes` key to
// `attributes` on the way out.
export type StreamDestinationAttributesResponse = StreamDestinationAttributesInput;

// Replace the napi-generated JSON-blob destinationAttributes with typed unions.
type _CreateStreamParamsNode = import("./index").CreateStreamParamsNode;
type _UpdateStreamParamsNode = import("./index").UpdateStreamParamsNode;
type _StreamNode = import("./index").StreamNode;
type _ListStreamsResponseNode = import("./index").ListStreamsResponseNode;

export type CreateStreamParams =
  Omit<_CreateStreamParamsNode, "destinationAttributes" | "extraDestinations"> & {
    destinationAttributes: StreamDestinationAttributesInput;
    extraDestinations?: StreamDestinationAttributesInput[] | null;
  };

export type UpdateStreamParams =
  Omit<_UpdateStreamParamsNode, "destinationAttributes" | "extraDestinations"> & {
    destinationAttributes?: StreamDestinationAttributesInput;
    extraDestinations?: StreamDestinationAttributesInput[] | null;
  };

export type Stream = Omit<_StreamNode, "destinationAttributes" | "extraDestinations"> & {
  destinationAttributes?: StreamDestinationAttributesResponse;
  extraDestinations?: StreamDestinationAttributesResponse[] | null;
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
  Pagination,
  GetEndpointSecurityResponse,
  // bulk
  BulkOperationResult,
  BulkUpdateEndpointStatusRequest,
  BulkUpdateEndpointStatusData,
  BulkUpdateEndpointStatusResponse,
  BulkTag,
  BulkAddTagRequest,
  BulkAddTagData,
  BulkAddTagResponse,
  BulkRemoveTagRequest,
  BulkRemoveTagData,
  BulkRemoveTagResponse,
  // account tags
  AccountTag,
  ListTagsData,
  ListTagsResponse,
  RenameTagRequest,
  RenameTagResponse,
  DeleteAccountTagData,
  DeleteAccountTagResponse,
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
  TagUsage,
  UsageByTagData,
  GetUsageByTagResponse,
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
// NOTE: keep these method signatures in sync with the napi-generated
// StreamsApiClient in ./index.d.ts. Adding a method to crates/node/src/lib.rs
// requires adding it here too; there is no automated check.
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

export class QuicknodeSdk {
  constructor(config: SdkFullConfig);
  static fromEnv(): QuicknodeSdk;
  admin: _QuicknodeSdk["admin"];
  streams: StreamsApiClientTyped;
  webhooks: _QuicknodeSdk["webhooks"];
  kvstore: _QuicknodeSdk["kvstore"];
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

// Typed error hierarchy. Any SDK call can throw one of these; catch
// QuicknodeError to handle them all, or a specific subclass for finer control.
export class QuicknodeError extends Error {}
export class ConfigError extends QuicknodeError {}
export class HttpError extends QuicknodeError {}
export class TimeoutError extends HttpError {}
export class ConnectionError extends HttpError {}
export class ApiError extends QuicknodeError {
  status: number;
  body: string;
}
export class DecodeError extends QuicknodeError {
  body: string;
}
