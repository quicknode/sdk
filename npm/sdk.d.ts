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
  StreamDestination,
} from "./index";

export type {
  SdkFullConfig,
  HttpConfig,
  AdminConfig,
  StreamsConfig,
  // streams
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
  CreateStreamParams,
  Stream,
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
} from "./index";

export class QuickNodeSdk {
  constructor(config: SdkFullConfig);
  static fromEnv(): QuickNodeSdk;
  admin: _QuickNodeSdk["admin"];
  streams: _QuickNodeSdk["streams"];
}

export class DestinationAttributes {
  destination: StreamDestination;
  value: string;
  static webhook(attrs: WebhookAttributes): DestinationAttributes;
  static s3(attrs: S3Attributes): DestinationAttributes;
  static azure(attrs: AzureAttributes): DestinationAttributes;
  static postgres(attrs: PostgresAttributes): DestinationAttributes;
  static mysql(attrs: MysqlAttributes): DestinationAttributes;
  static mongo(attrs: MongoAttributes): DestinationAttributes;
  static clickhouse(attrs: ClickhouseAttributes): DestinationAttributes;
  static snowflake(attrs: SnowflakeAttributes): DestinationAttributes;
  static kafka(attrs: KafkaAttributes): DestinationAttributes;
  static redis(attrs: RedisAttributes): DestinationAttributes;
}
