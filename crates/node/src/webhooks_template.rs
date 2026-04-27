use napi::bindgen_prelude::*;
use napi_derive::napi;
use quicknode_sdk as core;

use crate::key_case::{camel_to_snake, convert_keys};

// napi(object) cannot represent the flattened TemplateArgs enum on core's
// webhook params, so these node-facing params carry template_args as
// serde_json::Value.
//
// The Node input shape is `{ templateId, args: {...} }` — the inner key is
// renamed from the API's `templateArgs` to `args` to avoid
// `templateArgs.templateArgs.wallets` in TypeScript. node_ta_to_core()
// renames it back before deserializing.
//
// Keys inside `args` also need case conversion: TypeScript callers write
// camelCase (eventHashes), but core's serde structs expect snake_case
// (event_hashes). napi does this automatically for #[napi(object)] structs,
// but a raw serde_json::Value bypasses that — so we walk the inner object
// here.

fn node_ta_to_core(v: serde_json::Value) -> Result<core::webhooks::TemplateArgs> {
    let mut obj = match v {
        serde_json::Value::Object(o) => o,
        _ => {
            return Err(Error::from_reason(
                "template_args must be an object".to_string(),
            ))
        }
    };
    let args = obj
        .remove("args")
        .ok_or_else(|| Error::from_reason("templateArgs.args is required".to_string()))?;
    let args = convert_keys(args, camel_to_snake);
    // Core's tag key is `templateId`, content key is `templateArgs`. Input
    // already has `templateId`; rename `args` -> `templateArgs`.
    obj.insert("templateArgs".to_string(), args);
    let wire = serde_json::Value::Object(obj);
    serde_json::from_value::<core::webhooks::TemplateArgs>(wire)
        .map_err(|e| Error::from_reason(format!("invalid template_args: {e}")))
}

#[napi(object)]
pub struct CreateWebhookFromTemplateParamsNode {
    pub name: String,
    pub network: String,
    pub notification_email: Option<String>,
    pub destination_attributes: core::webhooks::WebhookDestinationAttributes,
    pub template_args: serde_json::Value,
}

impl CreateWebhookFromTemplateParamsNode {
    pub fn into_core(self) -> Result<core::webhooks::CreateWebhookFromTemplateParams> {
        let template_args = node_ta_to_core(self.template_args)?;
        Ok(core::webhooks::CreateWebhookFromTemplateParams {
            name: self.name,
            network: self.network,
            notification_email: self.notification_email,
            destination_attributes: self.destination_attributes,
            template_args,
        })
    }
}

#[napi(object)]
pub struct UpdateWebhookTemplateParamsNode {
    pub name: Option<String>,
    pub notification_email: Option<String>,
    pub destination_attributes: Option<core::webhooks::WebhookDestinationAttributes>,
    pub template_args: serde_json::Value,
}

impl UpdateWebhookTemplateParamsNode {
    pub fn into_core(self) -> Result<core::webhooks::UpdateWebhookTemplateParams> {
        let template_args = node_ta_to_core(self.template_args)?;
        Ok(core::webhooks::UpdateWebhookTemplateParams {
            name: self.name,
            notification_email: self.notification_email,
            destination_attributes: self.destination_attributes,
            template_args,
        })
    }
}
