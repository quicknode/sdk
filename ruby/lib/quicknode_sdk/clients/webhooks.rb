module QuicknodeSdk
  class Webhooks
    def initialize(native)
      @native = native
    end

    def method_missing(name, *args, **kwargs)
      return super unless @native.respond_to?(name)
      result = if kwargs.empty?
                 @native.public_send(name, *args)
               else
                 @native.public_send(name, **kwargs)
               end
      QuicknodeSdk.wrap(result)
    end

    def respond_to_missing?(name, include_private = false)
      @native.respond_to?(name, include_private) || super
    end
  end
end
