require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

# ── Sets ──────────────────────────────────────────────────────────────────

qn.kvstore.create_set(key: "e2e-set-key", value: "e2e-value")
puts "created set: e2e-set-key"

set = qn.kvstore.get_set(key: "e2e-set-key")
puts "get set: #{set[:value]}"

# get_sets is arity-1 native: exercise bare, kwargs, and positional hash.
sets = qn.kvstore.get_sets({})
puts "all sets: #{sets[:data].map { |e| e[:key] }.inspect}"
raise "get_sets bare broke" unless qn.kvstore.get_sets.keys.sort == sets.keys.sort
raise "get_sets kwargs splat broke" unless qn.kvstore.get_sets(**{}).keys.sort == sets.keys.sort

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
puts "get list items: #{list.dig(:data, :items).inspect}"

# get_lists is arity-1 native: exercise bare, kwargs, and positional hash.
lists = qn.kvstore.get_lists({})
puts "all list keys: #{lists.dig(:data, :keys).inspect}"
raise "get_lists bare broke" unless qn.kvstore.get_lists.keys.sort == lists.keys.sort
raise "get_lists kwargs splat broke" unless qn.kvstore.get_lists(**{}).keys.sort == lists.keys.sort

qn.kvstore.add_list_item(key: "e2e-list-key", item: "0x123")
puts "added list item: 0x123"

contains = qn.kvstore.list_contains_item(key: "e2e-list-key", item: "0x123")
puts "list contains 0x123: #{contains[:exists]}"

qn.kvstore.update_list(key: "e2e-list-key", add_items: ["0x456"], remove_items: ["0xabc"])
puts "updated list: added 0x456, removed 0xabc"

qn.kvstore.delete_list_item(key: "e2e-list-key", item: "0x123")
puts "deleted list item: 0x123"

qn.kvstore.delete_list(key: "e2e-list-key")
puts "deleted list: e2e-list-key"
