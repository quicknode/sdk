import asyncio
from quicknode_sdk import QuicknodeSdk, ApiError

CLUSTER_ID = "hyperliquid-core-mainnet"


async def main():
    qn = QuicknodeSdk.from_env()

    # Query
    resp = await qn.sql.query(
        query=(
            "SELECT toDateTime(block_time) AS time, action_type, user "
            "FROM hyperliquid_system_actions "
            "ORDER BY block_time DESC LIMIT 3"
        ),
        cluster_id=CLUSTER_ID,
    )
    print(
        f"query: {resp.rows} rows ({resp.rows_before_limit_at_least} before limit), "
        f"{resp.credits} credits, {resp.statistics.elapsed:.4f}s"
    )
    print(f"columns: {[c.name for c in resp.meta]}")
    if resp.data:
        # data rows are native dicts keyed by column name
        print(f"first row action_type: {resp.data[0]['action_type']}")

    # Schema
    schema = await qn.sql.get_schema(CLUSTER_ID)
    print(f"schema: {schema.chain} ({len(schema.tables)} tables)")
    if schema.tables:
        t = schema.tables[0]
        print(f"first table: {t.name} ({len(t.columns)} columns, {t.total_rows} rows)")

    # Error handling: an empty query is rejected with a 403.
    try:
        await qn.sql.query(query="", cluster_id=CLUSTER_ID)
    except ApiError as e:
        print(f"api error {e.status}: {e.body[:120]}")


asyncio.run(main())
