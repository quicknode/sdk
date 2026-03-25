import asyncio
from sdk import QuickNodeSdk


async def main():
    qn = QuickNodeSdk.from_env()

    # ── Sets ─────────────────────────────────────────────────────────────────

    await qn.kvstore.create_set(key="e2e-set-key", value="e2e-value")
    print("created set: e2e-set-key")

    set_resp = await qn.kvstore.get_set("e2e-set-key")
    print(f"get set: {set_resp.value}")

    sets = await qn.kvstore.get_sets()
    print(f"all sets: {[e.key for e in sets.data]}")

    await qn.kvstore.bulk_sets(
        add_sets={"e2e-bulk-key-1": "bulk-value-1", "e2e-bulk-key-2": "bulk-value-2"},
        delete_sets=["e2e-set-key"],
    )
    print("bulk sets: added 2, deleted e2e-set-key")

    await qn.kvstore.delete_set("e2e-bulk-key-1")
    await qn.kvstore.delete_set("e2e-bulk-key-2")
    print("deleted bulk sets")

    # ── Lists ─────────────────────────────────────────────────────────────────

    await qn.kvstore.create_list(key="e2e-list-key", items=["0xabc", "0xdef"])
    print("created list: e2e-list-key")

    list_resp = await qn.kvstore.get_list("e2e-list-key")
    print(f"get list items: {list_resp.data.items}")

    lists = await qn.kvstore.get_lists()
    print(f"all list keys: {lists.data.keys}")

    await qn.kvstore.add_list_item("e2e-list-key", "0x123")
    print("added list item: 0x123")

    contains = await qn.kvstore.list_contains_item("e2e-list-key", "0x123")
    print(f"list contains 0x123: {contains.exists}")

    await qn.kvstore.update_list(
        "e2e-list-key",
        add_items=["0x456"],
        remove_items=["0xabc"],
    )
    print("updated list: added 0x456, removed 0xabc")

    await qn.kvstore.delete_list_item("e2e-list-key", "0x123")
    print("deleted list item: 0x123")

    await qn.kvstore.delete_list("e2e-list-key")
    print("deleted list: e2e-list-key")


asyncio.run(main())
