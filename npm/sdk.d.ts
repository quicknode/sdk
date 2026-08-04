// sdk.d.ts
import {
  QuicknodeSdk as _QuicknodeSdk,
  SdkFullConfig,
  WebhookAttributes,
  S3Attributes,
  AzureAttributes,
  PostgresAttributes,
  KafkaAttributes,
  WebhookTemplateId,
  EvmWalletFilterTemplate,
  EvmContractEventsTemplate,
  EvmAbiFilterTemplate,
  SolanaWalletFilterTemplate,
  BitcoinWalletFilterTemplate,
  XrplWalletFilterTemplate,
  HyperliquidWalletEventsFilterTemplate,
  StellarWalletTransactionsFilterTemplate,
  RpcApiClient,
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
  | { destination: "kafka"; attributes: KafkaAttributes };

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

// Webhook template args (input). The inner key is `args` rather than the
// wire's `templateArgs` to avoid `templateArgs.templateArgs.wallets`; the
// Node binding renames it back before the request.
export type TemplateArgsInput =
  | { templateId: "evmWalletFilter"; args: EvmWalletFilterTemplate | EvmWalletFilterByListTemplate }
  | {
      templateId: "evmContractEvents";
      args: EvmContractEventsTemplate | EvmContractEventsByListTemplate;
    }
  | { templateId: "evmAbiFilter"; args: EvmAbiFilterTemplate | EvmAbiFilterByListTemplate }
  | {
      templateId: "solanaWalletFilter";
      args: SolanaWalletFilterTemplate | SolanaWalletFilterByListTemplate;
    }
  | {
      templateId: "bitcoinWalletFilter";
      args: BitcoinWalletFilterTemplate | BitcoinWalletFilterByListTemplate;
    }
  | { templateId: "xrplWalletFilter"; args: XrplWalletFilterTemplate | XrplWalletFilterByListTemplate }
  | {
      templateId: "hyperliquidWalletEventsFilter";
      args: HyperliquidWalletEventsFilterTemplate | HyperliquidWalletEventsFilterByListTemplate;
    }
  | {
      templateId: "stellarWalletTransactionsSourceAccountFilter";
      args: StellarWalletTransactionsFilterTemplate | StellarWalletTransactionsFilterByListTemplate;
    };

// Replace the napi-generated JSON-blob templateArgs with typed unions.
type _CreateWebhookFromTemplateParamsNode = import("./index").CreateWebhookFromTemplateParamsNode;
type _UpdateWebhookTemplateParamsNode = import("./index").UpdateWebhookTemplateParamsNode;

export type CreateWebhookFromTemplateParams =
  Omit<_CreateWebhookFromTemplateParamsNode, "templateArgs"> & {
    templateArgs: TemplateArgsInput;
  };

export type UpdateWebhookTemplateParams =
  Omit<_UpdateWebhookTemplateParamsNode, "templateArgs"> & {
    templateArgs: TemplateArgsInput;
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
  KafkaAttributes,
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
  // account
  AccountSubscription,
  AccountInfo,
  AccountInfoResponse,
  // api credits
  ApiCredit,
  GetApiCreditsResponse,
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
  RateLimitEntry,
  GetRateLimitsData,
  GetRateLimitsResponse,
  // endpoint URLs
  EndpointUrl,
  GetEndpointUrlsData,
  GetEndpointUrlsResponse,
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
  WebhookPageInfo,
  WebhookEnabledCountResponse,
  WebhookDestinationAttributes,
  ActivateWebhookParams,
  EvmWalletFilterTemplate,
  EvmContractEventsTemplate,
  EvmAbiFilterTemplate,
  SolanaWalletFilterTemplate,
  BitcoinWalletFilterTemplate,
  XrplWalletFilterTemplate,
  HyperliquidWalletEventsFilterTemplate,
  StellarWalletTransactionsFilterTemplate,
  EvmWalletFilterByListTemplate,
  EvmContractEventsByListTemplate,
  EvmAbiFilterByListTemplate,
  SolanaWalletFilterByListTemplate,
  BitcoinWalletFilterByListTemplate,
  XrplWalletFilterByListTemplate,
  HyperliquidWalletEventsFilterByListTemplate,
  StellarWalletTransactionsFilterByListTemplate,
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
  // sql
  SqlConfig,
  SqlApiClient,
  QueryResponseNode,
  ColumnMetaNode,
  QueryStatisticsNode,
  ChainSchemaNode,
  TableSchemaNode,
  ColumnSchemaNode,
  // rpc / tooling access
  RpcConfig,
  CachedToken,
  ToolingAccessStatus,
  RpcApiClient,
  // payment lane
  PaymentConfig,
} from "./index";

// A settlement receipt for the crypto-micropayment lane. Returned inside
// `RpcCallResponse.paymentReceipt` by `rpc.callWithReceipt`. `reference` is the
// settlement transaction hash. Present only on the MPP lane; `null` otherwise.
export interface PaymentReceipt {
  method: string;
  status: string;
  timestamp: string;
  reference: string;
}

// The result of `rpc.callWithReceipt`: the JSON-RPC `result` plus the optional
// settlement receipt (`null` for x402 and non-payment lanes).
export interface RpcCallResponse {
  result: any;
  paymentReceipt: PaymentReceipt | null;
}

// ── Payment lanes ──────────────────────────────────────────────
//
// Base-unit amounts are `string`, not `number`: they are u128 in the core and a
// JS number is an f64 that loses precision above 2^53. Pass and store them as
// decimal strings.

// An x402 gateway session (from `rpc.gatewayAuthenticate`). `token` is a live
// bearer credential — persist it, but keep it out of logs.
export interface GatewaySession {
  token: string;
  expUnix: number;
  accountId: string;
}

// An x402 credit balance (`rpc.gatewayCredits` / `rpc.gatewayBuyCredits`).
export interface CreditBalance {
  accountId: string;
  credits: number;
}

// The faucet result (`rpc.gatewayDrip`): the on-chain funding transaction, NOT
// a balance. Call `rpc.gatewayCredits` afterwards to read the new balance.
export interface DripReceipt {
  accountId: string;
  transactionHash: string;
}

// Local state for an open MPP payment channel (`rpc.mppOpen` /
// `rpc.mppTopUp`). Persist this verbatim: the gateway has no read-only channel
// endpoint, so a lost record means opening a new channel.
export interface ChannelState {
  channelId: string;
  token: string;
  payee: string;
  salt: string;
  authorizedSigner: string;
  escrowContract: string;
  /** Base units, decimal string. */
  deposit: string;
  /** Base units, decimal string. */
  cumulativeSpent: string;
  /** The gateway's per-call price, in base units, as a decimal string. */
  perCall: string;
  chainId: number;
}

// The gateway's view of a channel (`rpc.mppStatus`).
export interface ChannelStatus {
  channelId: string;
  /** Base units, decimal string. */
  acceptedCumulative: string;
  /** Base units, decimal string. */
  spent: string;
}

// A freshly generated payment wallet (`generatePaymentWallet`). `key` is the raw
// private key, returned exactly once at generation — nothing in the SDK stores
// or re-derives it, so persist it before discarding the object.
export interface GeneratedWallet {
  address: string;
  chain: "evm" | "svm" | "tempo";
  key: string;
}

/**
 * Generates a fresh payment keypair. Offline: no network call, no funds.
 * Randomness comes from the OS CSPRNG.
 */
export function generatePaymentWallet(chain: "evm" | "svm" | "tempo"): GeneratedWallet;

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

// Retypes napi's `any` templateArgs to the discriminated union. Keep method
// signatures in sync with the napi-generated WebhooksApiClient in ./index.d.ts.
export interface WebhooksApiClientTyped {
  listWebhooks(params?: import("./index").GetWebhooksParams | undefined | null): Promise<import("./index").ListWebhooksResponse>;
  deleteAllWebhooks(): Promise<void>;
  getWebhook(id: string): Promise<import("./index").Webhook>;
  updateWebhook(
    id: string,
    params: import("./index").UpdateWebhookParams
  ): Promise<import("./index").Webhook>;
  deleteWebhook(id: string): Promise<void>;
  pauseWebhook(id: string): Promise<void>;
  activateWebhook(id: string, params: import("./index").ActivateWebhookParams): Promise<void>;
  getEnabledCount(): Promise<import("./index").WebhookEnabledCountResponse>;
  createWebhookFromTemplate(params: CreateWebhookFromTemplateParams): Promise<import("./index").Webhook>;
  updateWebhookTemplate(
    webhookId: string,
    params: UpdateWebhookTemplateParams
  ): Promise<import("./index").Webhook>;
}

// Retypes the query response `data` rows from napi's `any[]` to
// `Record<string, unknown>[]` (rows are objects keyed by the selected columns;
// shape varies per query). Keep method signatures in sync with the
// napi-generated SqlApiClient in ./index.d.ts.
export interface QueryResult extends Omit<QueryResponseNode, "data"> {
  data: Array<Record<string, unknown>>;
}

export interface SqlApiClientTyped {
  query(query: string, clusterId: string): Promise<QueryResult>;
  getSchema(clusterId: string): Promise<ChainSchemaNode>;
}

// Retypes the payment-lane returns from napi's `any` to the interfaces above.
// napi emits `any` for every method returning a `serde_json::Value`, so without
// this the declared payment interfaces would be documentation only and a
// mistyped field would not be an error. Keep method signatures in sync with the
// napi-generated RpcApiClient in ./index.d.ts.
export interface RpcApiClientTyped
  extends Omit<
    RpcApiClient,
    | "callWithReceipt"
    | "gatewayAuthenticate"
    | "gatewayCredits"
    | "gatewayBuyCredits"
    | "gatewayDrip"
    | "gatewayDrawdownCall"
    | "mppOpen"
    | "mppTopUp"
    | "mppClose"
    | "mppStatus"
    | "mppSessionCall"
  > {
  callWithReceipt(
    method: string,
    params?: any | undefined | null,
    network?: string | undefined | null,
    endpointUrl?: string | undefined | null
  ): Promise<RpcCallResponse>;
  gatewayAuthenticate(): Promise<GatewaySession>;
  gatewayCredits(session: GatewaySession): Promise<CreditBalance>;
  gatewayBuyCredits(
    session: GatewaySession,
    network: string
  ): Promise<CreditBalance>;
  gatewayDrip(session: GatewaySession): Promise<DripReceipt>;
  gatewayDrawdownCall(
    method: string,
    session: GatewaySession,
    network: string,
    params?: any | undefined | null
  ): Promise<any>;
  mppOpen(deposit: string): Promise<ChannelState>;
  mppTopUp(
    channel: ChannelState,
    additionalDeposit: string
  ): Promise<ChannelState>;
  mppClose(channel: ChannelState): Promise<void>;
  mppStatus(channel: ChannelState): Promise<ChannelStatus>;
  mppSessionCall(
    method: string,
    network: string,
    channel: ChannelState,
    newCumulative: string,
    params?: any | undefined | null
  ): Promise<any>;
}

export class QuicknodeSdk {
  constructor(config: SdkFullConfig);
  static fromEnv(): QuicknodeSdk;
  admin: _QuicknodeSdk["admin"];
  streams: StreamsApiClientTyped;
  webhooks: WebhooksApiClientTyped;
  kvstore: _QuicknodeSdk["kvstore"];
  sql: SqlApiClientTyped;
  rpc: RpcApiClientTyped;
}

// Typed static factory methods producing each discriminated variant of
// TemplateArgsInput. Each template exposes both an inline form (passing the
// value template) and a by-list form (passing a *ByList template referencing
// a pre-created list by name). Consumers can also construct the object
// literal directly.
export const TemplateArgs: {
  evmWalletFilter(
    attrs: EvmWalletFilterTemplate | EvmWalletFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "evmWalletFilter" }>;
  evmContractEvents(
    attrs: EvmContractEventsTemplate | EvmContractEventsByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "evmContractEvents" }>;
  evmAbiFilter(
    attrs: EvmAbiFilterTemplate | EvmAbiFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "evmAbiFilter" }>;
  solanaWalletFilter(
    attrs: SolanaWalletFilterTemplate | SolanaWalletFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "solanaWalletFilter" }>;
  bitcoinWalletFilter(
    attrs: BitcoinWalletFilterTemplate | BitcoinWalletFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "bitcoinWalletFilter" }>;
  xrplWalletFilter(
    attrs: XrplWalletFilterTemplate | XrplWalletFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "xrplWalletFilter" }>;
  hyperliquidWalletEventsFilter(
    attrs: HyperliquidWalletEventsFilterTemplate | HyperliquidWalletEventsFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "hyperliquidWalletEventsFilter" }>;
  stellarWalletTransactionsFilter(
    attrs:
      | StellarWalletTransactionsFilterTemplate
      | StellarWalletTransactionsFilterByListTemplate
  ): Extract<TemplateArgsInput, { templateId: "stellarWalletTransactionsSourceAccountFilter" }>;
};

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
export class RpcError extends QuicknodeError {
  code: number;
}
// Payment-lane errors (crypto-micropayment `rpc.call`). Catch PaymentError to
// handle them all. PaymentIndeterminateError means the paid request was sent
// but its response was lost — the payment MAY have settled, so do NOT retry.
export class PaymentError extends QuicknodeError {}
export class PaymentUnsupportedError extends PaymentError {}
export class PaymentRejectedError extends PaymentError {
  status: number;
  body: string;
}
export class PaymentIndeterminateError extends PaymentError {}
