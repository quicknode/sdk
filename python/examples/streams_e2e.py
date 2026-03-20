import asyncio
import time
from sdk import (
    DestinationAttributes,
    QuickNodeSdk,
    WebhookAttributes,
)


async def main():
    qn = QuickNodeSdk.from_env()

    before = await qn.streams.list_streams()
    print(f"streams before: {before.page_info.total}")

    count = await qn.streams.get_enabled_count()
    print(f"enabled count: {count.total}")

    filter_result = await qn.streams.test_filter(
        network="ethereum-mainnet",
        dataset="block",
        block="17811625",
        filter_function="ZnVuY3Rpb24gbWFpbihkYXRhKSB7IHJldHVybiBkYXRhOyB9",
    )
    print(f"filter logs: {filter_result.logs}")
    time.sleep(1)

    stream = await qn.streams.create_stream(
        name="E2E Test Stream",
        network="ethereum-mainnet",
        dataset="block",
        region="usa_east",
        start_range=24691804,
        end_range=24691904,
        destination_attributes=DestinationAttributes.webhook(
            WebhookAttributes(
                url="https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
                max_retry=3,
                retry_interval_sec=1,
                post_timeout_sec=10,
                compression="none",
            )
        ),
        plan="growth_plan",
        threshold_fetch_buffer=1000,
        dataset_batch_size=1,
        include_stream_metadata="body",
        fix_block_reorgs=0,
        keep_distance_from_tip=0,
        elastic_batch_enabled=True,
        status="active",
    )
    stream_id = stream.id
    print(f"created: {stream_id} | {stream.status}")

    fetched = await qn.streams.get_stream(stream_id)
    print(f"fetched: {fetched.id} | {fetched.name}")

    updated = await qn.streams.update_stream(stream_id, name="E2E Test Stream Updated")
    print(f"updated name: {updated.name}")
    time.sleep(1)

    await qn.streams.pause_stream(stream_id)
    print("paused")

    await qn.streams.activate_stream(stream_id)
    print("activated")

    await qn.streams.delete_stream(stream_id)
    print(f"deleted: {stream_id}")
    time.sleep(1)

    after = await qn.streams.list_streams()
    print(f"streams after: {after.page_info.total}")


asyncio.run(main())
