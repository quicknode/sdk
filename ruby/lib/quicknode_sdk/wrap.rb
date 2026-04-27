require "hashie"

module QuicknodeSdk
  def self.wrap(v)
    case v
    when Hash  then Hashie::Mash.new(v)
    when Array then v.map { |x| wrap(x) }
    else v
    end
  end
end
