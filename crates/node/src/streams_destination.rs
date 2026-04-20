use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

// ── Node-side stream wrappers ──────────────────────────────────────────────
//
// Core CreateStreamParams, UpdateStreamParams, Stream, and ListStreamsResponse
// lost #[napi(object)] because they flatten a Rust enum (DestinationAttributes).
// These napi-side wrappers mirror those shapes but hold destination_attributes
// as serde_json::Value, so TypeScript consumers can use a discriminated union
// `{ destination: "webhook", attributes: { ... } }` (sdk.d.ts). Conversion to
// and from the core enum happens in into_core / from_core.
//
// Wire asymmetry: the API wire format is
// `{ destination, destination_attributes: {...} }`. The Node input shape uses
// `{ destination, attributes: {...} }` (nested key renamed to `attributes`)
// because double-nested `destination_attributes.destination_attributes` is
// awkward in TypeScript. This layer renames the inner key to match core.

fn node_da_to_core(v: serde_json::Value) -> Result<core::streams::DestinationAttributes> {
    let mut obj = match v {
        serde_json::Value::Object(o) => o,
        _ => {
            return Err(Error::from_reason(
                "destination_attributes must be an object".to_string(),
            ))
        }
    };
    let attrs = obj.remove("attributes").ok_or_else(|| {
        Error::from_reason("destination_attributes.attributes is required".to_string())
    })?;
    obj.insert("destination_attributes".to_string(), attrs);
    let wire = serde_json::Value::Object(obj);
    serde_json::from_value::<core::streams::DestinationAttributes>(wire)
        .map_err(|e| Error::from_reason(format!("invalid destination_attributes: {e}")))
}

fn core_da_to_node(attrs: &core::streams::DestinationAttributes) -> Result<serde_json::Value> {
    // Core serde serializes as { destination, destination_attributes } per the
    // wire format. Node responses preserve the wire shape so consumers match
    // the TypeScript `DestinationAttributesResponse` type in sdk.d.ts.
    serde_json::to_value(attrs)
        .map_err(|e| Error::from_reason(format!("failed to serialize destination_attributes: {e}")))
}

#[napi(object)]
pub struct CreateStreamParamsNode {
    pub name: String,
    pub region: core::streams::StreamRegion,
    pub network: String,
    pub dataset: core::streams::StreamDataset,
    pub start_range: i64,
    pub end_range: i64,
    // Shape: { destination: "webhook", attributes: { url, ... } }
    pub destination_attributes: serde_json::Value,
    pub plan: String,
    pub threshold_fetch_buffer: i64,
    pub dataset_batch_size: Option<i64>,
    pub max_batch_size: Option<i64>,
    pub max_buffer_range_size: Option<i64>,
    pub max_buffer_processing_workers: Option<i64>,
    pub keep_distance_from_tip: Option<i64>,
    pub filter_function: Option<String>,
    pub filter_language: Option<core::streams::FilterLanguage>,
    pub address_book_config: Option<core::streams::AddressBookConfig>,
    pub include_stream_metadata: Option<core::streams::StreamMetadataLocation>,
    pub product_type: Option<core::streams::ProductType>,
    pub status: Option<core::streams::StreamStatus>,
    pub notification_email: Option<String>,
    pub charge_min_cap: Option<i32>,
    pub fix_block_reorgs: Option<i32>,
    pub elastic_batch_enabled: Option<bool>,
}

impl CreateStreamParamsNode {
    pub fn into_core(self) -> Result<core::streams::CreateStreamParams> {
        let destination_attributes = node_da_to_core(self.destination_attributes)?;
        Ok(core::streams::CreateStreamParams {
            name: self.name,
            region: self.region,
            network: self.network,
            dataset: self.dataset,
            start_range: self.start_range,
            end_range: self.end_range,
            destination_attributes,
            plan: self.plan,
            threshold_fetch_buffer: self.threshold_fetch_buffer,
            dataset_batch_size: self.dataset_batch_size,
            max_batch_size: self.max_batch_size,
            max_buffer_range_size: self.max_buffer_range_size,
            max_buffer_processing_workers: self.max_buffer_processing_workers,
            keep_distance_from_tip: self.keep_distance_from_tip,
            filter_function: self.filter_function,
            filter_language: self.filter_language,
            address_book_config: self.address_book_config,
            include_stream_metadata: self.include_stream_metadata,
            product_type: self.product_type,
            status: self.status,
            notification_email: self.notification_email,
            charge_min_cap: self.charge_min_cap,
            fix_block_reorgs: self.fix_block_reorgs,
            elastic_batch_enabled: self.elastic_batch_enabled,
        })
    }
}

#[napi(object)]
#[derive(Default)]
pub struct UpdateStreamParamsNode {
    pub name: Option<String>,
    pub region: Option<core::streams::StreamRegion>,
    pub network: Option<String>,
    pub dataset: Option<core::streams::StreamDataset>,
    pub start_range: Option<i64>,
    pub end_range: Option<i64>,
    pub destination_attributes: Option<serde_json::Value>,
    pub plan: Option<String>,
    pub threshold_fetch_buffer: Option<i64>,
    pub dataset_batch_size: Option<i64>,
    pub max_batch_size: Option<i64>,
    pub max_buffer_range_size: Option<i64>,
    pub max_buffer_processing_workers: Option<i64>,
    pub keep_distance_from_tip: Option<i64>,
    pub filter_function: Option<String>,
    pub filter_language: Option<core::streams::FilterLanguage>,
    pub address_book_config: Option<core::streams::AddressBookConfig>,
    pub include_stream_metadata: Option<core::streams::StreamMetadataLocation>,
    pub notification_email: Option<String>,
    pub charge_min_cap: Option<i32>,
    pub fix_block_reorgs: Option<i32>,
    pub elastic_batch_enabled: Option<bool>,
    pub status: Option<core::streams::StreamStatus>,
    pub memo: Option<String>,
}

impl UpdateStreamParamsNode {
    pub fn into_core(self) -> Result<core::streams::UpdateStreamParams> {
        let destination_attributes = self
            .destination_attributes
            .map(node_da_to_core)
            .transpose()?;
        Ok(core::streams::UpdateStreamParams {
            name: self.name,
            region: self.region,
            network: self.network,
            dataset: self.dataset,
            start_range: self.start_range,
            end_range: self.end_range,
            destination_attributes,
            plan: self.plan,
            threshold_fetch_buffer: self.threshold_fetch_buffer,
            dataset_batch_size: self.dataset_batch_size,
            max_batch_size: self.max_batch_size,
            max_buffer_range_size: self.max_buffer_range_size,
            max_buffer_processing_workers: self.max_buffer_processing_workers,
            keep_distance_from_tip: self.keep_distance_from_tip,
            filter_function: self.filter_function,
            filter_language: self.filter_language,
            address_book_config: self.address_book_config,
            include_stream_metadata: self.include_stream_metadata,
            notification_email: self.notification_email,
            charge_min_cap: self.charge_min_cap,
            fix_block_reorgs: self.fix_block_reorgs,
            elastic_batch_enabled: self.elastic_batch_enabled,
            status: self.status,
            memo: self.memo,
        })
    }
}

#[napi(object)]
pub struct StreamNode {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub sequence: i64,
    pub network: String,
    pub dataset: String,
    pub region: String,
    pub start_range: i64,
    pub end_range: i64,
    pub plan: Option<String>,
    pub threshold_fetch_buffer: Option<i64>,
    pub dataset_batch_size: Option<i64>,
    pub max_batch_size: Option<i64>,
    pub max_buffer_range_size: Option<i64>,
    pub max_buffer_processing_workers: Option<i64>,
    pub keep_distance_from_tip: Option<i64>,
    pub filter_function: Option<String>,
    pub filter_language: Option<String>,
    pub include_stream_metadata: Option<String>,
    pub product_type: Option<String>,
    pub notification_email: Option<String>,
    pub fix_block_reorgs: Option<i32>,
    pub current_hash: Option<String>,
    // Shape: { destination: "webhook", destination_attributes: {...} } (matches wire format)
    pub destination_attributes: Option<serde_json::Value>,
    pub elastic_batch_enabled: Option<bool>,
    pub qn_account_id: Option<String>,
    pub charge_min_cap: Option<i32>,
    pub memo: Option<String>,
    pub address_book_config: Option<core::streams::AddressBookConfig>,
}

impl StreamNode {
    pub fn from_core(s: core::streams::Stream) -> Result<Self> {
        let destination_attributes = match &s.destination_attributes {
            Some(attrs) => Some(core_da_to_node(attrs)?),
            None => None,
        };
        Ok(Self {
            id: s.id,
            name: s.name,
            status: s.status,
            created_at: s.created_at,
            updated_at: s.updated_at,
            sequence: s.sequence,
            network: s.network,
            dataset: s.dataset,
            region: s.region,
            start_range: s.start_range,
            end_range: s.end_range,
            plan: s.plan,
            threshold_fetch_buffer: s.threshold_fetch_buffer,
            dataset_batch_size: s.dataset_batch_size,
            max_batch_size: s.max_batch_size,
            max_buffer_range_size: s.max_buffer_range_size,
            max_buffer_processing_workers: s.max_buffer_processing_workers,
            keep_distance_from_tip: s.keep_distance_from_tip,
            filter_function: s.filter_function,
            filter_language: s.filter_language,
            include_stream_metadata: s.include_stream_metadata,
            product_type: s.product_type,
            notification_email: s.notification_email,
            fix_block_reorgs: s.fix_block_reorgs,
            current_hash: s.current_hash,
            destination_attributes,
            elastic_batch_enabled: s.elastic_batch_enabled,
            qn_account_id: s.qn_account_id,
            charge_min_cap: s.charge_min_cap,
            memo: s.memo,
            address_book_config: s.address_book_config,
        })
    }
}

#[napi(object)]
pub struct ListStreamsResponseNode {
    pub data: Vec<StreamNode>,
    pub page_info: core::streams::PageInfo,
}

impl ListStreamsResponseNode {
    pub fn from_core(resp: core::streams::ListStreamsResponse) -> Result<Self> {
        let mut data = Vec::with_capacity(resp.data.len());
        for s in resp.data {
            data.push(StreamNode::from_core(s)?);
        }
        Ok(Self {
            data,
            page_info: resp.page_info,
        })
    }
}
