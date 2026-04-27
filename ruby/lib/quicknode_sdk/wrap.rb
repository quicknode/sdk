require "hashie"

module QuicknodeSdk
  class IndifferentHash < Hash
    include Hashie::Extensions::MergeInitializer
    include Hashie::Extensions::IndifferentAccess
  end

  def self.wrap(v)
    case v
    when Hash  then IndifferentHash.new(v).tap { |h| h.each { |k, val| h[k] = wrap(val) } }
    when Array then v.map { |x| wrap(x) }
    else v
    end
  end
end
