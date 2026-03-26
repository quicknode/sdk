require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env

# ── Sets ──────────────────────────────────────────────────────────────────

qn.kvstore.create_set("e2e-set-key", "e2e-value")
puts "created set: e2e-set-key"

set = JSON.parse(qn.kvstore.get_set("e2e-set-key"))
puts "get set: #{set["value"]}"

sets = JSON.parse(qn.kvstore.get_sets(nil, nil))
puts "all sets: #{sets["data"].map { |e| e["key"] }.inspect}"

qn.kvstore.bulk_sets(
  { "e2e-bulk-key-1" => "bulk-value-1", "e2e-bulk-key-2" => "bulk-value-2" },
  ["e2e-set-key"]
)
puts "bulk sets: added 2, deleted e2e-set-key"

qn.kvstore.delete_set("e2e-bulk-key-1")
qn.kvstore.delete_set("e2e-bulk-key-2")
puts "deleted bulk sets"

# ── Lists ─────────────────────────────────────────────────────────────────

qn.kvstore.create_list("e2e-list-key", ["0xabc", "0xdef"])
puts "created list: e2e-list-key"

list = JSON.parse(qn.kvstore.get_list("e2e-list-key", nil, nil))
puts "get list items: #{list["data"]["items"].inspect}"

lists = JSON.parse(qn.kvstore.get_lists(nil, nil))
puts "all list keys: #{lists["data"]["keys"].inspect}"

qn.kvstore.add_list_item("e2e-list-key", "0x123")
puts "added list item: 0x123"

contains = JSON.parse(qn.kvstore.list_contains_item("e2e-list-key", "0x123"))
puts "list contains 0x123: #{contains["exists"]}"

qn.kvstore.update_list("e2e-list-key", ["0x456"], ["0xabc"])
puts "updated list: added 0x456, removed 0xabc"

qn.kvstore.delete_list_item("e2e-list-key", "0x123")
puts "deleted list item: 0x123"

qn.kvstore.delete_list("e2e-list-key")
puts "deleted list: e2e-list-key"
