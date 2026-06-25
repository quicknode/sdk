use quicknode_sdk::{errors::SdkError, sql::QueryParams, QuicknodeSdk, SdkFullConfig};

const CLUSTER_ID: &str = "hyperliquid-core-mainnet";

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    // ── Query ─────────────────────────────────────────────────────────────────

    let params = QueryParams {
        query: "SELECT toDateTime(block_time) AS time, action_type, user \
                FROM hyperliquid_system_actions \
                ORDER BY block_time DESC LIMIT 3"
            .to_string(),
        cluster_id: CLUSTER_ID.to_string(),
    };

    match qn.sql.query(&params).await {
        Ok(resp) => {
            println!(
                "query: {} rows ({} before limit), {} credits, {:.4}s",
                resp.rows, resp.rows_before_limit_at_least, resp.credits, resp.statistics.elapsed
            );
            println!(
                "columns: {:?}",
                resp.meta.iter().map(|c| &c.name).collect::<Vec<_>>()
            );
            // Read a value out of a dynamic row to confirm the conversion works.
            if let Some(row) = resp.data.first() {
                println!("first row action_type: {}", row["action_type"]);
            }
        }
        Err(e) => eprintln!("query error: {e}"),
    }

    // ── Schema ──────────────────────────────────────────────────────────────

    match qn.sql.get_schema(CLUSTER_ID).await {
        Ok(schema) => {
            println!("schema: {} ({} tables)", schema.chain, schema.tables.len());
            if let Some(table) = schema.tables.first() {
                println!(
                    "first table: {} ({} columns, {} rows)",
                    table.name,
                    table.columns.len(),
                    table.total_rows
                );
            }
        }
        Err(e) => eprintln!("get_schema error: {e}"),
    }

    // ── Error handling ────────────────────────────────────────────────────────

    // An empty query is rejected with a 403 and a JSON body carrying the error
    // message.
    let bad = QueryParams {
        query: String::new(),
        cluster_id: CLUSTER_ID.to_string(),
    };
    match qn.sql.query(&bad).await {
        Err(SdkError::Api { status, body }) => {
            println!("api error {status}: {}", &body[..body.len().min(120)]);
        }
        other => eprintln!("expected Api error, got {other:?}"),
    }
}
