use napi::bindgen_prelude::*;
use napi_derive::napi;
use quicknode_sdk as core;

// napi(object) cannot represent the flattened TemplateArgs enum on core's
// webhook params, so these node-facing params carry template_args as
// serde_json::Value.
//
// The Node input shape is `{ templateId, args: {...} }` — the inner key is
// renamed from the API's `templateArgs` to `args` to avoid
// `templateArgs.templateArgs.wallets` in TypeScript. node_ta_to_core()
// renames it back before deserializing.
//
// Unlike streams_destination.rs, the inner keys are NOT case-converted.
// Webhook template structs in core are modeled around the API wire format
// (camelCase: e.g. EvmContractEventsTemplate carries
// `#[serde(rename_all = "camelCase")]` so it expects `eventHashes`, not
// `event_hashes`). All other template structs only have single-word fields
// that are identical in either case. Snake-casing the input here would
// silently drop multi-word fields like `eventHashes`.

pub(crate) fn node_ta_to_core(v: serde_json::Value) -> Result<core::webhooks::TemplateArgs> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn evm_contract_events_preserves_event_hashes_through_outbound_wire() {
        let input = json!({
            "templateId": "evmContractEvents",
            "args": {
                "contracts": ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
                "eventHashes": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                ],
            },
        });

        let parsed = node_ta_to_core(input).unwrap();
        let core::webhooks::TemplateArgs::EvmContractEvents(t) = &parsed else {
            unreachable!("expected EvmContractEvents variant")
        };
        assert_eq!(
            t.event_hashes.as_deref(),
            Some(
                [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                        .to_string()
                ]
                .as_slice()
            ),
        );

        let params = core::webhooks::CreateWebhookFromTemplateParams {
            name: "t".to_string(),
            network: "ethereum-mainnet".to_string(),
            notification_email: None,
            destination_attributes: core::webhooks::WebhookDestinationAttributes {
                url: "https://x".to_string(),
                security_token: None,
                compression: None,
            },
            template_args: parsed,
        };
        let outbound = serde_json::to_value(&params).unwrap();
        let template_args = outbound.get("templateArgs").unwrap();
        assert_eq!(
            template_args["eventHashes"][0].as_str(),
            Some("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
        );
        assert!(template_args.get("event_hashes").is_none());
    }

    #[test]
    fn evm_wallet_filter_single_word_field_still_works() {
        let input = json!({
            "templateId": "evmWalletFilter",
            "args": { "wallets": ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"] },
        });
        let parsed = node_ta_to_core(input).unwrap();
        assert!(matches!(
            parsed,
            core::webhooks::TemplateArgs::EvmWalletFilter(_)
        ));
    }
}
