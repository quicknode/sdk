module QuicknodeSdk
  class SDK
    def self.from_env
      new(Native::SDK.from_env)
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
