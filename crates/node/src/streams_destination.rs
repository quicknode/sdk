use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

// napi(object) cannot represent the flattened DestinationAttributes enum on
// core's stream types, so these wrappers carry it as serde_json::Value.
//
// The Node input shape is `{ destination, attributes: {...} }` — the inner
// key is renamed from the API's `destination_attributes` to `attributes` to
// avoid `destinationAttributes.destinationAttributes.url` in TypeScript.
// node_da_to_core() renames it back before deserializing. Responses keep the
// wire shape (no rename) since consumers usually just read them.
//
// Keys in the inner attributes object also need case conversion: TypeScript
// callers write camelCase (maxRetry), but core's serde structs expect
// snake_case (max_retry). napi does this automatically for #[napi(object)]
// structs, but a raw serde_json::Value bypasses that — so we walk the inner
// object here.

fn camel_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.push(c.to_ascii_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn convert_keys<F: Fn(&str) -> String + Copy>(v: serde_json::Value, f: F) -> serde_json::Value {
    match v {
        serde_json::Value::Object(o) => serde_json::Value::Object(
            o.into_iter()
                .map(|(k, v)| (f(&k), convert_keys(v, f)))
                .collect(),
        ),
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.into_iter().map(|v| convert_keys(v, f)).collect())
        }
        other => other,
    }
}

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
        Error::from_reason("destinationAttributes.attributes is required".to_string())
    })?;
    // Convert attribute keys (camelCase → snake_case) so they match the core
    // struct field names. The outer `destination` key is already a string enum
    // tag and does not need conversion.
    let attrs = convert_keys(attrs, camel_to_snake);
    obj.insert("destination_attributes".to_string(), attrs);
    let wire = serde_json::Value::Object(obj);
    serde_json::from_value::<core::streams::DestinationAttributes>(wire)
        .map_err(|e| Error::from_reason(format!("invalid destination_attributes: {e}")))
}

fn node_extras_to_core(
    items: Option<Vec<serde_json::Value>>,
) -> Result<Option<Vec<core::streams::DestinationAttributes>>> {
    items
        .map(|v| v.into_iter().map(node_da_to_core).collect())
        .transpose()
}

fn core_extras_to_node(
    items: &Option<Vec<core::streams::DestinationAttributes>>,
) -> Result<Option<Vec<serde_json::Value>>> {
    items
        .as_ref()
        .map(|v| v.iter().map(core_da_to_node).collect())
        .transpose()
}

fn core_da_to_node(attrs: &core::streams::DestinationAttributes) -> Result<serde_json::Value> {
    let v = serde_json::to_value(attrs).map_err(|e| {
        Error::from_reason(format!("failed to serialize destination_attributes: {e}"))
    })?;
    // Rename the inner wire key `destination_attributes` -> `attributes` and
    // camelCase its keys so the response shape matches the input shape
    // (`{ destination, attributes }`), letting TS consumers round-trip a
    // response back into an update without renaming.
    let serde_json::Value::Object(mut obj) = v else {
        return Ok(v);
    };
    if let Some(inner) = obj.remove("destination_attributes") {
        obj.insert(
            "attributes".to_string(),
            convert_keys(inner, snake_to_camel),
        );
    }
    Ok(serde_json::Value::Object(obj))
}

#[napi(object)]
pub struct CreateStreamParamsNode {
    pub name: String,
    pub region: core::streams::StreamRegion,
    pub network: String,
    pub dataset: core::streams::StreamDataset,
    pub start_range: i64,
    pub end_range: i64,
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
    // Each element carries the same { destination, attributes } shape as the
    // primary destination; we rewrite to core's wire format in node_da_to_core.
    pub extra_destinations: Option<Vec<serde_json::Value>>,
}

impl CreateStreamParamsNode {
    pub fn into_core(self) -> Result<core::streams::CreateStreamParams> {
        let destination_attributes = node_da_to_core(self.destination_attributes)?;
        let extra_destinations = node_extras_to_core(self.extra_destinations)?;
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
            extra_destinations,
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
    pub extra_destinations: Option<Vec<serde_json::Value>>,
}

impl UpdateStreamParamsNode {
    pub fn into_core(self) -> Result<core::streams::UpdateStreamParams> {
        let destination_attributes = self
            .destination_attributes
            .map(node_da_to_core)
            .transpose()?;
        let extra_destinations = node_extras_to_core(self.extra_destinations)?;
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
            extra_destinations,
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
    pub destination_attributes: Option<serde_json::Value>,
    pub elastic_batch_enabled: Option<bool>,
    pub qn_account_id: Option<String>,
    pub charge_min_cap: Option<i32>,
    pub memo: Option<String>,
    pub address_book_config: Option<core::streams::AddressBookConfig>,
    pub extra_destinations: Option<Vec<serde_json::Value>>,
}

impl StreamNode {
    pub fn from_core(s: core::streams::Stream) -> Result<Self> {
        let destination_attributes = match &s.destination_attributes {
            Some(attrs) => Some(core_da_to_node(attrs)?),
            None => None,
        };
        let extra_destinations = core_extras_to_node(&s.extra_destinations)?;
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
            extra_destinations,
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
