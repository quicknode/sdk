import { QuicknodeSdk, ApiError } from "../sdk";

const CLUSTER_ID = "hyperliquid-core-mainnet";

async function main() {
  const qn = QuicknodeSdk.fromEnv();

  // Query
  const resp = await qn.sql.query(
    "SELECT toDateTime(block_time) AS time, action_type, user " +
      "FROM hyperliquid_system_actions " +
      "ORDER BY block_time DESC LIMIT 3",
    CLUSTER_ID,
  );
  console.log(
    `query: ${resp.rows} rows (${resp.rowsBeforeLimitAtLeast} before limit), ` +
      `${resp.credits} credits, ${resp.statistics.elapsed.toFixed(4)}s`,
  );
  console.log(`columns: ${resp.meta.map((c) => c.name).join(", ")}`);
  if (resp.data.length > 0) {
    console.log(`response: ${JSON.stringify(resp.data, null, 2)}`);
    // data rows are plain objects keyed by column name
    console.log(`first row action_type: ${resp.data[0].action_type}`);
  }

  // Schema
  const schema = await qn.sql.getSchema(CLUSTER_ID);
  console.log(`schema: ${schema.chain} (${schema.tables.length} tables)`);
  if (schema.tables.length > 0) {
    const t = schema.tables[0];
    console.log(
      `first table: ${t.name} (${t.columns.length} columns, ${t.totalRows} rows)`,
    );
  }

  // Error handling: an empty query is rejected with a 403.
  try {
    await qn.sql.query("", CLUSTER_ID);
  } catch (e) {
    if (!(e instanceof ApiError)) throw e;
    console.log(`api error ${e.status}: ${e.body.substring(0, 120)}`);
  }
}

main();
