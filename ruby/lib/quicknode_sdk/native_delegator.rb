module QuicknodeSdk
  # Shared base class for the user-facing client wrappers (Admin, Streams,
  # Webhooks, KvStore). Each Magnus-bound native client is held in @native and
  # all method calls are forwarded through method_missing.
  #
  # The native side exposes two kinds of methods: arity-0 (e.g. list_chains)
  # and arity-1 taking a single positional Hash of options (e.g.
  # get_endpoints). To support all three Ruby call styles documented in the
  # README and examples — bare (qn.admin.get_endpoints), kwargs
  # (qn.admin.get_endpoints(limit: 5)), and positional hash
  # (qn.streams.list_streams({})) — we coerce whatever the caller passed into
  # a single options hash, then dispatch on the native arity. Arity-0 methods
  # reject any argument (Magnus enforces this), so we must call them with no
  # args; arity-1 methods always need a Hash passed POSITIONALLY (Magnus's
  # RHash is a positional arg, and Ruby 3 treats `**{}` as zero arguments —
  # so we must not splat the options).
  class NativeDelegator
    def initialize(native)
      @native = native
    end

    def method_missing(name, *args, **kwargs)
      return super unless @native.respond_to?(name)
      opts = if !kwargs.empty?
               kwargs
             elsif args.length == 1 && args[0].is_a?(Hash)
               args[0]
             else
               {}
             end
      result = if @native.method(name).arity == 0
                 @native.public_send(name)
               else
                 @native.public_send(name, opts)
               end
      QuicknodeSdk.wrap(result)
    end

    def respond_to_missing?(name, include_private = false)
      @native.respond_to?(name, include_private) || super
    end
  end
end
