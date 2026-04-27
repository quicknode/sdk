require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

# ── Sets ──────────────────────────────────────────────────────────────────

qn.kvstore.create_set(key: "e2e-set-key", value: "e2e-value")
puts "created set: e2e-set-key"

set = qn.kvstore.get_set(key: "e2e-set-key")
puts "get set: #{set.value}"

sets = qn.kvstore.get_sets({})
puts "all sets: #{sets.data.map(&:key).inspect}"

qn.kvstore.bulk_sets(delete_sets: ["e2e-set-key"])
puts "bulk sets: deleted e2e-set-key"

qn.kvstore.create_set(key: "e2e-bulk-key-1", value: "bulk-value-1")
qn.kvstore.create_set(key: "e2e-bulk-key-2", value: "bulk-value-2")
qn.kvstore.delete_set(key: "e2e-bulk-key-1")
qn.kvstore.delete_set(key: "e2e-bulk-key-2")
puts "deleted bulk sets"

# ── Lists ─────────────────────────────────────────────────────────────────

qn.kvstore.create_list(key: "e2e-list-key", items: ["0xabc", "0xdef"])
puts "created list: e2e-list-key"

list = qn.kvstore.get_list(key: "e2e-list-key")
puts "get list items: #{list.data.items.inspect}"

# Note: data.keys would call Hash#keys (Mash subclasses Hash); use string access for the API field.
lists = qn.kvstore.get_lists({})
puts "all list keys: #{lists.data["keys"].inspect}"

qn.kvstore.add_list_item(key: "e2e-list-key", item: "0x123")
puts "added list item: 0x123"

contains = qn.kvstore.list_contains_item(key: "e2e-list-key", item: "0x123")
puts "list contains 0x123: #{contains.exists}"

qn.kvstore.update_list(key: "e2e-list-key", add_items: ["0x456"], remove_items: ["0xabc"])
puts "updated list: added 0x456, removed 0xabc"

qn.kvstore.delete_list_item(key: "e2e-list-key", item: "0x123")
puts "deleted list item: 0x123"

qn.kvstore.delete_list(key: "e2e-list-key")
puts "deleted list: e2e-list-key"
