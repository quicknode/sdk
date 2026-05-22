Gem::Specification.new do |s|
  s.name        = "quicknode_sdk"
  s.version = "0.1.0-alpha.24"
  s.summary     = "Quicknode SDK for Ruby"
  s.authors     = ["Quicknode"]
  s.license     = "MIT"
  s.files       = Dir["lib/**/*.rb"] + Dir["sig/**/*.rbs"] + ["README.md"]
  s.required_ruby_version = ">= 3.0"
  s.add_runtime_dependency "hashie", "~> 5.0"
end
