#!/usr/bin/env ruby
# Writes a temporary platform-specific gemspec (quicknode_sdk_platform.gemspec)
# that includes a prebuilt native library. Run from the ruby/ directory.
#
# Usage: ruby ../scripts/build-platform-gem.rb <platform> <lib_file>
# Example: ruby ../scripts/build-platform-gem.rb arm64-darwin lib/quicknode_sdk/quicknode_sdk.bundle

platform, lib_file = ARGV
abort "Usage: build-platform-gem.rb <platform> <lib_file>" unless platform && lib_file

spec = Gem::Specification.load('quicknode_sdk.gemspec')
spec.platform = Gem::Platform.new(platform)
spec.extensions = []
spec.files += [lib_file]
File.write('quicknode_sdk_platform.gemspec', spec.to_ruby)
