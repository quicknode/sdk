import { QuicknodeSdk } from "../sdk";
import type {
  CreateSetParams,
  BulkSetsParams,
  CreateListParams,
  AddListItemParams,
  UpdateListParams,
} from "../sdk";

async function main() {
  const qn = QuicknodeSdk.fromEnv();

  // ── Sets ──────────────────────────────────────────────────────────────────

  await qn.kvstore.createSet({ key: "e2e-set-key1", value: "e2e-value" });

  const createSetParams: CreateSetParams = {
    key: "e2e-set-key",
    value: "e2e-value",
  };
  await qn.kvstore.createSet(createSetParams);
  console.log("created set: e2e-set-key");

  const set = await qn.kvstore.getSet("e2e-set-key");
  console.log(`get set: ${set.value}`);

  const sets = await qn.kvstore.getSets();
  console.log(`all sets: ${JSON.stringify(sets.data.map((e) => e.key))}`);

  const bulkParams: BulkSetsParams = {
    addSets: {
      "e2e-bulk-key-1": "bulk-value-1",
      "e2e-bulk-key-2": "bulk-value-2",
    },
    deleteSets: ["e2e-set-key"],
  };
  await qn.kvstore.bulkSets(bulkParams);
  console.log("bulk sets: added 2, deleted e2e-set-key");

  await qn.kvstore.deleteSet("e2e-bulk-key-1");
  await qn.kvstore.deleteSet("e2e-bulk-key-2");
  console.log("deleted bulk sets");

  // ── Lists ─────────────────────────────────────────────────────────────────

  const createListParams: CreateListParams = {
    key: "e2e-list-key",
    items: ["0xabc", "0xdef"],
  };
  await qn.kvstore.createList(createListParams);
  console.log("created list: e2e-list-key");

  const list = await qn.kvstore.getList("e2e-list-key");
  console.log(`get list items: ${JSON.stringify(list.data.items)}`);

  const lists = await qn.kvstore.getLists();
  console.log(`all list keys: ${JSON.stringify(lists.data.keys)}`);

  const addItemParams: AddListItemParams = { item: "0x123" };
  await qn.kvstore.addListItem("e2e-list-key", addItemParams);
  console.log("added list item: 0x123");

  const contains = await qn.kvstore.listContainsItem("e2e-list-key", "0x123");
  console.log(`list contains 0x123: ${contains.exists}`);

  const updateParams: UpdateListParams = {
    addItems: ["0x456"],
    removeItems: ["0xabc"],
  };
  await qn.kvstore.updateList("e2e-list-key", updateParams);
  console.log("updated list: added 0x456, removed 0xabc");

  await qn.kvstore.deleteListItem("e2e-list-key", "0x123");
  console.log("deleted list item: 0x123");

  await qn.kvstore.deleteList("e2e-list-key");
  console.log("deleted list: e2e-list-key");
}

main();
