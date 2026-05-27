module QuicknodeSdk
  class SDK
    def self.from_env
      new(Native::SDK.from_env)
    end

    # Build an SDK from an explicit config hash. Mirrors the Python/Node
    # constructor surface — supports custom headers, timeouts, and base URLs
    # without relying on env vars.
    #
    #   QuicknodeSdk::SDK.from_config(
    #     api_key: "...",
    #     http: { headers: { "X-Correlation-Id" => "abc" } }
    #   )
    def self.from_config(opts)
      new(Native::SDK.from_config(opts))
    end

    def initialize(native)
      @native = native
    end

    def admin
      Admin.new(@native.admin)
    end

    def streams
      Streams.new(@native.streams)
    end

    def webhooks
      Webhooks.new(@native.webhooks)
    end

    def kvstore
      KvStore.new(@native.kvstore)
    end
  end
end
