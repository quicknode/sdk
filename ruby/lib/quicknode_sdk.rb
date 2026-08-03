begin
  require_relative "quicknode_sdk/quicknode_sdk"
rescue LoadError => e
  raise LoadError, <<~MSG
    Could not load the quicknode_sdk native extension for this platform (#{RUBY_PLATFORM}).
    Precompiled binaries are published for: x86_64-linux, aarch64-linux, arm64-darwin.
    Original error: #{e.message}
  MSG
end
require_relative "quicknode_sdk/wrap"
require_relative "quicknode_sdk/native_delegator"
require_relative "quicknode_sdk/clients/admin"
require_relative "quicknode_sdk/clients/streams"
require_relative "quicknode_sdk/clients/webhooks"
require_relative "quicknode_sdk/clients/kvstore"
require_relative "quicknode_sdk/clients/sql"
require_relative "quicknode_sdk/clients/rpc"
require_relative "quicknode_sdk/sdk"

module QuicknodeSdk
  # Generates a fresh payment keypair for :evm, :svm, or :tempo. Offline — no
  # network call, no funds. Returns {address:, chain:, key:}; `key` is the raw
  # private key in the format the payment config's key: accepts.
  #
  # The key is returned exactly once, at generation: nothing in the SDK stores
  # or re-derives it, so persist it before discarding the Hash.
  def self.generate_payment_wallet(**opts)
    wrap(Native.generate_payment_wallet(opts))
  end
end
