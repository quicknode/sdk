// ESM wrapper: re-exports the CJS entry via dynamic import.
// The native .node binary loader (index.js) uses require() for platform
// detection and cannot be expressed as static ESM — this wrapper bridges
// the gap for ESM consumers while keeping the loader intact.
//
// We re-export everything from the default export so that new types added
// to sdk.js are automatically available here without manual updates.
import cjs from './sdk.js';

export default cjs;
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
