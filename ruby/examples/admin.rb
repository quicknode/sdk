require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env
response = JSON.parse(qn.admin.get_endpoints(20, nil, nil, nil))
response["data"].each do |ep|
  puts "#{ep["id"]} | #{ep["network"]}"
end
