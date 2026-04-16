// ESM wrapper: re-exports the CJS entry via dynamic import.
// The native .node binary loader (index.js) uses require() for platform
// detection and cannot be expressed as static ESM — this wrapper bridges
// the gap for ESM consumers while keeping the loader intact.
import cjs from './sdk.js';

export const {
  QuickNodeSdk,
  DestinationAttributes,
  TemplateArgs,
  StreamRegion,
  StreamDataset,
  StreamDestination,
  FilterLanguage,
  StreamMetadataLocation,
  ProductType,
  StreamStatus,
  WebhookTemplateId,
  WebhookStartFrom,
  AdminApiClient,
  StreamsApiClient,
  WebhooksApiClient,
  KvStoreApiClient,
} = cjs;

export default cjs;
