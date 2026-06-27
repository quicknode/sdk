//go:build darwin && arm64

// Package-local cgo link directives for macOS arm64. Statically links the
// UniFFI facade staticlib (libquicknode_sdk.a), built fresh by `just go-build`
// and gitignored per CLAUDE.md (native libs are never committed). One such file
// exists per supported GOOS/GOARCH, selected by the build tag above. macOS
// static linking pulls in the system frameworks the Rust std/reqwest stack
// needs (Security, CoreFoundation, SystemConfiguration) plus libresolv.
package quicknode_sdk

// #cgo LDFLAGS: ${SRCDIR}/lib/darwin_arm64/libquicknode_sdk.a -lresolv -framework Security -framework CoreFoundation -framework SystemConfiguration
import "C"
