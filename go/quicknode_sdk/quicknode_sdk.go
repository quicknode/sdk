package quicknode_sdk

// #include <quicknode_sdk.h>
import "C"

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"runtime"
	"sync/atomic"
	"unsafe"
)

// This is needed, because as of go 1.24
// type RustBuffer C.RustBuffer cannot have methods,
// RustBuffer is treated as non-local type
type GoRustBuffer struct {
	inner C.RustBuffer
}

type RustBufferI interface {
	AsReader() *bytes.Reader
	Free()
	ToGoBytes() []byte
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

// C.RustBuffer fields exposed as an interface so they can be accessed in different Go packages.
// See https://github.com/golang/go/issues/13467
type ExternalCRustBuffer interface {
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

func RustBufferFromC(b C.RustBuffer) ExternalCRustBuffer {
	return GoRustBuffer{
		inner: b,
	}
}

func CFromRustBuffer(b ExternalCRustBuffer) C.RustBuffer {
	return C.RustBuffer{
		capacity: C.uint64_t(b.Capacity()),
		len:      C.uint64_t(b.Len()),
		data:     (*C.uchar)(b.Data()),
	}
}

func RustBufferFromExternal(b ExternalCRustBuffer) GoRustBuffer {
	return GoRustBuffer{
		inner: C.RustBuffer{
			capacity: C.uint64_t(b.Capacity()),
			len:      C.uint64_t(b.Len()),
			data:     (*C.uchar)(b.Data()),
		},
	}
}

func (cb GoRustBuffer) Capacity() uint64 {
	return uint64(cb.inner.capacity)
}

func (cb GoRustBuffer) Len() uint64 {
	return uint64(cb.inner.len)
}

func (cb GoRustBuffer) Data() unsafe.Pointer {
	return unsafe.Pointer(cb.inner.data)
}

func (cb GoRustBuffer) AsReader() *bytes.Reader {
	b := unsafe.Slice((*byte)(cb.inner.data), C.uint64_t(cb.inner.len))
	return bytes.NewReader(b)
}

func (cb GoRustBuffer) Free() {
	rustCall(func(status *C.RustCallStatus) bool {
		C.ffi_quicknode_sdk_rustbuffer_free(cb.inner, status)
		return false
	})
}

func (cb GoRustBuffer) ToGoBytes() []byte {
	return C.GoBytes(unsafe.Pointer(cb.inner.data), C.int(cb.inner.len))
}

func stringToRustBuffer(str string) C.RustBuffer {
	return bytesToRustBuffer([]byte(str))
}

func bytesToRustBuffer(b []byte) C.RustBuffer {
	if len(b) == 0 {
		return C.RustBuffer{}
	}
	// We can pass the pointer along here, as it is pinned
	// for the duration of this call
	foreign := C.ForeignBytes{
		len:  C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}

	return rustCall(func(status *C.RustCallStatus) C.RustBuffer {
		return C.ffi_quicknode_sdk_rustbuffer_from_bytes(foreign, status)
	})
}

type BufLifter[GoType any] interface {
	Lift(value RustBufferI) GoType
}

type BufLowerer[GoType any] interface {
	Lower(value GoType) C.RustBuffer
}

type BufReader[GoType any] interface {
	Read(reader io.Reader) GoType
}

type BufWriter[GoType any] interface {
	Write(writer io.Writer, value GoType)
}

func LowerIntoRustBuffer[GoType any](bufWriter BufWriter[GoType], value GoType) C.RustBuffer {
	// This might be not the most efficient way but it does not require knowing allocation size
	// beforehand
	var buffer bytes.Buffer
	bufWriter.Write(&buffer, value)

	bytes, err := io.ReadAll(&buffer)
	if err != nil {
		panic(fmt.Errorf("reading written data: %w", err))
	}
	return bytesToRustBuffer(bytes)
}

func LiftFromRustBuffer[GoType any](bufReader BufReader[GoType], rbuf RustBufferI) GoType {
	defer rbuf.Free()
	reader := rbuf.AsReader()
	item := bufReader.Read(reader)
	if reader.Len() > 0 {
		// TODO: Remove this
		leftover, _ := io.ReadAll(reader)
		panic(fmt.Errorf("Junk remaining in buffer after lifting: %s", string(leftover)))
	}
	return item
}

func rustCallWithError[E any, U any](converter BufReader[E], callback func(*C.RustCallStatus) U) (U, E) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	err := checkCallStatus(converter, status)
	return returnValue, err
}

func checkCallStatus[E any](converter BufReader[E], status C.RustCallStatus) E {
	switch status.code {
	case 0:
		var zero E
		return zero
	case 1:
		return LiftFromRustBuffer(converter, GoRustBuffer{inner: status.errorBuf})
	case 2:
		// when the rust code sees a panic, it tries to construct a rustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{inner: status.errorBuf})))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		panic(fmt.Errorf("unknown status code: %d", status.code))
	}
}

func checkCallStatusUnknown(status C.RustCallStatus) error {
	switch status.code {
	case 0:
		return nil
	case 1:
		panic(fmt.Errorf("function not returning an error returned an error"))
	case 2:
		// when the rust code sees a panic, it tries to construct a C.RustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{
				inner: status.errorBuf,
			})))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return fmt.Errorf("unknown status code: %d", status.code)
	}
}

func rustCall[U any](callback func(*C.RustCallStatus) U) U {
	returnValue, err := rustCallWithError[error](nil, callback)
	if err != nil {
		panic(err)
	}
	return returnValue
}

type NativeError interface {
	AsError() error
}

func writeInt8(writer io.Writer, value int8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint8(writer io.Writer, value uint8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt16(writer io.Writer, value int16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint16(writer io.Writer, value uint16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt32(writer io.Writer, value int32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint32(writer io.Writer, value uint32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt64(writer io.Writer, value int64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint64(writer io.Writer, value uint64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat32(writer io.Writer, value float32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat64(writer io.Writer, value float64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func readInt8(reader io.Reader) int8 {
	var result int8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint8(reader io.Reader) uint8 {
	var result uint8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt16(reader io.Reader) int16 {
	var result int16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint16(reader io.Reader) uint16 {
	var result uint16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt32(reader io.Reader) int32 {
	var result int32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint32(reader io.Reader) uint32 {
	var result uint32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt64(reader io.Reader) int64 {
	var result int64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint64(reader io.Reader) uint64 {
	var result uint64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat32(reader io.Reader) float32 {
	var result float32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat64(reader io.Reader) float64 {
	var result float64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func init() {

	uniffiCheckChecksums()
}

func uniffiCheckChecksums() {
	// Get the bindings contract version from our ComponentInterface
	bindingsContractVersion := 30
	// Get the scaffolding contract version by calling the into the dylib
	scaffoldingContractVersion := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint32_t {
		return C.ffi_quicknode_sdk_uniffi_contract_version()
	})
	if bindingsContractVersion != int(scaffoldingContractVersion) {
		// If this happens try cleaning and rebuilding your project
		panic("quicknode_sdk: UniFFI contract version mismatch")
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_archive_endpoint()
		})
		if checksum != 59372 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_archive_endpoint: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_bulk_add_tag()
		})
		if checksum != 14428 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_bulk_add_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_bulk_remove_tag()
		})
		if checksum != 27065 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_bulk_remove_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_bulk_update_endpoint_status()
		})
		if checksum != 30094 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_bulk_update_endpoint_status: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_domain_mask()
		})
		if checksum != 8117 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_domain_mask: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_endpoint()
		})
		if checksum != 34815 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_endpoint: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_ip()
		})
		if checksum != 31844 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_ip: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_jwt()
		})
		if checksum != 19331 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_jwt: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_method_rate_limit()
		})
		if checksum != 10455 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_method_rate_limit: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_or_update_ip_custom_header()
		})
		if checksum != 12103 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_or_update_ip_custom_header: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_referrer()
		})
		if checksum != 50682 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_referrer: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_request_filter()
		})
		if checksum != 41950 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_request_filter: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_tag()
		})
		if checksum != 55012 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_team()
		})
		if checksum != 9412 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_team: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_create_token()
		})
		if checksum != 58891 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_create_token: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_account_tag()
		})
		if checksum != 2282 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_account_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_domain_mask()
		})
		if checksum != 44390 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_domain_mask: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_ip()
		})
		if checksum != 32255 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_ip: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_ip_custom_header()
		})
		if checksum != 55795 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_ip_custom_header: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_jwt()
		})
		if checksum != 4297 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_jwt: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_method_rate_limit()
		})
		if checksum != 13399 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_method_rate_limit: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_rate_limit_override()
		})
		if checksum != 11375 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_rate_limit_override: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_referrer()
		})
		if checksum != 61924 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_referrer: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_request_filter()
		})
		if checksum != 5706 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_request_filter: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_tag()
		})
		if checksum != 64944 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_team()
		})
		if checksum != 64466 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_team: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_delete_token()
		})
		if checksum != 53886 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_delete_token: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_disable_multichain()
		})
		if checksum != 34232 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_disable_multichain: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_enable_multichain()
		})
		if checksum != 57344 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_enable_multichain: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_account_metrics()
		})
		if checksum != 35023 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_account_metrics: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_logs()
		})
		if checksum != 2511 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_logs: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_metrics()
		})
		if checksum != 16409 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_metrics: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_security()
		})
		if checksum != 31637 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_security: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_urls()
		})
		if checksum != 62411 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoint_urls: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoints()
		})
		if checksum != 59058 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_endpoints: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_log_details()
		})
		if checksum != 63264 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_log_details: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_method_rate_limits()
		})
		if checksum != 63206 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_method_rate_limits: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_rate_limits()
		})
		if checksum != 64031 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_rate_limits: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_security_options()
		})
		if checksum != 57662 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_security_options: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_team()
		})
		if checksum != 50754 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_team: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_usage()
		})
		if checksum != 25735 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_usage: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_chain()
		})
		if checksum != 36802 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_chain: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_endpoint()
		})
		if checksum != 9301 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_endpoint: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_method()
		})
		if checksum != 63927 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_method: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_tag()
		})
		if checksum != 19443 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_get_usage_by_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_invite_team_member()
		})
		if checksum != 63686 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_invite_team_member: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_chains()
		})
		if checksum != 14362 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_chains: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_invoices()
		})
		if checksum != 42115 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_invoices: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_payments()
		})
		if checksum != 53377 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_payments: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_tags()
		})
		if checksum != 4935 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_tags: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_team_endpoints()
		})
		if checksum != 54511 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_team_endpoints: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_list_teams()
		})
		if checksum != 50708 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_list_teams: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_remove_team_member()
		})
		if checksum != 44639 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_remove_team_member: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_rename_tag()
		})
		if checksum != 53012 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_rename_tag: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_resend_team_invite()
		})
		if checksum != 34863 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_resend_team_invite: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_show_endpoint()
		})
		if checksum != 6454 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_show_endpoint: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_endpoint()
		})
		if checksum != 3243 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_endpoint: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_endpoint_status()
		})
		if checksum != 21671 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_endpoint_status: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_method_rate_limit()
		})
		if checksum != 22255 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_method_rate_limit: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_rate_limits()
		})
		if checksum != 58114 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_rate_limits: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_request_filter()
		})
		if checksum != 44932 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_request_filter: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_security_options()
		})
		if checksum != 8494 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_security_options: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_adminclient_update_team_endpoints()
		})
		if checksum != 60896 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_adminclient_update_team_endpoints: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_add_list_item()
		})
		if checksum != 31883 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_add_list_item: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_bulk_sets()
		})
		if checksum != 10422 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_bulk_sets: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_create_list()
		})
		if checksum != 55476 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_create_list: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_create_set()
		})
		if checksum != 29640 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_create_set: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_list()
		})
		if checksum != 1593 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_list: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_list_item()
		})
		if checksum != 3522 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_list_item: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_set()
		})
		if checksum != 41111 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_delete_set: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_list()
		})
		if checksum != 64241 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_list: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_lists()
		})
		if checksum != 51431 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_lists: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_set()
		})
		if checksum != 46568 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_set: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_sets()
		})
		if checksum != 9604 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_get_sets: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_list_contains_item()
		})
		if checksum != 15544 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_list_contains_item: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_kvstoreclient_update_list()
		})
		if checksum != 19036 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_kvstoreclient_update_list: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_admin()
		})
		if checksum != 42388 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_admin: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_kvstore()
		})
		if checksum != 8810 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_kvstore: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_sql()
		})
		if checksum != 63289 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_sql: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_streams()
		})
		if checksum != 2331 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_streams: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_webhooks()
		})
		if checksum != 62815 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_webhooks: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_sqlclient_get_schema()
		})
		if checksum != 42430 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_sqlclient_get_schema: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_sqlclient_query()
		})
		if checksum != 58877 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_sqlclient_query: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_activate_stream()
		})
		if checksum != 28789 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_activate_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_create_stream()
		})
		if checksum != 50224 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_create_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_delete_all_streams()
		})
		if checksum != 39751 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_delete_all_streams: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_delete_stream()
		})
		if checksum != 47107 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_delete_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_get_enabled_count()
		})
		if checksum != 48336 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_get_enabled_count: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_get_stream()
		})
		if checksum != 45346 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_get_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_list_streams()
		})
		if checksum != 16607 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_list_streams: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_pause_stream()
		})
		if checksum != 53405 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_pause_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_test_filter()
		})
		if checksum != 54024 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_test_filter: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_streamsclient_update_stream()
		})
		if checksum != 693 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_streamsclient_update_stream: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_activate_webhook()
		})
		if checksum != 47971 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_activate_webhook: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_create_webhook_from_template()
		})
		if checksum != 65003 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_create_webhook_from_template: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_delete_all_webhooks()
		})
		if checksum != 19391 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_delete_all_webhooks: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_delete_webhook()
		})
		if checksum != 16559 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_delete_webhook: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_get_enabled_count()
		})
		if checksum != 56083 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_get_enabled_count: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_get_webhook()
		})
		if checksum != 825 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_get_webhook: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_list_webhooks()
		})
		if checksum != 48011 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_list_webhooks: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_pause_webhook()
		})
		if checksum != 63687 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_pause_webhook: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_update_webhook()
		})
		if checksum != 55595 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_update_webhook: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_method_webhooksclient_update_webhook_template()
		})
		if checksum != 48110 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_webhooksclient_update_webhook_template: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new()
		})
		if checksum != 59143 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new_with_base_urls()
		})
		if checksum != 31173 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new_with_base_urls: UniFFI API checksum mismatch")
		}
	}
}

type FfiConverterUint16 struct{}

var FfiConverterUint16INSTANCE = FfiConverterUint16{}

func (FfiConverterUint16) Lower(value uint16) C.uint16_t {
	return C.uint16_t(value)
}

func (FfiConverterUint16) Write(writer io.Writer, value uint16) {
	writeUint16(writer, value)
}

func (FfiConverterUint16) Lift(value C.uint16_t) uint16 {
	return uint16(value)
}

func (FfiConverterUint16) Read(reader io.Reader) uint16 {
	return readUint16(reader)
}

type FfiDestroyerUint16 struct{}

func (FfiDestroyerUint16) Destroy(_ uint16) {}

type FfiConverterInt32 struct{}

var FfiConverterInt32INSTANCE = FfiConverterInt32{}

func (FfiConverterInt32) Lower(value int32) C.int32_t {
	return C.int32_t(value)
}

func (FfiConverterInt32) Write(writer io.Writer, value int32) {
	writeInt32(writer, value)
}

func (FfiConverterInt32) Lift(value C.int32_t) int32 {
	return int32(value)
}

func (FfiConverterInt32) Read(reader io.Reader) int32 {
	return readInt32(reader)
}

type FfiDestroyerInt32 struct{}

func (FfiDestroyerInt32) Destroy(_ int32) {}

type FfiConverterInt64 struct{}

var FfiConverterInt64INSTANCE = FfiConverterInt64{}

func (FfiConverterInt64) Lower(value int64) C.int64_t {
	return C.int64_t(value)
}

func (FfiConverterInt64) Write(writer io.Writer, value int64) {
	writeInt64(writer, value)
}

func (FfiConverterInt64) Lift(value C.int64_t) int64 {
	return int64(value)
}

func (FfiConverterInt64) Read(reader io.Reader) int64 {
	return readInt64(reader)
}

type FfiDestroyerInt64 struct{}

func (FfiDestroyerInt64) Destroy(_ int64) {}

type FfiConverterFloat64 struct{}

var FfiConverterFloat64INSTANCE = FfiConverterFloat64{}

func (FfiConverterFloat64) Lower(value float64) C.double {
	return C.double(value)
}

func (FfiConverterFloat64) Write(writer io.Writer, value float64) {
	writeFloat64(writer, value)
}

func (FfiConverterFloat64) Lift(value C.double) float64 {
	return float64(value)
}

func (FfiConverterFloat64) Read(reader io.Reader) float64 {
	return readFloat64(reader)
}

type FfiDestroyerFloat64 struct{}

func (FfiDestroyerFloat64) Destroy(_ float64) {}

type FfiConverterBool struct{}

var FfiConverterBoolINSTANCE = FfiConverterBool{}

func (FfiConverterBool) Lower(value bool) C.int8_t {
	if value {
		return C.int8_t(1)
	}
	return C.int8_t(0)
}

func (FfiConverterBool) Write(writer io.Writer, value bool) {
	if value {
		writeInt8(writer, 1)
	} else {
		writeInt8(writer, 0)
	}
}

func (FfiConverterBool) Lift(value C.int8_t) bool {
	return value != 0
}

func (FfiConverterBool) Read(reader io.Reader) bool {
	return readInt8(reader) != 0
}

type FfiDestroyerBool struct{}

func (FfiDestroyerBool) Destroy(_ bool) {}

type FfiConverterString struct{}

var FfiConverterStringINSTANCE = FfiConverterString{}

func (FfiConverterString) Lift(rb RustBufferI) string {
	defer rb.Free()
	reader := rb.AsReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return string(b)
}

func (FfiConverterString) Read(reader io.Reader) string {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil && err != io.EOF {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading string, expected %d, read %d", length, read_length))
	}
	return string(buffer)
}

func (FfiConverterString) Lower(value string) C.RustBuffer {
	return stringToRustBuffer(value)
}

func (c FfiConverterString) LowerExternal(value string) ExternalCRustBuffer {
	return RustBufferFromC(stringToRustBuffer(value))
}

func (FfiConverterString) Write(writer io.Writer, value string) {
	if len(value) > math.MaxInt32 {
		panic("String is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := io.WriteString(writer, value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing string, expected %d, written %d", len(value), write_length))
	}
}

type FfiDestroyerString struct{}

func (FfiDestroyerString) Destroy(_ string) {}

// Below is an implementation of synchronization requirements outlined in the link.
// https://github.com/mozilla/uniffi-rs/blob/0dc031132d9493ca812c3af6e7dd60ad2ea95bf0/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt#L31

type FfiObject struct {
	handle        C.uint64_t
	callCounter   atomic.Int64
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t
	freeFunction  func(C.uint64_t, *C.RustCallStatus)
	destroyed     atomic.Bool
}

func newFfiObject(
	handle C.uint64_t,
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t,
	freeFunction func(C.uint64_t, *C.RustCallStatus),
) FfiObject {
	return FfiObject{
		handle:        handle,
		cloneFunction: cloneFunction,
		freeFunction:  freeFunction,
	}
}

func (ffiObject *FfiObject) incrementPointer(debugName string) C.uint64_t {
	for {
		counter := ffiObject.callCounter.Load()
		if counter <= -1 {
			panic(fmt.Errorf("%v object has already been destroyed", debugName))
		}
		if counter == math.MaxInt64 {
			panic(fmt.Errorf("%v object call counter would overflow", debugName))
		}
		if ffiObject.callCounter.CompareAndSwap(counter, counter+1) {
			break
		}
	}

	return rustCall(func(status *C.RustCallStatus) C.uint64_t {
		return ffiObject.cloneFunction(ffiObject.handle, status)
	})
}

func (ffiObject *FfiObject) decrementPointer() {
	if ffiObject.callCounter.Add(-1) == -1 {
		ffiObject.freeRustArcPtr()
	}
}

func (ffiObject *FfiObject) destroy() {
	if ffiObject.destroyed.CompareAndSwap(false, true) {
		if ffiObject.callCounter.Add(-1) == -1 {
			ffiObject.freeRustArcPtr()
		}
	}
}

func (ffiObject *FfiObject) freeRustArcPtr() {
	if ffiObject.handle == 0 {
		return
	}
	rustCall(func(status *C.RustCallStatus) int32 {
		ffiObject.freeFunction(ffiObject.handle, status)
		return 0
	})
}

// Admin API sub-client.
type AdminClientInterface interface {
	// Archive an endpoint.
	ArchiveEndpoint(id string) error
	// Apply a single tag label to multiple endpoints in one call.
	BulkAddTag(params BulkAddTagRequest) (BulkAddTagResponse, error)
	// Remove a tag from multiple endpoints in one call.
	BulkRemoveTag(params BulkRemoveTagRequest) (BulkRemoveTagResponse, error)
	// Pause or unpause multiple endpoints in a single call.
	BulkUpdateEndpointStatus(params BulkUpdateEndpointStatusRequest) (BulkUpdateEndpointStatusResponse, error)
	// Add a domain mask to an endpoint.
	CreateDomainMask(id string, params CreateDomainMaskRequest) error
	// Create a new endpoint for a given blockchain and network.
	CreateEndpoint(params CreateEndpointRequest) (CreateEndpointResponse, error)
	// Add an IP address to an endpoint's security whitelist.
	CreateIp(id string, params CreateIpRequest) error
	// Create a new JWT for endpoint authentication.
	CreateJwt(id string, params CreateJwtRequest) error
	// Create a per-method rate limit on an endpoint.
	CreateMethodRateLimit(id string, params CreateMethodRateLimitRequest) (CreateMethodRateLimitResponse, error)
	// Set the custom HTTP header used to identify the client IP for an
	// endpoint.
	CreateOrUpdateIpCustomHeader(id string, params CreateOrUpdateIpCustomHeaderRequest) (CreateOrUpdateIpCustomHeaderResponse, error)
	// Add a referrer to an endpoint's security settings.
	CreateReferrer(id string, params CreateReferrerRequest) error
	// Create a request filter (method whitelist) on an endpoint.
	CreateRequestFilter(id string, params CreateRequestFilterRequest) (CreateRequestFilterResponse, error)
	// Create a new tag on a specific endpoint from a label.
	CreateTag(id string, params CreateTagRequest) error
	// Create a new team.
	CreateTeam(params CreateTeamRequest) (CreateTeamResponse, error)
	// Generate a new authentication token for an endpoint.
	CreateToken(id string) error
	// Delete an account-level tag. It must first be removed from all endpoints.
	DeleteAccountTag(id int32) (DeleteAccountTagResponse, error)
	// Remove a domain mask from an endpoint by domain mask id.
	DeleteDomainMask(id string, domainMaskId string) (DeleteBoolResponse, error)
	// Remove an IP address from an endpoint's security whitelist by ip id.
	DeleteIp(id string, ipId string) (DeleteBoolResponse, error)
	// Remove the custom IP header configuration from an endpoint.
	DeleteIpCustomHeader(id string) (DeleteBoolResponse, error)
	// Remove a JWT from an endpoint's security configuration by jwt id.
	DeleteJwt(id string, jwtId string) error
	// Remove a method rate limit from an endpoint by method rate limit id.
	DeleteMethodRateLimit(id string, methodRateLimitId string) error
	// Delete a user-set rate-limit override by its UUID.
	DeleteRateLimitOverride(id string, overrideId string) error
	// Remove a referrer from an endpoint's security settings by referrer id.
	DeleteReferrer(id string, referrerId string) (DeleteBoolResponse, error)
	// Remove a request filter from an endpoint by request filter id.
	DeleteRequestFilter(id string, requestFilterId string) error
	// Remove a tag from a specific endpoint by tag id.
	DeleteTag(id string, tagId string) error
	// Delete a team by id. The team must have no members.
	DeleteTeam(id int64) (DeleteTeamResponse, error)
	// Revoke a token on an endpoint by token id.
	DeleteToken(id string, tokenId string) (DeleteBoolResponse, error)
	// Disable multichain functionality on an endpoint.
	DisableMultichain(id string) error
	// Enable multichain functionality on an endpoint.
	EnableMultichain(id string) error
	// Fetch aggregated metrics across all endpoints on the account.
	GetAccountMetrics(params GetAccountMetricsRequest) (GetAccountMetricsResponse, error)
	// Fetch activity logs for a specific endpoint.
	GetEndpointLogs(id string, params GetEndpointLogsRequest) (GetEndpointLogsResponse, error)
	// Fetch time-series metrics for a specific endpoint.
	GetEndpointMetrics(id string, params GetEndpointMetricsRequest) (GetEndpointMetricsResponse, error)
	// Fetch the full security configuration for an endpoint in a single call.
	GetEndpointSecurity(id string) (GetEndpointSecurityResponse, error)
	// Fetch the HTTP and WebSocket URLs for the endpoint.
	GetEndpointUrls(id string) (GetEndpointUrlsResponse, error)
	// List endpoints on the account. Supports searching, filtering, sorting,
	// and pagination.
	GetEndpoints(params GetEndpointsRequest) (GetEndpointsResponse, error)
	// Fetch the raw request/response payloads for a specific log entry.
	GetLogDetails(id string, requestId string) (GetLogDetailsResponse, error)
	// Fetch the method rate limits configured on an endpoint.
	GetMethodRateLimits(id string) (GetMethodRateLimitsResponse, error)
	// Fetch the endpoint-level rate limits currently enforced.
	GetRateLimits(id string) (GetRateLimitsResponse, error)
	// Fetch the security options (feature toggles) for an endpoint.
	GetSecurityOptions(id string) (GetSecurityOptionsResponse, error)
	// Fetch a specific team by id.
	GetTeam(id int64) (GetTeamResponse, error)
	// Fetch account RPC usage totals for an optional time range.
	GetUsage(params GetUsageRequest) (GetUsageResponse, error)
	// Fetch RPC usage grouped by chain over an optional time range.
	GetUsageByChain(params GetUsageRequest) (GetUsageByChainResponse, error)
	// Fetch RPC usage broken down per endpoint over an optional time range.
	GetUsageByEndpoint(params GetUsageRequest) (GetUsageByEndpointResponse, error)
	// Fetch RPC usage grouped by method over an optional time range.
	GetUsageByMethod(params GetUsageRequest) (GetUsageByMethodResponse, error)
	// Fetch RPC usage grouped by endpoint tag over an optional time range.
	GetUsageByTag(params GetUsageRequest) (GetUsageByTagResponse, error)
	// Invite a user to a team by email.
	InviteTeamMember(id int64, params InviteTeamMemberRequest) (InviteTeamMemberResponse, error)
	// List all chains supported by Quicknode along with their networks.
	ListChains() (ListChainsResponse, error)
	// List the account's invoices.
	ListInvoices() (ListInvoicesResponse, error)
	// List all payments on the account.
	ListPayments() (ListPaymentsResponse, error)
	// List all account-level tags, including tags with zero endpoints.
	ListTags() (ListTagsResponse, error)
	// List the endpoints accessible to a given team.
	ListTeamEndpoints(id int64) (ListTeamEndpointsResponse, error)
	// List all teams on the account.
	ListTeams() (ListTeamsResponse, error)
	// Remove a user from a team by team id and user id.
	RemoveTeamMember(id int64, userId int64, params RemoveTeamMemberRequest) (RemoveTeamMemberResponse, error)
	// Update the label of an account tag.
	RenameTag(id int32, params RenameTagRequest) (RenameTagResponse, error)
	// Resend the invitation email to a pending team member.
	ResendTeamInvite(id int64, userId int64) (ResendTeamInviteResponse, error)
	// Fetch details for a specific endpoint by ID.
	ShowEndpoint(id string) (ShowEndpointResponse, error)
	// Update editable fields on an endpoint (e.g. its label).
	UpdateEndpoint(id string, params UpdateEndpointRequest) error
	// Pause or unpause an endpoint by setting its status.
	UpdateEndpointStatus(id string, params UpdateEndpointStatusRequest) (UpdateEndpointStatusResponse, error)
	// Update an existing method rate limit on an endpoint.
	UpdateMethodRateLimit(id string, methodRateLimitId string, params UpdateMethodRateLimitRequest) (UpdateMethodRateLimitResponse, error)
	// Partially update the endpoint-level rate-limit overrides.
	UpdateRateLimits(id string, params UpdateRateLimitsRequest) error
	// Update an existing request filter on an endpoint.
	UpdateRequestFilter(id string, requestFilterId string, params UpdateRequestFilterRequest) error
	// Update which security features are enabled on an endpoint.
	UpdateSecurityOptions(id string, params UpdateSecurityOptionsRequest) (UpdateSecurityOptionsResponse, error)
	// Assign or unassign endpoints for a team.
	UpdateTeamEndpoints(id int64, params UpdateTeamEndpointsRequest) (UpdateTeamEndpointsResponse, error)
}

// Admin API sub-client.
type AdminClient struct {
	ffiObject FfiObject
}

// Archive an endpoint.
func (_self *AdminClient) ArchiveEndpoint(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_archive_endpoint(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Apply a single tag label to multiple endpoints in one call.
func (_self *AdminClient) BulkAddTag(params BulkAddTagRequest) (BulkAddTagResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_bulk_add_tag(
				_pointer, FfiConverterBulkAddTagRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue BulkAddTagResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBulkAddTagResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove a tag from multiple endpoints in one call.
func (_self *AdminClient) BulkRemoveTag(params BulkRemoveTagRequest) (BulkRemoveTagResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_bulk_remove_tag(
				_pointer, FfiConverterBulkRemoveTagRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue BulkRemoveTagResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBulkRemoveTagResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Pause or unpause multiple endpoints in a single call.
func (_self *AdminClient) BulkUpdateEndpointStatus(params BulkUpdateEndpointStatusRequest) (BulkUpdateEndpointStatusResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_bulk_update_endpoint_status(
				_pointer, FfiConverterBulkUpdateEndpointStatusRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue BulkUpdateEndpointStatusResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBulkUpdateEndpointStatusResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Add a domain mask to an endpoint.
func (_self *AdminClient) CreateDomainMask(id string, params CreateDomainMaskRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_domain_mask(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateDomainMaskRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new endpoint for a given blockchain and network.
func (_self *AdminClient) CreateEndpoint(params CreateEndpointRequest) (CreateEndpointResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_create_endpoint(
				_pointer, FfiConverterCreateEndpointRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue CreateEndpointResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCreateEndpointResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Add an IP address to an endpoint's security whitelist.
func (_self *AdminClient) CreateIp(id string, params CreateIpRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_ip(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateIpRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new JWT for endpoint authentication.
func (_self *AdminClient) CreateJwt(id string, params CreateJwtRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_jwt(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateJwtRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a per-method rate limit on an endpoint.
func (_self *AdminClient) CreateMethodRateLimit(id string, params CreateMethodRateLimitRequest) (CreateMethodRateLimitResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_create_method_rate_limit(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateMethodRateLimitRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue CreateMethodRateLimitResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCreateMethodRateLimitResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Set the custom HTTP header used to identify the client IP for an
// endpoint.
func (_self *AdminClient) CreateOrUpdateIpCustomHeader(id string, params CreateOrUpdateIpCustomHeaderRequest) (CreateOrUpdateIpCustomHeaderResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_create_or_update_ip_custom_header(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateOrUpdateIpCustomHeaderRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue CreateOrUpdateIpCustomHeaderResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCreateOrUpdateIpCustomHeaderResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Add a referrer to an endpoint's security settings.
func (_self *AdminClient) CreateReferrer(id string, params CreateReferrerRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_referrer(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateReferrerRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a request filter (method whitelist) on an endpoint.
func (_self *AdminClient) CreateRequestFilter(id string, params CreateRequestFilterRequest) (CreateRequestFilterResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_create_request_filter(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateRequestFilterRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue CreateRequestFilterResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCreateRequestFilterResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Create a new tag on a specific endpoint from a label.
func (_self *AdminClient) CreateTag(id string, params CreateTagRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_tag(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterCreateTagRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new team.
func (_self *AdminClient) CreateTeam(params CreateTeamRequest) (CreateTeamResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_create_team(
				_pointer, FfiConverterCreateTeamRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue CreateTeamResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCreateTeamResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Generate a new authentication token for an endpoint.
func (_self *AdminClient) CreateToken(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_create_token(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Delete an account-level tag. It must first be removed from all endpoints.
func (_self *AdminClient) DeleteAccountTag(id int32) (DeleteAccountTagResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_account_tag(
				_pointer, FfiConverterInt32INSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteAccountTagResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteAccountTagResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove a domain mask from an endpoint by domain mask id.
func (_self *AdminClient) DeleteDomainMask(id string, domainMaskId string) (DeleteBoolResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_domain_mask(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(domainMaskId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteBoolResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteBoolResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove an IP address from an endpoint's security whitelist by ip id.
func (_self *AdminClient) DeleteIp(id string, ipId string) (DeleteBoolResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_ip(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(ipId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteBoolResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteBoolResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove the custom IP header configuration from an endpoint.
func (_self *AdminClient) DeleteIpCustomHeader(id string) (DeleteBoolResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_ip_custom_header(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteBoolResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteBoolResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove a JWT from an endpoint's security configuration by jwt id.
func (_self *AdminClient) DeleteJwt(id string, jwtId string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_delete_jwt(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(jwtId), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a method rate limit from an endpoint by method rate limit id.
func (_self *AdminClient) DeleteMethodRateLimit(id string, methodRateLimitId string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_delete_method_rate_limit(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(methodRateLimitId), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Delete a user-set rate-limit override by its UUID.
func (_self *AdminClient) DeleteRateLimitOverride(id string, overrideId string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_delete_rate_limit_override(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(overrideId), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a referrer from an endpoint's security settings by referrer id.
func (_self *AdminClient) DeleteReferrer(id string, referrerId string) (DeleteBoolResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_referrer(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(referrerId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteBoolResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteBoolResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove a request filter from an endpoint by request filter id.
func (_self *AdminClient) DeleteRequestFilter(id string, requestFilterId string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_delete_request_filter(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(requestFilterId), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a tag from a specific endpoint by tag id.
func (_self *AdminClient) DeleteTag(id string, tagId string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_delete_tag(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(tagId), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Delete a team by id. The team must have no members.
func (_self *AdminClient) DeleteTeam(id int64) (DeleteTeamResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_team(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteTeamResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteTeamResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Revoke a token on an endpoint by token id.
func (_self *AdminClient) DeleteToken(id string, tokenId string) (DeleteBoolResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_delete_token(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(tokenId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue DeleteBoolResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterDeleteBoolResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Disable multichain functionality on an endpoint.
func (_self *AdminClient) DisableMultichain(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_disable_multichain(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Enable multichain functionality on an endpoint.
func (_self *AdminClient) EnableMultichain(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_enable_multichain(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Fetch aggregated metrics across all endpoints on the account.
func (_self *AdminClient) GetAccountMetrics(params GetAccountMetricsRequest) (GetAccountMetricsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_account_metrics(
				_pointer, FfiConverterGetAccountMetricsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetAccountMetricsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetAccountMetricsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch activity logs for a specific endpoint.
func (_self *AdminClient) GetEndpointLogs(id string, params GetEndpointLogsRequest) (GetEndpointLogsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_endpoint_logs(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterGetEndpointLogsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetEndpointLogsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetEndpointLogsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch time-series metrics for a specific endpoint.
func (_self *AdminClient) GetEndpointMetrics(id string, params GetEndpointMetricsRequest) (GetEndpointMetricsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_endpoint_metrics(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterGetEndpointMetricsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetEndpointMetricsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetEndpointMetricsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the full security configuration for an endpoint in a single call.
func (_self *AdminClient) GetEndpointSecurity(id string) (GetEndpointSecurityResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_endpoint_security(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetEndpointSecurityResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetEndpointSecurityResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the HTTP and WebSocket URLs for the endpoint.
func (_self *AdminClient) GetEndpointUrls(id string) (GetEndpointUrlsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_endpoint_urls(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetEndpointUrlsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetEndpointUrlsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List endpoints on the account. Supports searching, filtering, sorting,
// and pagination.
func (_self *AdminClient) GetEndpoints(params GetEndpointsRequest) (GetEndpointsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_endpoints(
				_pointer, FfiConverterGetEndpointsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetEndpointsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetEndpointsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the raw request/response payloads for a specific log entry.
func (_self *AdminClient) GetLogDetails(id string, requestId string) (GetLogDetailsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_log_details(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(requestId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetLogDetailsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetLogDetailsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the method rate limits configured on an endpoint.
func (_self *AdminClient) GetMethodRateLimits(id string) (GetMethodRateLimitsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_method_rate_limits(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetMethodRateLimitsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetMethodRateLimitsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the endpoint-level rate limits currently enforced.
func (_self *AdminClient) GetRateLimits(id string) (GetRateLimitsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_rate_limits(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetRateLimitsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetRateLimitsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the security options (feature toggles) for an endpoint.
func (_self *AdminClient) GetSecurityOptions(id string) (GetSecurityOptionsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_security_options(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetSecurityOptionsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetSecurityOptionsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch a specific team by id.
func (_self *AdminClient) GetTeam(id int64) (GetTeamResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_team(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetTeamResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetTeamResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch account RPC usage totals for an optional time range.
func (_self *AdminClient) GetUsage(params GetUsageRequest) (GetUsageResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_usage(
				_pointer, FfiConverterGetUsageRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetUsageResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetUsageResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch RPC usage grouped by chain over an optional time range.
func (_self *AdminClient) GetUsageByChain(params GetUsageRequest) (GetUsageByChainResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_usage_by_chain(
				_pointer, FfiConverterGetUsageRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetUsageByChainResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetUsageByChainResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch RPC usage broken down per endpoint over an optional time range.
func (_self *AdminClient) GetUsageByEndpoint(params GetUsageRequest) (GetUsageByEndpointResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_usage_by_endpoint(
				_pointer, FfiConverterGetUsageRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetUsageByEndpointResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetUsageByEndpointResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch RPC usage grouped by method over an optional time range.
func (_self *AdminClient) GetUsageByMethod(params GetUsageRequest) (GetUsageByMethodResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_usage_by_method(
				_pointer, FfiConverterGetUsageRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetUsageByMethodResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetUsageByMethodResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch RPC usage grouped by endpoint tag over an optional time range.
func (_self *AdminClient) GetUsageByTag(params GetUsageRequest) (GetUsageByTagResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_get_usage_by_tag(
				_pointer, FfiConverterGetUsageRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetUsageByTagResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetUsageByTagResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Invite a user to a team by email.
func (_self *AdminClient) InviteTeamMember(id int64, params InviteTeamMemberRequest) (InviteTeamMemberResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_invite_team_member(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), FfiConverterInviteTeamMemberRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue InviteTeamMemberResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterInviteTeamMemberResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List all chains supported by Quicknode along with their networks.
func (_self *AdminClient) ListChains() (ListChainsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_chains(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListChainsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListChainsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List the account's invoices.
func (_self *AdminClient) ListInvoices() (ListInvoicesResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_invoices(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListInvoicesResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListInvoicesResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List all payments on the account.
func (_self *AdminClient) ListPayments() (ListPaymentsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_payments(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListPaymentsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListPaymentsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List all account-level tags, including tags with zero endpoints.
func (_self *AdminClient) ListTags() (ListTagsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_tags(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListTagsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListTagsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List the endpoints accessible to a given team.
func (_self *AdminClient) ListTeamEndpoints(id int64) (ListTeamEndpointsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_team_endpoints(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListTeamEndpointsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListTeamEndpointsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// List all teams on the account.
func (_self *AdminClient) ListTeams() (ListTeamsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_list_teams(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListTeamsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListTeamsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove a user from a team by team id and user id.
func (_self *AdminClient) RemoveTeamMember(id int64, userId int64, params RemoveTeamMemberRequest) (RemoveTeamMemberResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_remove_team_member(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), FfiConverterInt64INSTANCE.Lower(userId), FfiConverterRemoveTeamMemberRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue RemoveTeamMemberResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterRemoveTeamMemberResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update the label of an account tag.
func (_self *AdminClient) RenameTag(id int32, params RenameTagRequest) (RenameTagResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_rename_tag(
				_pointer, FfiConverterInt32INSTANCE.Lower(id), FfiConverterRenameTagRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue RenameTagResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterRenameTagResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Resend the invitation email to a pending team member.
func (_self *AdminClient) ResendTeamInvite(id int64, userId int64) (ResendTeamInviteResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_resend_team_invite(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), FfiConverterInt64INSTANCE.Lower(userId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ResendTeamInviteResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterResendTeamInviteResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch details for a specific endpoint by ID.
func (_self *AdminClient) ShowEndpoint(id string) (ShowEndpointResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_show_endpoint(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ShowEndpointResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterShowEndpointResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update editable fields on an endpoint (e.g. its label).
func (_self *AdminClient) UpdateEndpoint(id string, params UpdateEndpointRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_update_endpoint(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateEndpointRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Pause or unpause an endpoint by setting its status.
func (_self *AdminClient) UpdateEndpointStatus(id string, params UpdateEndpointStatusRequest) (UpdateEndpointStatusResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_update_endpoint_status(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateEndpointStatusRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue UpdateEndpointStatusResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUpdateEndpointStatusResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update an existing method rate limit on an endpoint.
func (_self *AdminClient) UpdateMethodRateLimit(id string, methodRateLimitId string, params UpdateMethodRateLimitRequest) (UpdateMethodRateLimitResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_update_method_rate_limit(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(methodRateLimitId), FfiConverterUpdateMethodRateLimitRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue UpdateMethodRateLimitResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUpdateMethodRateLimitResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Partially update the endpoint-level rate-limit overrides.
func (_self *AdminClient) UpdateRateLimits(id string, params UpdateRateLimitsRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_update_rate_limits(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateRateLimitsRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Update an existing request filter on an endpoint.
func (_self *AdminClient) UpdateRequestFilter(id string, requestFilterId string, params UpdateRequestFilterRequest) error {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_adminclient_update_request_filter(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterStringINSTANCE.Lower(requestFilterId), FfiConverterUpdateRequestFilterRequestINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Update which security features are enabled on an endpoint.
func (_self *AdminClient) UpdateSecurityOptions(id string, params UpdateSecurityOptionsRequest) (UpdateSecurityOptionsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_update_security_options(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateSecurityOptionsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue UpdateSecurityOptionsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUpdateSecurityOptionsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Assign or unassign endpoints for a team.
func (_self *AdminClient) UpdateTeamEndpoints(id int64, params UpdateTeamEndpointsRequest) (UpdateTeamEndpointsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*AdminClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_adminclient_update_team_endpoints(
				_pointer, FfiConverterInt64INSTANCE.Lower(id), FfiConverterUpdateTeamEndpointsRequestINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue UpdateTeamEndpointsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterUpdateTeamEndpointsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}
func (object *AdminClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterAdminClient struct{}

var FfiConverterAdminClientINSTANCE = FfiConverterAdminClient{}

func (c FfiConverterAdminClient) Lift(handle C.uint64_t) *AdminClient {
	result := &AdminClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_adminclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_adminclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*AdminClient).Destroy)
	return result
}

func (c FfiConverterAdminClient) Read(reader io.Reader) *AdminClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterAdminClient) Lower(value *AdminClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*AdminClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterAdminClient) Write(writer io.Writer, value *AdminClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalAdminClient(handle uint64) *AdminClient {
	return FfiConverterAdminClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalAdminClient(value *AdminClient) uint64 {
	return uint64(FfiConverterAdminClientINSTANCE.Lower(value))
}

type FfiDestroyerAdminClient struct{}

func (_ FfiDestroyerAdminClient) Destroy(value *AdminClient) {
	value.Destroy()
}

// KvStore API sub-client.
type KvStoreClientInterface interface {
	// Append a single item to the list identified by `key`.
	AddListItem(key string, params AddListItemParams) error
	// Add and remove multiple sets in a single request.
	BulkSets(params BulkSetsParams) error
	// Create a new list under the given key, seeded with the provided items.
	CreateList(params CreateListParams) error
	// Create a new set, storing a single string value under the given key.
	CreateSet(params CreateSetParams) error
	// Remove a list and all of its items by key.
	DeleteList(key string) error
	// Remove a specific item from the list identified by `key`.
	DeleteListItem(key string, item string) error
	// Remove a single set by key.
	DeleteSet(key string) error
	// Fetch a paginated page of items from the list identified by `key`.
	GetList(key string, params GetListParams) (GetListResponse, error)
	// Fetch a paginated page of list keys from the store.
	GetLists(params GetListsParams) (GetListsResponse, error)
	// Fetch the string value stored for a single set by key.
	GetSet(key string) (GetSetResponse, error)
	// Fetch a paginated page of key/value entries from the store.
	GetSets(params GetSetsParams) (GetSetsResponse, error)
	// Check whether the specified list contains the given item.
	ListContainsItem(key string, item string) (ListContainsItemResponse, error)
	// Update an existing list by adding and/or removing items.
	UpdateList(key string, params UpdateListParams) error
}

// KvStore API sub-client.
type KvStoreClient struct {
	ffiObject FfiObject
}

// Append a single item to the list identified by `key`.
func (_self *KvStoreClient) AddListItem(key string, params AddListItemParams) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_add_list_item(
			_pointer, FfiConverterStringINSTANCE.Lower(key), FfiConverterAddListItemParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Add and remove multiple sets in a single request.
func (_self *KvStoreClient) BulkSets(params BulkSetsParams) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_bulk_sets(
			_pointer, FfiConverterBulkSetsParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new list under the given key, seeded with the provided items.
func (_self *KvStoreClient) CreateList(params CreateListParams) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_create_list(
			_pointer, FfiConverterCreateListParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new set, storing a single string value under the given key.
func (_self *KvStoreClient) CreateSet(params CreateSetParams) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_create_set(
			_pointer, FfiConverterCreateSetParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a list and all of its items by key.
func (_self *KvStoreClient) DeleteList(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_delete_list(
			_pointer, FfiConverterStringINSTANCE.Lower(key), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a specific item from the list identified by `key`.
func (_self *KvStoreClient) DeleteListItem(key string, item string) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_delete_list_item(
			_pointer, FfiConverterStringINSTANCE.Lower(key), FfiConverterStringINSTANCE.Lower(item), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Remove a single set by key.
func (_self *KvStoreClient) DeleteSet(key string) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_delete_set(
			_pointer, FfiConverterStringINSTANCE.Lower(key), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Fetch a paginated page of items from the list identified by `key`.
func (_self *KvStoreClient) GetList(key string, params GetListParams) (GetListResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_kvstoreclient_get_list(
				_pointer, FfiConverterStringINSTANCE.Lower(key), FfiConverterGetListParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetListResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetListResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch a paginated page of list keys from the store.
func (_self *KvStoreClient) GetLists(params GetListsParams) (GetListsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_kvstoreclient_get_lists(
				_pointer, FfiConverterGetListsParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetListsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetListsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch the string value stored for a single set by key.
func (_self *KvStoreClient) GetSet(key string) (GetSetResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_kvstoreclient_get_set(
				_pointer, FfiConverterStringINSTANCE.Lower(key), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetSetResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetSetResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch a paginated page of key/value entries from the store.
func (_self *KvStoreClient) GetSets(params GetSetsParams) (GetSetsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_kvstoreclient_get_sets(
				_pointer, FfiConverterGetSetsParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue GetSetsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterGetSetsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Check whether the specified list contains the given item.
func (_self *KvStoreClient) ListContainsItem(key string, item string) (ListContainsItemResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_kvstoreclient_list_contains_item(
				_pointer, FfiConverterStringINSTANCE.Lower(key), FfiConverterStringINSTANCE.Lower(item), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListContainsItemResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListContainsItemResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update an existing list by adding and/or removing items.
func (_self *KvStoreClient) UpdateList(key string, params UpdateListParams) error {
	_pointer := _self.ffiObject.incrementPointer("*KvStoreClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_kvstoreclient_update_list(
			_pointer, FfiConverterStringINSTANCE.Lower(key), FfiConverterUpdateListParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}
func (object *KvStoreClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterKvStoreClient struct{}

var FfiConverterKvStoreClientINSTANCE = FfiConverterKvStoreClient{}

func (c FfiConverterKvStoreClient) Lift(handle C.uint64_t) *KvStoreClient {
	result := &KvStoreClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_kvstoreclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_kvstoreclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*KvStoreClient).Destroy)
	return result
}

func (c FfiConverterKvStoreClient) Read(reader io.Reader) *KvStoreClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterKvStoreClient) Lower(value *KvStoreClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*KvStoreClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterKvStoreClient) Write(writer io.Writer, value *KvStoreClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalKvStoreClient(handle uint64) *KvStoreClient {
	return FfiConverterKvStoreClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalKvStoreClient(value *KvStoreClient) uint64 {
	return uint64(FfiConverterKvStoreClientINSTANCE.Lower(value))
}

type FfiDestroyerKvStoreClient struct{}

func (_ FfiDestroyerKvStoreClient) Destroy(value *KvStoreClient) {
	value.Destroy()
}

// Root Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and hands
// out per-product sub-clients.
type QuicknodeSdkClientInterface interface {
	// Admin API sub-client: endpoints, tags, teams, billing, usage, metrics,
	// security, and rate limits.
	Admin() *AdminClient
	// Key-Value Store sub-client: manage sets and lists.
	Kvstore() *KvStoreClient
	// SQL API sub-client: run SQL queries and fetch schemas.
	Sql() *SqlClient
	// Streams API sub-client: create and manage blockchain data streams.
	Streams() *StreamsClient
	// Webhooks API sub-client: create webhooks from templates and manage their
	// lifecycle.
	Webhooks() *WebhooksClient
}

// Root Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and hands
// out per-product sub-clients.
type QuicknodeSdkClient struct {
	ffiObject FfiObject
}

// Construct an SDK client from an API key. The `User-Agent` is attributed
// to the Go binding.
func NewQuicknodeSdkClient(apiKey string) (*QuicknodeSdkClient, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_constructor_quicknodesdkclient_new(FfiConverterStringINSTANCE.Lower(apiKey), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *QuicknodeSdkClient
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterQuicknodeSdkClientINSTANCE.Lift(_uniffiRV), nil
	}
}

// Construct an SDK client overriding one or more sub-client base URLs.
// Useful for testing against a mock server or pointing at a proxy;
// production callers use [`Self::new`]. Any field left `None` uses the
// default Quicknode endpoint.
func QuicknodeSdkClientNewWithBaseUrls(apiKey string, overrides BaseUrlOverrides) (*QuicknodeSdkClient, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_constructor_quicknodesdkclient_new_with_base_urls(FfiConverterStringINSTANCE.Lower(apiKey), FfiConverterBaseUrlOverridesINSTANCE.Lower(overrides), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *QuicknodeSdkClient
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterQuicknodeSdkClientINSTANCE.Lift(_uniffiRV), nil
	}
}

// Admin API sub-client: endpoints, tags, teams, billing, usage, metrics,
// security, and rate limits.
func (_self *QuicknodeSdkClient) Admin() *AdminClient {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterAdminClientINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_admin(
			_pointer, _uniffiStatus)
	}))
}

// Key-Value Store sub-client: manage sets and lists.
func (_self *QuicknodeSdkClient) Kvstore() *KvStoreClient {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterKvStoreClientINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_kvstore(
			_pointer, _uniffiStatus)
	}))
}

// SQL API sub-client: run SQL queries and fetch schemas.
func (_self *QuicknodeSdkClient) Sql() *SqlClient {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSqlClientINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_sql(
			_pointer, _uniffiStatus)
	}))
}

// Streams API sub-client: create and manage blockchain data streams.
func (_self *QuicknodeSdkClient) Streams() *StreamsClient {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterStreamsClientINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_streams(
			_pointer, _uniffiStatus)
	}))
}

// Webhooks API sub-client: create webhooks from templates and manage their
// lifecycle.
func (_self *QuicknodeSdkClient) Webhooks() *WebhooksClient {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterWebhooksClientINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_webhooks(
			_pointer, _uniffiStatus)
	}))
}
func (object *QuicknodeSdkClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterQuicknodeSdkClient struct{}

var FfiConverterQuicknodeSdkClientINSTANCE = FfiConverterQuicknodeSdkClient{}

func (c FfiConverterQuicknodeSdkClient) Lift(handle C.uint64_t) *QuicknodeSdkClient {
	result := &QuicknodeSdkClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_quicknodesdkclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_quicknodesdkclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*QuicknodeSdkClient).Destroy)
	return result
}

func (c FfiConverterQuicknodeSdkClient) Read(reader io.Reader) *QuicknodeSdkClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterQuicknodeSdkClient) Lower(value *QuicknodeSdkClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterQuicknodeSdkClient) Write(writer io.Writer, value *QuicknodeSdkClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalQuicknodeSdkClient(handle uint64) *QuicknodeSdkClient {
	return FfiConverterQuicknodeSdkClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalQuicknodeSdkClient(value *QuicknodeSdkClient) uint64 {
	return uint64(FfiConverterQuicknodeSdkClientINSTANCE.Lower(value))
}

type FfiDestroyerQuicknodeSdkClient struct{}

func (_ FfiDestroyerQuicknodeSdkClient) Destroy(value *QuicknodeSdkClient) {
	value.Destroy()
}

// SQL API sub-client.
type SqlClientInterface interface {
	// Fetch the database schema for a cluster.
	GetSchema(clusterId string) (ChainSchema, error)
	// Execute a SQL query against the given cluster and return the result set.
	Query(params QueryParams) (QueryResponse, error)
}

// SQL API sub-client.
type SqlClient struct {
	ffiObject FfiObject
}

// Fetch the database schema for a cluster.
func (_self *SqlClient) GetSchema(clusterId string) (ChainSchema, error) {
	_pointer := _self.ffiObject.incrementPointer("*SqlClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_sqlclient_get_schema(
				_pointer, FfiConverterStringINSTANCE.Lower(clusterId), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ChainSchema
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterChainSchemaINSTANCE.Lift(_uniffiRV), nil
	}
}

// Execute a SQL query against the given cluster and return the result set.
func (_self *SqlClient) Query(params QueryParams) (QueryResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*SqlClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_sqlclient_query(
				_pointer, FfiConverterQueryParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue QueryResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterQueryResponseINSTANCE.Lift(_uniffiRV), nil
	}
}
func (object *SqlClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterSqlClient struct{}

var FfiConverterSqlClientINSTANCE = FfiConverterSqlClient{}

func (c FfiConverterSqlClient) Lift(handle C.uint64_t) *SqlClient {
	result := &SqlClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_sqlclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_sqlclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*SqlClient).Destroy)
	return result
}

func (c FfiConverterSqlClient) Read(reader io.Reader) *SqlClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterSqlClient) Lower(value *SqlClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*SqlClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterSqlClient) Write(writer io.Writer, value *SqlClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalSqlClient(handle uint64) *SqlClient {
	return FfiConverterSqlClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalSqlClient(value *SqlClient) uint64 {
	return uint64(FfiConverterSqlClientINSTANCE.Lower(value))
}

type FfiDestroyerSqlClient struct{}

func (_ FfiDestroyerSqlClient) Destroy(value *SqlClient) {
	value.Destroy()
}

// Streams API sub-client.
type StreamsClientInterface interface {
	// Activate a stream by id.
	ActivateStream(id string) error
	// Create a new stream.
	CreateStream(params CreateStreamParams) (Stream, error)
	// Delete all streams on the account.
	DeleteAllStreams() error
	// Delete a stream by id.
	DeleteStream(id string) error
	// Count currently enabled streams, optionally filtered by type.
	GetEnabledCount(streamType *string) (EnabledCountResponse, error)
	// Fetch a single stream by id.
	GetStream(id string) (Stream, error)
	// List streams on the account.
	ListStreams(params ListStreamsParams) (ListStreamsResponse, error)
	// Pause a stream by id.
	PauseStream(id string) error
	// Test a filter function against a stream configuration.
	TestFilter(params TestFilterParams) (TestFilterResponse, error)
	// Update a stream. Only set fields are modified.
	UpdateStream(id string, params UpdateStreamParams) (Stream, error)
}

// Streams API sub-client.
type StreamsClient struct {
	ffiObject FfiObject
}

// Activate a stream by id.
func (_self *StreamsClient) ActivateStream(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_streamsclient_activate_stream(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new stream.
func (_self *StreamsClient) CreateStream(params CreateStreamParams) (Stream, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_create_stream(
				_pointer, FfiConverterCreateStreamParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Stream
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStreamINSTANCE.Lift(_uniffiRV), nil
	}
}

// Delete all streams on the account.
func (_self *StreamsClient) DeleteAllStreams() error {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_streamsclient_delete_all_streams(
			_pointer, _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Delete a stream by id.
func (_self *StreamsClient) DeleteStream(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_streamsclient_delete_stream(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Count currently enabled streams, optionally filtered by type.
func (_self *StreamsClient) GetEnabledCount(streamType *string) (EnabledCountResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_get_enabled_count(
				_pointer, FfiConverterOptionalStringINSTANCE.Lower(streamType), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue EnabledCountResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterEnabledCountResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch a single stream by id.
func (_self *StreamsClient) GetStream(id string) (Stream, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_get_stream(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Stream
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStreamINSTANCE.Lift(_uniffiRV), nil
	}
}

// List streams on the account.
func (_self *StreamsClient) ListStreams(params ListStreamsParams) (ListStreamsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_list_streams(
				_pointer, FfiConverterListStreamsParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListStreamsResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListStreamsResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Pause a stream by id.
func (_self *StreamsClient) PauseStream(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_streamsclient_pause_stream(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Test a filter function against a stream configuration.
func (_self *StreamsClient) TestFilter(params TestFilterParams) (TestFilterResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_test_filter(
				_pointer, FfiConverterTestFilterParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue TestFilterResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterTestFilterResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update a stream. Only set fields are modified.
func (_self *StreamsClient) UpdateStream(id string, params UpdateStreamParams) (Stream, error) {
	_pointer := _self.ffiObject.incrementPointer("*StreamsClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_streamsclient_update_stream(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateStreamParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Stream
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStreamINSTANCE.Lift(_uniffiRV), nil
	}
}
func (object *StreamsClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterStreamsClient struct{}

var FfiConverterStreamsClientINSTANCE = FfiConverterStreamsClient{}

func (c FfiConverterStreamsClient) Lift(handle C.uint64_t) *StreamsClient {
	result := &StreamsClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_streamsclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_streamsclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*StreamsClient).Destroy)
	return result
}

func (c FfiConverterStreamsClient) Read(reader io.Reader) *StreamsClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterStreamsClient) Lower(value *StreamsClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*StreamsClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterStreamsClient) Write(writer io.Writer, value *StreamsClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalStreamsClient(handle uint64) *StreamsClient {
	return FfiConverterStreamsClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalStreamsClient(value *StreamsClient) uint64 {
	return uint64(FfiConverterStreamsClientINSTANCE.Lower(value))
}

type FfiDestroyerStreamsClient struct{}

func (_ FfiDestroyerStreamsClient) Destroy(value *StreamsClient) {
	value.Destroy()
}

// Webhooks API sub-client.
type WebhooksClientInterface interface {
	// Activate a created or paused webhook so it begins delivering events.
	ActivateWebhook(id string, params ActivateWebhookParams) error
	// Create a new webhook from a predefined filter template.
	CreateWebhookFromTemplate(params CreateWebhookFromTemplateParams) (Webhook, error)
	// Remove every webhook on the account.
	DeleteAllWebhooks() error
	// Permanently remove a single webhook by ID.
	DeleteWebhook(id string) error
	// Count the enabled webhooks currently configured on the account.
	GetEnabledCount() (WebhookEnabledCountResponse, error)
	// Fetch a single webhook's full configuration and status by ID.
	GetWebhook(id string) (Webhook, error)
	// List webhooks on the account with pagination.
	ListWebhooks(params GetWebhooksParams) (ListWebhooksResponse, error)
	// Pause a webhook by ID so it stops delivering events until reactivated.
	PauseWebhook(id string) error
	// Modify an existing webhook's configuration.
	UpdateWebhook(id string, params UpdateWebhookParams) (Webhook, error)
	// Update an existing template-backed webhook's template arguments.
	UpdateWebhookTemplate(webhookId string, params UpdateWebhookTemplateParams) (Webhook, error)
}

// Webhooks API sub-client.
type WebhooksClient struct {
	ffiObject FfiObject
}

// Activate a created or paused webhook so it begins delivering events.
func (_self *WebhooksClient) ActivateWebhook(id string, params ActivateWebhookParams) error {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_webhooksclient_activate_webhook(
			_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterActivateWebhookParamsINSTANCE.Lower(params), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Create a new webhook from a predefined filter template.
func (_self *WebhooksClient) CreateWebhookFromTemplate(params CreateWebhookFromTemplateParams) (Webhook, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_create_webhook_from_template(
				_pointer, FfiConverterCreateWebhookFromTemplateParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Webhook
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterWebhookINSTANCE.Lift(_uniffiRV), nil
	}
}

// Remove every webhook on the account.
func (_self *WebhooksClient) DeleteAllWebhooks() error {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_webhooksclient_delete_all_webhooks(
			_pointer, _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Permanently remove a single webhook by ID.
func (_self *WebhooksClient) DeleteWebhook(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_webhooksclient_delete_webhook(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Count the enabled webhooks currently configured on the account.
func (_self *WebhooksClient) GetEnabledCount() (WebhookEnabledCountResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_get_enabled_count(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue WebhookEnabledCountResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterWebhookEnabledCountResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Fetch a single webhook's full configuration and status by ID.
func (_self *WebhooksClient) GetWebhook(id string) (Webhook, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_get_webhook(
				_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Webhook
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterWebhookINSTANCE.Lift(_uniffiRV), nil
	}
}

// List webhooks on the account with pagination.
func (_self *WebhooksClient) ListWebhooks(params GetWebhooksParams) (ListWebhooksResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_list_webhooks(
				_pointer, FfiConverterGetWebhooksParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ListWebhooksResponse
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterListWebhooksResponseINSTANCE.Lift(_uniffiRV), nil
	}
}

// Pause a webhook by ID so it stops delivering events until reactivated.
func (_self *WebhooksClient) PauseWebhook(id string) error {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_quicknode_sdk_fn_method_webhooksclient_pause_webhook(
			_pointer, FfiConverterStringINSTANCE.Lower(id), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Modify an existing webhook's configuration.
func (_self *WebhooksClient) UpdateWebhook(id string, params UpdateWebhookParams) (Webhook, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_update_webhook(
				_pointer, FfiConverterStringINSTANCE.Lower(id), FfiConverterUpdateWebhookParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Webhook
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterWebhookINSTANCE.Lift(_uniffiRV), nil
	}
}

// Update an existing template-backed webhook's template arguments.
func (_self *WebhooksClient) UpdateWebhookTemplate(webhookId string, params UpdateWebhookTemplateParams) (Webhook, error) {
	_pointer := _self.ffiObject.incrementPointer("*WebhooksClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_webhooksclient_update_webhook_template(
				_pointer, FfiConverterStringINSTANCE.Lower(webhookId), FfiConverterUpdateWebhookTemplateParamsINSTANCE.Lower(params), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue Webhook
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterWebhookINSTANCE.Lift(_uniffiRV), nil
	}
}
func (object *WebhooksClient) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterWebhooksClient struct{}

var FfiConverterWebhooksClientINSTANCE = FfiConverterWebhooksClient{}

func (c FfiConverterWebhooksClient) Lift(handle C.uint64_t) *WebhooksClient {
	result := &WebhooksClient{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_quicknode_sdk_fn_clone_webhooksclient(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_quicknode_sdk_fn_free_webhooksclient(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*WebhooksClient).Destroy)
	return result
}

func (c FfiConverterWebhooksClient) Read(reader io.Reader) *WebhooksClient {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterWebhooksClient) Lower(value *WebhooksClient) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*WebhooksClient")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterWebhooksClient) Write(writer io.Writer, value *WebhooksClient) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalWebhooksClient(handle uint64) *WebhooksClient {
	return FfiConverterWebhooksClientINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalWebhooksClient(value *WebhooksClient) uint64 {
	return uint64(FfiConverterWebhooksClientINSTANCE.Lower(value))
}

type FfiDestroyerWebhooksClient struct{}

func (_ FfiDestroyerWebhooksClient) Destroy(value *WebhooksClient) {
	value.Destroy()
}

// An account-level tag, shared across endpoints.
type AccountTag struct {
	// Tag identifier.
	Id int32
	// Tag label.
	Label string
	// Number of endpoints the tag is applied to.
	UsageCount int32
}

func (r *AccountTag) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Label)
	FfiDestroyerInt32{}.Destroy(r.UsageCount)
}

type FfiConverterAccountTag struct{}

var FfiConverterAccountTagINSTANCE = FfiConverterAccountTag{}

func (c FfiConverterAccountTag) Lift(rb RustBufferI) AccountTag {
	return LiftFromRustBuffer[AccountTag](c, rb)
}

func (c FfiConverterAccountTag) Read(reader io.Reader) AccountTag {
	return AccountTag{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterAccountTag) Lower(value AccountTag) C.RustBuffer {
	return LowerIntoRustBuffer[AccountTag](c, value)
}

func (c FfiConverterAccountTag) LowerExternal(value AccountTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[AccountTag](c, value))
}

func (c FfiConverterAccountTag) Write(writer io.Writer, value AccountTag) {
	FfiConverterInt32INSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Label)
	FfiConverterInt32INSTANCE.Write(writer, value.UsageCount)
}

type FfiDestroyerAccountTag struct{}

func (_ FfiDestroyerAccountTag) Destroy(value AccountTag) {
	value.Destroy()
}

// Parameters for `activate_webhook`.
type ActivateWebhookParams struct {
	// Position to begin (or resume) delivery from.
	StartFrom WebhookStartFrom
}

func (r *ActivateWebhookParams) Destroy() {
	FfiDestroyerWebhookStartFrom{}.Destroy(r.StartFrom)
}

type FfiConverterActivateWebhookParams struct{}

var FfiConverterActivateWebhookParamsINSTANCE = FfiConverterActivateWebhookParams{}

func (c FfiConverterActivateWebhookParams) Lift(rb RustBufferI) ActivateWebhookParams {
	return LiftFromRustBuffer[ActivateWebhookParams](c, rb)
}

func (c FfiConverterActivateWebhookParams) Read(reader io.Reader) ActivateWebhookParams {
	return ActivateWebhookParams{
		FfiConverterWebhookStartFromINSTANCE.Read(reader),
	}
}

func (c FfiConverterActivateWebhookParams) Lower(value ActivateWebhookParams) C.RustBuffer {
	return LowerIntoRustBuffer[ActivateWebhookParams](c, value)
}

func (c FfiConverterActivateWebhookParams) LowerExternal(value ActivateWebhookParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ActivateWebhookParams](c, value))
}

func (c FfiConverterActivateWebhookParams) Write(writer io.Writer, value ActivateWebhookParams) {
	FfiConverterWebhookStartFromINSTANCE.Write(writer, value.StartFrom)
}

type FfiDestroyerActivateWebhookParams struct{}

func (_ FfiDestroyerActivateWebhookParams) Destroy(value ActivateWebhookParams) {
	value.Destroy()
}

// Parameters for `add_list_item`.
type AddListItemParams struct {
	// Item to append to the list.
	Item string
}

func (r *AddListItemParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Item)
}

type FfiConverterAddListItemParams struct{}

var FfiConverterAddListItemParamsINSTANCE = FfiConverterAddListItemParams{}

func (c FfiConverterAddListItemParams) Lift(rb RustBufferI) AddListItemParams {
	return LiftFromRustBuffer[AddListItemParams](c, rb)
}

func (c FfiConverterAddListItemParams) Read(reader io.Reader) AddListItemParams {
	return AddListItemParams{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterAddListItemParams) Lower(value AddListItemParams) C.RustBuffer {
	return LowerIntoRustBuffer[AddListItemParams](c, value)
}

func (c FfiConverterAddListItemParams) LowerExternal(value AddListItemParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[AddListItemParams](c, value))
}

func (c FfiConverterAddListItemParams) Write(writer io.Writer, value AddListItemParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Item)
}

type FfiDestroyerAddListItemParams struct{}

func (_ FfiDestroyerAddListItemParams) Destroy(value AddListItemParams) {
	value.Destroy()
}

// Links a stream's filter to an address book so JSON paths resolve against its
// managed address set.
type AddressBookConfig struct {
	// Identifier of the address book to use.
	AddressBookId string
	// Optional JSON path that resolves to an object whose fields are matched against the book.
	ObjectsFilterPath *string
	// JSON paths whose resolved values are matched against the book's addresses.
	ElementsFilterPaths []string
}

func (r *AddressBookConfig) Destroy() {
	FfiDestroyerString{}.Destroy(r.AddressBookId)
	FfiDestroyerOptionalString{}.Destroy(r.ObjectsFilterPath)
	FfiDestroyerSequenceString{}.Destroy(r.ElementsFilterPaths)
}

type FfiConverterAddressBookConfig struct{}

var FfiConverterAddressBookConfigINSTANCE = FfiConverterAddressBookConfig{}

func (c FfiConverterAddressBookConfig) Lift(rb RustBufferI) AddressBookConfig {
	return LiftFromRustBuffer[AddressBookConfig](c, rb)
}

func (c FfiConverterAddressBookConfig) Read(reader io.Reader) AddressBookConfig {
	return AddressBookConfig{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterAddressBookConfig) Lower(value AddressBookConfig) C.RustBuffer {
	return LowerIntoRustBuffer[AddressBookConfig](c, value)
}

func (c FfiConverterAddressBookConfig) LowerExternal(value AddressBookConfig) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[AddressBookConfig](c, value))
}

func (c FfiConverterAddressBookConfig) Write(writer io.Writer, value AddressBookConfig) {
	FfiConverterStringINSTANCE.Write(writer, value.AddressBookId)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.ObjectsFilterPath)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.ElementsFilterPaths)
}

type FfiDestroyerAddressBookConfig struct{}

func (_ FfiDestroyerAddressBookConfig) Destroy(value AddressBookConfig) {
	value.Destroy()
}

// Configuration for delivering stream batches to Azure Blob Storage.
type AzureAttributes struct {
	// Azure storage account name.
	StorageAccount string
	// SAS token used to authorize writes.
	SasToken string
	// Container that receives written blobs.
	Container string
	// Compression applied to written blobs (e.g. `none`, `gzip`).
	Compression string
	// File format/extension for written blobs (e.g. `.json`).
	FileType string
	// Maximum number of retry attempts for a failed write.
	MaxRetry int32
	// Seconds to wait between retry attempts.
	RetryIntervalSec int32
	// Optional name prefix prepended to each written blob.
	BlobPrefix *string
}

func (r *AzureAttributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.StorageAccount)
	FfiDestroyerString{}.Destroy(r.SasToken)
	FfiDestroyerString{}.Destroy(r.Container)
	FfiDestroyerString{}.Destroy(r.Compression)
	FfiDestroyerString{}.Destroy(r.FileType)
	FfiDestroyerInt32{}.Destroy(r.MaxRetry)
	FfiDestroyerInt32{}.Destroy(r.RetryIntervalSec)
	FfiDestroyerOptionalString{}.Destroy(r.BlobPrefix)
}

type FfiConverterAzureAttributes struct{}

var FfiConverterAzureAttributesINSTANCE = FfiConverterAzureAttributes{}

func (c FfiConverterAzureAttributes) Lift(rb RustBufferI) AzureAttributes {
	return LiftFromRustBuffer[AzureAttributes](c, rb)
}

func (c FfiConverterAzureAttributes) Read(reader io.Reader) AzureAttributes {
	return AzureAttributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterAzureAttributes) Lower(value AzureAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[AzureAttributes](c, value)
}

func (c FfiConverterAzureAttributes) LowerExternal(value AzureAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[AzureAttributes](c, value))
}

func (c FfiConverterAzureAttributes) Write(writer io.Writer, value AzureAttributes) {
	FfiConverterStringINSTANCE.Write(writer, value.StorageAccount)
	FfiConverterStringINSTANCE.Write(writer, value.SasToken)
	FfiConverterStringINSTANCE.Write(writer, value.Container)
	FfiConverterStringINSTANCE.Write(writer, value.Compression)
	FfiConverterStringINSTANCE.Write(writer, value.FileType)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxRetry)
	FfiConverterInt32INSTANCE.Write(writer, value.RetryIntervalSec)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.BlobPrefix)
}

type FfiDestroyerAzureAttributes struct{}

func (_ FfiDestroyerAzureAttributes) Destroy(value AzureAttributes) {
	value.Destroy()
}

// Per-sub-client base URL overrides for [`QuicknodeSdkClient::new_with_base_urls`].
// Each `None` field falls back to the default Quicknode endpoint.
type BaseUrlOverrides struct {
	Admin    *string
	Streams  *string
	Webhooks *string
	Kvstore  *string
	Sql      *string
}

func (r *BaseUrlOverrides) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Admin)
	FfiDestroyerOptionalString{}.Destroy(r.Streams)
	FfiDestroyerOptionalString{}.Destroy(r.Webhooks)
	FfiDestroyerOptionalString{}.Destroy(r.Kvstore)
	FfiDestroyerOptionalString{}.Destroy(r.Sql)
}

type FfiConverterBaseUrlOverrides struct{}

var FfiConverterBaseUrlOverridesINSTANCE = FfiConverterBaseUrlOverrides{}

func (c FfiConverterBaseUrlOverrides) Lift(rb RustBufferI) BaseUrlOverrides {
	return LiftFromRustBuffer[BaseUrlOverrides](c, rb)
}

func (c FfiConverterBaseUrlOverrides) Read(reader io.Reader) BaseUrlOverrides {
	return BaseUrlOverrides{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBaseUrlOverrides) Lower(value BaseUrlOverrides) C.RustBuffer {
	return LowerIntoRustBuffer[BaseUrlOverrides](c, value)
}

func (c FfiConverterBaseUrlOverrides) LowerExternal(value BaseUrlOverrides) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BaseUrlOverrides](c, value))
}

func (c FfiConverterBaseUrlOverrides) Write(writer io.Writer, value BaseUrlOverrides) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Admin)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Streams)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Webhooks)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Kvstore)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Sql)
}

type FfiDestroyerBaseUrlOverrides struct{}

func (_ FfiDestroyerBaseUrlOverrides) Destroy(value BaseUrlOverrides) {
	value.Destroy()
}

// ByList form of `BitcoinWalletFilterTemplate`.
type BitcoinWalletFilterByListTemplate struct {
	// Name of the pre-created wallets list.
	WalletsListName string
}

func (r *BitcoinWalletFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.WalletsListName)
}

type FfiConverterBitcoinWalletFilterByListTemplate struct{}

var FfiConverterBitcoinWalletFilterByListTemplateINSTANCE = FfiConverterBitcoinWalletFilterByListTemplate{}

func (c FfiConverterBitcoinWalletFilterByListTemplate) Lift(rb RustBufferI) BitcoinWalletFilterByListTemplate {
	return LiftFromRustBuffer[BitcoinWalletFilterByListTemplate](c, rb)
}

func (c FfiConverterBitcoinWalletFilterByListTemplate) Read(reader io.Reader) BitcoinWalletFilterByListTemplate {
	return BitcoinWalletFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBitcoinWalletFilterByListTemplate) Lower(value BitcoinWalletFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[BitcoinWalletFilterByListTemplate](c, value)
}

func (c FfiConverterBitcoinWalletFilterByListTemplate) LowerExternal(value BitcoinWalletFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BitcoinWalletFilterByListTemplate](c, value))
}

func (c FfiConverterBitcoinWalletFilterByListTemplate) Write(writer io.Writer, value BitcoinWalletFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.WalletsListName)
}

type FfiDestroyerBitcoinWalletFilterByListTemplate struct{}

func (_ FfiDestroyerBitcoinWalletFilterByListTemplate) Destroy(value BitcoinWalletFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for a Bitcoin wallet filter.
type BitcoinWalletFilterTemplate struct {
	// Bitcoin wallet addresses to match against.
	Wallets []string
}

func (r *BitcoinWalletFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Wallets)
}

type FfiConverterBitcoinWalletFilterTemplate struct{}

var FfiConverterBitcoinWalletFilterTemplateINSTANCE = FfiConverterBitcoinWalletFilterTemplate{}

func (c FfiConverterBitcoinWalletFilterTemplate) Lift(rb RustBufferI) BitcoinWalletFilterTemplate {
	return LiftFromRustBuffer[BitcoinWalletFilterTemplate](c, rb)
}

func (c FfiConverterBitcoinWalletFilterTemplate) Read(reader io.Reader) BitcoinWalletFilterTemplate {
	return BitcoinWalletFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBitcoinWalletFilterTemplate) Lower(value BitcoinWalletFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[BitcoinWalletFilterTemplate](c, value)
}

func (c FfiConverterBitcoinWalletFilterTemplate) LowerExternal(value BitcoinWalletFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BitcoinWalletFilterTemplate](c, value))
}

func (c FfiConverterBitcoinWalletFilterTemplate) Write(writer io.Writer, value BitcoinWalletFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Wallets)
}

type FfiDestroyerBitcoinWalletFilterTemplate struct{}

func (_ FfiDestroyerBitcoinWalletFilterTemplate) Destroy(value BitcoinWalletFilterTemplate) {
	value.Destroy()
}

// Summary data for a `bulk_add_tag` response.
type BulkAddTagData struct {
	// Total number of endpoints processed.
	Total int32
	// Number successfully tagged.
	UpdatedCount int32
	// Number that failed.
	FailedCount int32
	// Per-endpoint outcomes.
	Results []BulkOperationResult
	// The tag that was applied.
	Tag BulkTag
}

func (r *BulkAddTagData) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.Total)
	FfiDestroyerInt32{}.Destroy(r.UpdatedCount)
	FfiDestroyerInt32{}.Destroy(r.FailedCount)
	FfiDestroyerSequenceBulkOperationResult{}.Destroy(r.Results)
	FfiDestroyerBulkTag{}.Destroy(r.Tag)
}

type FfiConverterBulkAddTagData struct{}

var FfiConverterBulkAddTagDataINSTANCE = FfiConverterBulkAddTagData{}

func (c FfiConverterBulkAddTagData) Lift(rb RustBufferI) BulkAddTagData {
	return LiftFromRustBuffer[BulkAddTagData](c, rb)
}

func (c FfiConverterBulkAddTagData) Read(reader io.Reader) BulkAddTagData {
	return BulkAddTagData{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterSequenceBulkOperationResultINSTANCE.Read(reader),
		FfiConverterBulkTagINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkAddTagData) Lower(value BulkAddTagData) C.RustBuffer {
	return LowerIntoRustBuffer[BulkAddTagData](c, value)
}

func (c FfiConverterBulkAddTagData) LowerExternal(value BulkAddTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkAddTagData](c, value))
}

func (c FfiConverterBulkAddTagData) Write(writer io.Writer, value BulkAddTagData) {
	FfiConverterInt32INSTANCE.Write(writer, value.Total)
	FfiConverterInt32INSTANCE.Write(writer, value.UpdatedCount)
	FfiConverterInt32INSTANCE.Write(writer, value.FailedCount)
	FfiConverterSequenceBulkOperationResultINSTANCE.Write(writer, value.Results)
	FfiConverterBulkTagINSTANCE.Write(writer, value.Tag)
}

type FfiDestroyerBulkAddTagData struct{}

func (_ FfiDestroyerBulkAddTagData) Destroy(value BulkAddTagData) {
	value.Destroy()
}

// Parameters for `bulk_add_tag`.
type BulkAddTagRequest struct {
	// Endpoint ids to tag.
	Ids []string
	// Label of the tag to apply (created if it doesn't exist). Maximum 25 characters.
	Label string
}

func (r *BulkAddTagRequest) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Ids)
	FfiDestroyerString{}.Destroy(r.Label)
}

type FfiConverterBulkAddTagRequest struct{}

var FfiConverterBulkAddTagRequestINSTANCE = FfiConverterBulkAddTagRequest{}

func (c FfiConverterBulkAddTagRequest) Lift(rb RustBufferI) BulkAddTagRequest {
	return LiftFromRustBuffer[BulkAddTagRequest](c, rb)
}

func (c FfiConverterBulkAddTagRequest) Read(reader io.Reader) BulkAddTagRequest {
	return BulkAddTagRequest{
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkAddTagRequest) Lower(value BulkAddTagRequest) C.RustBuffer {
	return LowerIntoRustBuffer[BulkAddTagRequest](c, value)
}

func (c FfiConverterBulkAddTagRequest) LowerExternal(value BulkAddTagRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkAddTagRequest](c, value))
}

func (c FfiConverterBulkAddTagRequest) Write(writer io.Writer, value BulkAddTagRequest) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Ids)
	FfiConverterStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerBulkAddTagRequest struct{}

func (_ FfiDestroyerBulkAddTagRequest) Destroy(value BulkAddTagRequest) {
	value.Destroy()
}

// Response from `bulk_add_tag`.
type BulkAddTagResponse struct {
	// Bulk add-tag summary.
	Data *BulkAddTagData
	// Error message when the request did not succeed.
	Error *string
}

func (r *BulkAddTagResponse) Destroy() {
	FfiDestroyerOptionalBulkAddTagData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterBulkAddTagResponse struct{}

var FfiConverterBulkAddTagResponseINSTANCE = FfiConverterBulkAddTagResponse{}

func (c FfiConverterBulkAddTagResponse) Lift(rb RustBufferI) BulkAddTagResponse {
	return LiftFromRustBuffer[BulkAddTagResponse](c, rb)
}

func (c FfiConverterBulkAddTagResponse) Read(reader io.Reader) BulkAddTagResponse {
	return BulkAddTagResponse{
		FfiConverterOptionalBulkAddTagDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkAddTagResponse) Lower(value BulkAddTagResponse) C.RustBuffer {
	return LowerIntoRustBuffer[BulkAddTagResponse](c, value)
}

func (c FfiConverterBulkAddTagResponse) LowerExternal(value BulkAddTagResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkAddTagResponse](c, value))
}

func (c FfiConverterBulkAddTagResponse) Write(writer io.Writer, value BulkAddTagResponse) {
	FfiConverterOptionalBulkAddTagDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerBulkAddTagResponse struct{}

func (_ FfiDestroyerBulkAddTagResponse) Destroy(value BulkAddTagResponse) {
	value.Destroy()
}

// Per-endpoint result within a bulk response.
type BulkOperationResult struct {
	// Endpoint id the result refers to.
	Id string
	// Whether the operation succeeded for this endpoint.
	Success bool
}

func (r *BulkOperationResult) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerBool{}.Destroy(r.Success)
}

type FfiConverterBulkOperationResult struct{}

var FfiConverterBulkOperationResultINSTANCE = FfiConverterBulkOperationResult{}

func (c FfiConverterBulkOperationResult) Lift(rb RustBufferI) BulkOperationResult {
	return LiftFromRustBuffer[BulkOperationResult](c, rb)
}

func (c FfiConverterBulkOperationResult) Read(reader io.Reader) BulkOperationResult {
	return BulkOperationResult{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkOperationResult) Lower(value BulkOperationResult) C.RustBuffer {
	return LowerIntoRustBuffer[BulkOperationResult](c, value)
}

func (c FfiConverterBulkOperationResult) LowerExternal(value BulkOperationResult) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkOperationResult](c, value))
}

func (c FfiConverterBulkOperationResult) Write(writer io.Writer, value BulkOperationResult) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterBoolINSTANCE.Write(writer, value.Success)
}

type FfiDestroyerBulkOperationResult struct{}

func (_ FfiDestroyerBulkOperationResult) Destroy(value BulkOperationResult) {
	value.Destroy()
}

// Summary data for a `bulk_remove_tag` response.
type BulkRemoveTagData struct {
	// Total number of endpoints processed.
	Total int32
	// Number successfully updated.
	UpdatedCount int32
	// Number that failed.
	FailedCount int32
	// Per-endpoint outcomes.
	Results []BulkOperationResult
}

func (r *BulkRemoveTagData) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.Total)
	FfiDestroyerInt32{}.Destroy(r.UpdatedCount)
	FfiDestroyerInt32{}.Destroy(r.FailedCount)
	FfiDestroyerSequenceBulkOperationResult{}.Destroy(r.Results)
}

type FfiConverterBulkRemoveTagData struct{}

var FfiConverterBulkRemoveTagDataINSTANCE = FfiConverterBulkRemoveTagData{}

func (c FfiConverterBulkRemoveTagData) Lift(rb RustBufferI) BulkRemoveTagData {
	return LiftFromRustBuffer[BulkRemoveTagData](c, rb)
}

func (c FfiConverterBulkRemoveTagData) Read(reader io.Reader) BulkRemoveTagData {
	return BulkRemoveTagData{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterSequenceBulkOperationResultINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkRemoveTagData) Lower(value BulkRemoveTagData) C.RustBuffer {
	return LowerIntoRustBuffer[BulkRemoveTagData](c, value)
}

func (c FfiConverterBulkRemoveTagData) LowerExternal(value BulkRemoveTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkRemoveTagData](c, value))
}

func (c FfiConverterBulkRemoveTagData) Write(writer io.Writer, value BulkRemoveTagData) {
	FfiConverterInt32INSTANCE.Write(writer, value.Total)
	FfiConverterInt32INSTANCE.Write(writer, value.UpdatedCount)
	FfiConverterInt32INSTANCE.Write(writer, value.FailedCount)
	FfiConverterSequenceBulkOperationResultINSTANCE.Write(writer, value.Results)
}

type FfiDestroyerBulkRemoveTagData struct{}

func (_ FfiDestroyerBulkRemoveTagData) Destroy(value BulkRemoveTagData) {
	value.Destroy()
}

// Parameters for `bulk_remove_tag`.
type BulkRemoveTagRequest struct {
	// Endpoint ids to untag.
	Ids []string
	// Tag to remove.
	TagId int32
}

func (r *BulkRemoveTagRequest) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Ids)
	FfiDestroyerInt32{}.Destroy(r.TagId)
}

type FfiConverterBulkRemoveTagRequest struct{}

var FfiConverterBulkRemoveTagRequestINSTANCE = FfiConverterBulkRemoveTagRequest{}

func (c FfiConverterBulkRemoveTagRequest) Lift(rb RustBufferI) BulkRemoveTagRequest {
	return LiftFromRustBuffer[BulkRemoveTagRequest](c, rb)
}

func (c FfiConverterBulkRemoveTagRequest) Read(reader io.Reader) BulkRemoveTagRequest {
	return BulkRemoveTagRequest{
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkRemoveTagRequest) Lower(value BulkRemoveTagRequest) C.RustBuffer {
	return LowerIntoRustBuffer[BulkRemoveTagRequest](c, value)
}

func (c FfiConverterBulkRemoveTagRequest) LowerExternal(value BulkRemoveTagRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkRemoveTagRequest](c, value))
}

func (c FfiConverterBulkRemoveTagRequest) Write(writer io.Writer, value BulkRemoveTagRequest) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Ids)
	FfiConverterInt32INSTANCE.Write(writer, value.TagId)
}

type FfiDestroyerBulkRemoveTagRequest struct{}

func (_ FfiDestroyerBulkRemoveTagRequest) Destroy(value BulkRemoveTagRequest) {
	value.Destroy()
}

// Response from `bulk_remove_tag`.
type BulkRemoveTagResponse struct {
	// Bulk remove-tag summary.
	Data *BulkRemoveTagData
	// Error message when the request did not succeed.
	Error *string
}

func (r *BulkRemoveTagResponse) Destroy() {
	FfiDestroyerOptionalBulkRemoveTagData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterBulkRemoveTagResponse struct{}

var FfiConverterBulkRemoveTagResponseINSTANCE = FfiConverterBulkRemoveTagResponse{}

func (c FfiConverterBulkRemoveTagResponse) Lift(rb RustBufferI) BulkRemoveTagResponse {
	return LiftFromRustBuffer[BulkRemoveTagResponse](c, rb)
}

func (c FfiConverterBulkRemoveTagResponse) Read(reader io.Reader) BulkRemoveTagResponse {
	return BulkRemoveTagResponse{
		FfiConverterOptionalBulkRemoveTagDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkRemoveTagResponse) Lower(value BulkRemoveTagResponse) C.RustBuffer {
	return LowerIntoRustBuffer[BulkRemoveTagResponse](c, value)
}

func (c FfiConverterBulkRemoveTagResponse) LowerExternal(value BulkRemoveTagResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkRemoveTagResponse](c, value))
}

func (c FfiConverterBulkRemoveTagResponse) Write(writer io.Writer, value BulkRemoveTagResponse) {
	FfiConverterOptionalBulkRemoveTagDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerBulkRemoveTagResponse struct{}

func (_ FfiDestroyerBulkRemoveTagResponse) Destroy(value BulkRemoveTagResponse) {
	value.Destroy()
}

// Parameters for `bulk_sets`. Either or both fields may be supplied.
type BulkSetsParams struct {
	// Key/value pairs to add.
	AddSets *map[string]string
	// Keys to delete.
	DeleteSets *[]string
}

func (r *BulkSetsParams) Destroy() {
	FfiDestroyerOptionalMapStringString{}.Destroy(r.AddSets)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.DeleteSets)
}

type FfiConverterBulkSetsParams struct{}

var FfiConverterBulkSetsParamsINSTANCE = FfiConverterBulkSetsParams{}

func (c FfiConverterBulkSetsParams) Lift(rb RustBufferI) BulkSetsParams {
	return LiftFromRustBuffer[BulkSetsParams](c, rb)
}

func (c FfiConverterBulkSetsParams) Read(reader io.Reader) BulkSetsParams {
	return BulkSetsParams{
		FfiConverterOptionalMapStringStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkSetsParams) Lower(value BulkSetsParams) C.RustBuffer {
	return LowerIntoRustBuffer[BulkSetsParams](c, value)
}

func (c FfiConverterBulkSetsParams) LowerExternal(value BulkSetsParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkSetsParams](c, value))
}

func (c FfiConverterBulkSetsParams) Write(writer io.Writer, value BulkSetsParams) {
	FfiConverterOptionalMapStringStringINSTANCE.Write(writer, value.AddSets)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.DeleteSets)
}

type FfiDestroyerBulkSetsParams struct{}

func (_ FfiDestroyerBulkSetsParams) Destroy(value BulkSetsParams) {
	value.Destroy()
}

// Tag reference returned on bulk tag operations.
type BulkTag struct {
	// Tag identifier.
	TagId int32
	// Tag label.
	Label string
}

func (r *BulkTag) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.TagId)
	FfiDestroyerString{}.Destroy(r.Label)
}

type FfiConverterBulkTag struct{}

var FfiConverterBulkTagINSTANCE = FfiConverterBulkTag{}

func (c FfiConverterBulkTag) Lift(rb RustBufferI) BulkTag {
	return LiftFromRustBuffer[BulkTag](c, rb)
}

func (c FfiConverterBulkTag) Read(reader io.Reader) BulkTag {
	return BulkTag{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkTag) Lower(value BulkTag) C.RustBuffer {
	return LowerIntoRustBuffer[BulkTag](c, value)
}

func (c FfiConverterBulkTag) LowerExternal(value BulkTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkTag](c, value))
}

func (c FfiConverterBulkTag) Write(writer io.Writer, value BulkTag) {
	FfiConverterInt32INSTANCE.Write(writer, value.TagId)
	FfiConverterStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerBulkTag struct{}

func (_ FfiDestroyerBulkTag) Destroy(value BulkTag) {
	value.Destroy()
}

// Summary data for a `bulk_update_endpoint_status` response.
type BulkUpdateEndpointStatusData struct {
	// Total number of endpoints processed.
	Total int32
	// Number successfully updated.
	UpdatedCount int32
	// Number that failed.
	FailedCount int32
	// Per-endpoint outcomes.
	Results []BulkOperationResult
}

func (r *BulkUpdateEndpointStatusData) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.Total)
	FfiDestroyerInt32{}.Destroy(r.UpdatedCount)
	FfiDestroyerInt32{}.Destroy(r.FailedCount)
	FfiDestroyerSequenceBulkOperationResult{}.Destroy(r.Results)
}

type FfiConverterBulkUpdateEndpointStatusData struct{}

var FfiConverterBulkUpdateEndpointStatusDataINSTANCE = FfiConverterBulkUpdateEndpointStatusData{}

func (c FfiConverterBulkUpdateEndpointStatusData) Lift(rb RustBufferI) BulkUpdateEndpointStatusData {
	return LiftFromRustBuffer[BulkUpdateEndpointStatusData](c, rb)
}

func (c FfiConverterBulkUpdateEndpointStatusData) Read(reader io.Reader) BulkUpdateEndpointStatusData {
	return BulkUpdateEndpointStatusData{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterSequenceBulkOperationResultINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkUpdateEndpointStatusData) Lower(value BulkUpdateEndpointStatusData) C.RustBuffer {
	return LowerIntoRustBuffer[BulkUpdateEndpointStatusData](c, value)
}

func (c FfiConverterBulkUpdateEndpointStatusData) LowerExternal(value BulkUpdateEndpointStatusData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkUpdateEndpointStatusData](c, value))
}

func (c FfiConverterBulkUpdateEndpointStatusData) Write(writer io.Writer, value BulkUpdateEndpointStatusData) {
	FfiConverterInt32INSTANCE.Write(writer, value.Total)
	FfiConverterInt32INSTANCE.Write(writer, value.UpdatedCount)
	FfiConverterInt32INSTANCE.Write(writer, value.FailedCount)
	FfiConverterSequenceBulkOperationResultINSTANCE.Write(writer, value.Results)
}

type FfiDestroyerBulkUpdateEndpointStatusData struct{}

func (_ FfiDestroyerBulkUpdateEndpointStatusData) Destroy(value BulkUpdateEndpointStatusData) {
	value.Destroy()
}

// Parameters for `bulk_update_endpoint_status`.
type BulkUpdateEndpointStatusRequest struct {
	// Endpoint ids to update.
	Ids []string
	// Target status (`active` or `paused`).
	Status string
}

func (r *BulkUpdateEndpointStatusRequest) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Ids)
	FfiDestroyerString{}.Destroy(r.Status)
}

type FfiConverterBulkUpdateEndpointStatusRequest struct{}

var FfiConverterBulkUpdateEndpointStatusRequestINSTANCE = FfiConverterBulkUpdateEndpointStatusRequest{}

func (c FfiConverterBulkUpdateEndpointStatusRequest) Lift(rb RustBufferI) BulkUpdateEndpointStatusRequest {
	return LiftFromRustBuffer[BulkUpdateEndpointStatusRequest](c, rb)
}

func (c FfiConverterBulkUpdateEndpointStatusRequest) Read(reader io.Reader) BulkUpdateEndpointStatusRequest {
	return BulkUpdateEndpointStatusRequest{
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkUpdateEndpointStatusRequest) Lower(value BulkUpdateEndpointStatusRequest) C.RustBuffer {
	return LowerIntoRustBuffer[BulkUpdateEndpointStatusRequest](c, value)
}

func (c FfiConverterBulkUpdateEndpointStatusRequest) LowerExternal(value BulkUpdateEndpointStatusRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkUpdateEndpointStatusRequest](c, value))
}

func (c FfiConverterBulkUpdateEndpointStatusRequest) Write(writer io.Writer, value BulkUpdateEndpointStatusRequest) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Ids)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
}

type FfiDestroyerBulkUpdateEndpointStatusRequest struct{}

func (_ FfiDestroyerBulkUpdateEndpointStatusRequest) Destroy(value BulkUpdateEndpointStatusRequest) {
	value.Destroy()
}

// Response from `bulk_update_endpoint_status`.
type BulkUpdateEndpointStatusResponse struct {
	// Bulk update summary.
	Data *BulkUpdateEndpointStatusData
	// Error message when the request did not succeed.
	Error *string
}

func (r *BulkUpdateEndpointStatusResponse) Destroy() {
	FfiDestroyerOptionalBulkUpdateEndpointStatusData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterBulkUpdateEndpointStatusResponse struct{}

var FfiConverterBulkUpdateEndpointStatusResponseINSTANCE = FfiConverterBulkUpdateEndpointStatusResponse{}

func (c FfiConverterBulkUpdateEndpointStatusResponse) Lift(rb RustBufferI) BulkUpdateEndpointStatusResponse {
	return LiftFromRustBuffer[BulkUpdateEndpointStatusResponse](c, rb)
}

func (c FfiConverterBulkUpdateEndpointStatusResponse) Read(reader io.Reader) BulkUpdateEndpointStatusResponse {
	return BulkUpdateEndpointStatusResponse{
		FfiConverterOptionalBulkUpdateEndpointStatusDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBulkUpdateEndpointStatusResponse) Lower(value BulkUpdateEndpointStatusResponse) C.RustBuffer {
	return LowerIntoRustBuffer[BulkUpdateEndpointStatusResponse](c, value)
}

func (c FfiConverterBulkUpdateEndpointStatusResponse) LowerExternal(value BulkUpdateEndpointStatusResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BulkUpdateEndpointStatusResponse](c, value))
}

func (c FfiConverterBulkUpdateEndpointStatusResponse) Write(writer io.Writer, value BulkUpdateEndpointStatusResponse) {
	FfiConverterOptionalBulkUpdateEndpointStatusDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerBulkUpdateEndpointStatusResponse struct{}

func (_ FfiDestroyerBulkUpdateEndpointStatusResponse) Destroy(value BulkUpdateEndpointStatusResponse) {
	value.Destroy()
}

// A blockchain supported by Quicknode along with its networks.
type Chain struct {
	// Chain slug (e.g. `ethereum`).
	Slug string
	// Networks available on this chain.
	Networks []ChainNetwork
	// Whether the chain is shown in selection UIs.
	IsSelectChain *bool
}

func (r *Chain) Destroy() {
	FfiDestroyerString{}.Destroy(r.Slug)
	FfiDestroyerSequenceChainNetwork{}.Destroy(r.Networks)
	FfiDestroyerOptionalBool{}.Destroy(r.IsSelectChain)
}

type FfiConverterChain struct{}

var FfiConverterChainINSTANCE = FfiConverterChain{}

func (c FfiConverterChain) Lift(rb RustBufferI) Chain {
	return LiftFromRustBuffer[Chain](c, rb)
}

func (c FfiConverterChain) Read(reader io.Reader) Chain {
	return Chain{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceChainNetworkINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterChain) Lower(value Chain) C.RustBuffer {
	return LowerIntoRustBuffer[Chain](c, value)
}

func (c FfiConverterChain) LowerExternal(value Chain) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Chain](c, value))
}

func (c FfiConverterChain) Write(writer io.Writer, value Chain) {
	FfiConverterStringINSTANCE.Write(writer, value.Slug)
	FfiConverterSequenceChainNetworkINSTANCE.Write(writer, value.Networks)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.IsSelectChain)
}

type FfiDestroyerChain struct{}

func (_ FfiDestroyerChain) Destroy(value Chain) {
	value.Destroy()
}

// A network within a supported chain.
type ChainNetwork struct {
	// Network slug (e.g. `mainnet`).
	Slug string
	// Human-readable network name.
	Name string
	// Numeric chain id, when applicable.
	ChainId *int64
}

func (r *ChainNetwork) Destroy() {
	FfiDestroyerString{}.Destroy(r.Slug)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalInt64{}.Destroy(r.ChainId)
}

type FfiConverterChainNetwork struct{}

var FfiConverterChainNetworkINSTANCE = FfiConverterChainNetwork{}

func (c FfiConverterChainNetwork) Lift(rb RustBufferI) ChainNetwork {
	return LiftFromRustBuffer[ChainNetwork](c, rb)
}

func (c FfiConverterChainNetwork) Read(reader io.Reader) ChainNetwork {
	return ChainNetwork{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterChainNetwork) Lower(value ChainNetwork) C.RustBuffer {
	return LowerIntoRustBuffer[ChainNetwork](c, value)
}

func (c FfiConverterChainNetwork) LowerExternal(value ChainNetwork) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ChainNetwork](c, value))
}

func (c FfiConverterChainNetwork) Write(writer io.Writer, value ChainNetwork) {
	FfiConverterStringINSTANCE.Write(writer, value.Slug)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.ChainId)
}

type FfiDestroyerChainNetwork struct{}

func (_ FfiDestroyerChainNetwork) Destroy(value ChainNetwork) {
	value.Destroy()
}

// Response from `get_schema`: the schema for a single chain/cluster.
type ChainSchema struct {
	// Human-readable chain name (e.g. `"Hyperliquid (HyperCore)"`).
	Chain string
	// Cluster identifier the schema belongs to.
	ClusterId string
	// Tables available in this cluster.
	Tables []TableSchema
}

func (r *ChainSchema) Destroy() {
	FfiDestroyerString{}.Destroy(r.Chain)
	FfiDestroyerString{}.Destroy(r.ClusterId)
	FfiDestroyerSequenceTableSchema{}.Destroy(r.Tables)
}

type FfiConverterChainSchema struct{}

var FfiConverterChainSchemaINSTANCE = FfiConverterChainSchema{}

func (c FfiConverterChainSchema) Lift(rb RustBufferI) ChainSchema {
	return LiftFromRustBuffer[ChainSchema](c, rb)
}

func (c FfiConverterChainSchema) Read(reader io.Reader) ChainSchema {
	return ChainSchema{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceTableSchemaINSTANCE.Read(reader),
	}
}

func (c FfiConverterChainSchema) Lower(value ChainSchema) C.RustBuffer {
	return LowerIntoRustBuffer[ChainSchema](c, value)
}

func (c FfiConverterChainSchema) LowerExternal(value ChainSchema) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ChainSchema](c, value))
}

func (c FfiConverterChainSchema) Write(writer io.Writer, value ChainSchema) {
	FfiConverterStringINSTANCE.Write(writer, value.Chain)
	FfiConverterStringINSTANCE.Write(writer, value.ClusterId)
	FfiConverterSequenceTableSchemaINSTANCE.Write(writer, value.Tables)
}

type FfiDestroyerChainSchema struct{}

func (_ FfiDestroyerChainSchema) Destroy(value ChainSchema) {
	value.Destroy()
}

// Per-chain usage row.
type ChainUsage struct {
	// Chain name or slug.
	Name string
	// Credits consumed on the chain.
	CreditsUsed int64
}

func (r *ChainUsage) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerInt64{}.Destroy(r.CreditsUsed)
}

type FfiConverterChainUsage struct{}

var FfiConverterChainUsageINSTANCE = FfiConverterChainUsage{}

func (c FfiConverterChainUsage) Lift(rb RustBufferI) ChainUsage {
	return LiftFromRustBuffer[ChainUsage](c, rb)
}

func (c FfiConverterChainUsage) Read(reader io.Reader) ChainUsage {
	return ChainUsage{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterChainUsage) Lower(value ChainUsage) C.RustBuffer {
	return LowerIntoRustBuffer[ChainUsage](c, value)
}

func (c FfiConverterChainUsage) LowerExternal(value ChainUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ChainUsage](c, value))
}

func (c FfiConverterChainUsage) Write(writer io.Writer, value ChainUsage) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterInt64INSTANCE.Write(writer, value.CreditsUsed)
}

type FfiDestroyerChainUsage struct{}

func (_ FfiDestroyerChainUsage) Destroy(value ChainUsage) {
	value.Destroy()
}

// Metadata describing a single column in a query result set.
type ColumnMeta struct {
	// Column name as it appears in the result set.
	Name string
	// Column data type (e.g. `"DateTime('UTC')"`, `"LowCardinality(String)"`).
	ColumnType string
}

func (r *ColumnMeta) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.ColumnType)
}

type FfiConverterColumnMeta struct{}

var FfiConverterColumnMetaINSTANCE = FfiConverterColumnMeta{}

func (c FfiConverterColumnMeta) Lift(rb RustBufferI) ColumnMeta {
	return LiftFromRustBuffer[ColumnMeta](c, rb)
}

func (c FfiConverterColumnMeta) Read(reader io.Reader) ColumnMeta {
	return ColumnMeta{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterColumnMeta) Lower(value ColumnMeta) C.RustBuffer {
	return LowerIntoRustBuffer[ColumnMeta](c, value)
}

func (c FfiConverterColumnMeta) LowerExternal(value ColumnMeta) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ColumnMeta](c, value))
}

func (c FfiConverterColumnMeta) Write(writer io.Writer, value ColumnMeta) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.ColumnType)
}

type FfiDestroyerColumnMeta struct{}

func (_ FfiDestroyerColumnMeta) Destroy(value ColumnMeta) {
	value.Destroy()
}

// A single column in a table schema.
type ColumnSchema struct {
	// Column name.
	Name string
	// Column data type (e.g. `"UInt64"`, `"FixedString(42)"`).
	ColumnType string
}

func (r *ColumnSchema) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.ColumnType)
}

type FfiConverterColumnSchema struct{}

var FfiConverterColumnSchemaINSTANCE = FfiConverterColumnSchema{}

func (c FfiConverterColumnSchema) Lift(rb RustBufferI) ColumnSchema {
	return LiftFromRustBuffer[ColumnSchema](c, rb)
}

func (c FfiConverterColumnSchema) Read(reader io.Reader) ColumnSchema {
	return ColumnSchema{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterColumnSchema) Lower(value ColumnSchema) C.RustBuffer {
	return LowerIntoRustBuffer[ColumnSchema](c, value)
}

func (c FfiConverterColumnSchema) LowerExternal(value ColumnSchema) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ColumnSchema](c, value))
}

func (c FfiConverterColumnSchema) Write(writer io.Writer, value ColumnSchema) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.ColumnType)
}

type FfiDestroyerColumnSchema struct{}

func (_ FfiDestroyerColumnSchema) Destroy(value ColumnSchema) {
	value.Destroy()
}

// Parameters for `create_domain_mask`.
type CreateDomainMaskRequest struct {
	// Custom domain that will mask the endpoint's Quicknode URL.
	DomainMask *string
}

func (r *CreateDomainMaskRequest) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.DomainMask)
}

type FfiConverterCreateDomainMaskRequest struct{}

var FfiConverterCreateDomainMaskRequestINSTANCE = FfiConverterCreateDomainMaskRequest{}

func (c FfiConverterCreateDomainMaskRequest) Lift(rb RustBufferI) CreateDomainMaskRequest {
	return LiftFromRustBuffer[CreateDomainMaskRequest](c, rb)
}

func (c FfiConverterCreateDomainMaskRequest) Read(reader io.Reader) CreateDomainMaskRequest {
	return CreateDomainMaskRequest{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateDomainMaskRequest) Lower(value CreateDomainMaskRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateDomainMaskRequest](c, value)
}

func (c FfiConverterCreateDomainMaskRequest) LowerExternal(value CreateDomainMaskRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateDomainMaskRequest](c, value))
}

func (c FfiConverterCreateDomainMaskRequest) Write(writer io.Writer, value CreateDomainMaskRequest) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.DomainMask)
}

type FfiDestroyerCreateDomainMaskRequest struct{}

func (_ FfiDestroyerCreateDomainMaskRequest) Destroy(value CreateDomainMaskRequest) {
	value.Destroy()
}

// Parameters for `create_endpoint`.
type CreateEndpointRequest struct {
	// Blockchain the endpoint should serve (e.g. `ethereum`).
	Chain *string
	// Specific network within the chain (e.g. `mainnet`).
	Network *string
}

func (r *CreateEndpointRequest) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Chain)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
}

type FfiConverterCreateEndpointRequest struct{}

var FfiConverterCreateEndpointRequestINSTANCE = FfiConverterCreateEndpointRequest{}

func (c FfiConverterCreateEndpointRequest) Lift(rb RustBufferI) CreateEndpointRequest {
	return LiftFromRustBuffer[CreateEndpointRequest](c, rb)
}

func (c FfiConverterCreateEndpointRequest) Read(reader io.Reader) CreateEndpointRequest {
	return CreateEndpointRequest{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateEndpointRequest) Lower(value CreateEndpointRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateEndpointRequest](c, value)
}

func (c FfiConverterCreateEndpointRequest) LowerExternal(value CreateEndpointRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateEndpointRequest](c, value))
}

func (c FfiConverterCreateEndpointRequest) Write(writer io.Writer, value CreateEndpointRequest) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Chain)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
}

type FfiDestroyerCreateEndpointRequest struct{}

func (_ FfiDestroyerCreateEndpointRequest) Destroy(value CreateEndpointRequest) {
	value.Destroy()
}

// Response from `create_endpoint`.
type CreateEndpointResponse struct {
	// The newly created endpoint.
	Data SingleEndpoint
	// Error message when the request did not succeed.
	Error *string
}

func (r *CreateEndpointResponse) Destroy() {
	FfiDestroyerSingleEndpoint{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterCreateEndpointResponse struct{}

var FfiConverterCreateEndpointResponseINSTANCE = FfiConverterCreateEndpointResponse{}

func (c FfiConverterCreateEndpointResponse) Lift(rb RustBufferI) CreateEndpointResponse {
	return LiftFromRustBuffer[CreateEndpointResponse](c, rb)
}

func (c FfiConverterCreateEndpointResponse) Read(reader io.Reader) CreateEndpointResponse {
	return CreateEndpointResponse{
		FfiConverterSingleEndpointINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateEndpointResponse) Lower(value CreateEndpointResponse) C.RustBuffer {
	return LowerIntoRustBuffer[CreateEndpointResponse](c, value)
}

func (c FfiConverterCreateEndpointResponse) LowerExternal(value CreateEndpointResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateEndpointResponse](c, value))
}

func (c FfiConverterCreateEndpointResponse) Write(writer io.Writer, value CreateEndpointResponse) {
	FfiConverterSingleEndpointINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerCreateEndpointResponse struct{}

func (_ FfiDestroyerCreateEndpointResponse) Destroy(value CreateEndpointResponse) {
	value.Destroy()
}

// Parameters for `create_ip`.
type CreateIpRequest struct {
	// IP address to whitelist.
	Ip string
}

func (r *CreateIpRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Ip)
}

type FfiConverterCreateIpRequest struct{}

var FfiConverterCreateIpRequestINSTANCE = FfiConverterCreateIpRequest{}

func (c FfiConverterCreateIpRequest) Lift(rb RustBufferI) CreateIpRequest {
	return LiftFromRustBuffer[CreateIpRequest](c, rb)
}

func (c FfiConverterCreateIpRequest) Read(reader io.Reader) CreateIpRequest {
	return CreateIpRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateIpRequest) Lower(value CreateIpRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateIpRequest](c, value)
}

func (c FfiConverterCreateIpRequest) LowerExternal(value CreateIpRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateIpRequest](c, value))
}

func (c FfiConverterCreateIpRequest) Write(writer io.Writer, value CreateIpRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Ip)
}

type FfiDestroyerCreateIpRequest struct{}

func (_ FfiDestroyerCreateIpRequest) Destroy(value CreateIpRequest) {
	value.Destroy()
}

// Parameters for `create_jwt`.
type CreateJwtRequest struct {
	// Public key used to verify signed JWTs.
	PublicKey string
	// Key identifier (`kid`) embedded in JWT headers.
	Kid string
	// Human-readable name for the JWT configuration.
	Name string
}

func (r *CreateJwtRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.PublicKey)
	FfiDestroyerString{}.Destroy(r.Kid)
	FfiDestroyerString{}.Destroy(r.Name)
}

type FfiConverterCreateJwtRequest struct{}

var FfiConverterCreateJwtRequestINSTANCE = FfiConverterCreateJwtRequest{}

func (c FfiConverterCreateJwtRequest) Lift(rb RustBufferI) CreateJwtRequest {
	return LiftFromRustBuffer[CreateJwtRequest](c, rb)
}

func (c FfiConverterCreateJwtRequest) Read(reader io.Reader) CreateJwtRequest {
	return CreateJwtRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateJwtRequest) Lower(value CreateJwtRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateJwtRequest](c, value)
}

func (c FfiConverterCreateJwtRequest) LowerExternal(value CreateJwtRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateJwtRequest](c, value))
}

func (c FfiConverterCreateJwtRequest) Write(writer io.Writer, value CreateJwtRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.PublicKey)
	FfiConverterStringINSTANCE.Write(writer, value.Kid)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
}

type FfiDestroyerCreateJwtRequest struct{}

func (_ FfiDestroyerCreateJwtRequest) Destroy(value CreateJwtRequest) {
	value.Destroy()
}

// Parameters for `create_list`.
type CreateListParams struct {
	// Unique key identifying the list.
	Key string
	// Initial items inserted into the list.
	Items []string
}

func (r *CreateListParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Key)
	FfiDestroyerSequenceString{}.Destroy(r.Items)
}

type FfiConverterCreateListParams struct{}

var FfiConverterCreateListParamsINSTANCE = FfiConverterCreateListParams{}

func (c FfiConverterCreateListParams) Lift(rb RustBufferI) CreateListParams {
	return LiftFromRustBuffer[CreateListParams](c, rb)
}

func (c FfiConverterCreateListParams) Read(reader io.Reader) CreateListParams {
	return CreateListParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateListParams) Lower(value CreateListParams) C.RustBuffer {
	return LowerIntoRustBuffer[CreateListParams](c, value)
}

func (c FfiConverterCreateListParams) LowerExternal(value CreateListParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateListParams](c, value))
}

func (c FfiConverterCreateListParams) Write(writer io.Writer, value CreateListParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Key)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Items)
}

type FfiDestroyerCreateListParams struct{}

func (_ FfiDestroyerCreateListParams) Destroy(value CreateListParams) {
	value.Destroy()
}

// Parameters for `create_method_rate_limit`.
type CreateMethodRateLimitRequest struct {
	// Interval over which the rate applies (e.g. `second`).
	Interval string
	// RPC methods the limiter applies to.
	Methods []string
	// Maximum number of calls allowed per interval.
	Rate int32
}

func (r *CreateMethodRateLimitRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Interval)
	FfiDestroyerSequenceString{}.Destroy(r.Methods)
	FfiDestroyerInt32{}.Destroy(r.Rate)
}

type FfiConverterCreateMethodRateLimitRequest struct{}

var FfiConverterCreateMethodRateLimitRequestINSTANCE = FfiConverterCreateMethodRateLimitRequest{}

func (c FfiConverterCreateMethodRateLimitRequest) Lift(rb RustBufferI) CreateMethodRateLimitRequest {
	return LiftFromRustBuffer[CreateMethodRateLimitRequest](c, rb)
}

func (c FfiConverterCreateMethodRateLimitRequest) Read(reader io.Reader) CreateMethodRateLimitRequest {
	return CreateMethodRateLimitRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateMethodRateLimitRequest) Lower(value CreateMethodRateLimitRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateMethodRateLimitRequest](c, value)
}

func (c FfiConverterCreateMethodRateLimitRequest) LowerExternal(value CreateMethodRateLimitRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateMethodRateLimitRequest](c, value))
}

func (c FfiConverterCreateMethodRateLimitRequest) Write(writer io.Writer, value CreateMethodRateLimitRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Interval)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Methods)
	FfiConverterInt32INSTANCE.Write(writer, value.Rate)
}

type FfiDestroyerCreateMethodRateLimitRequest struct{}

func (_ FfiDestroyerCreateMethodRateLimitRequest) Destroy(value CreateMethodRateLimitRequest) {
	value.Destroy()
}

// Response from `create_method_rate_limit`.
type CreateMethodRateLimitResponse struct {
	// The created rate limiter.
	Data *MethodRateLimiter
	// Error message when the request did not succeed.
	Error *string
}

func (r *CreateMethodRateLimitResponse) Destroy() {
	FfiDestroyerOptionalMethodRateLimiter{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterCreateMethodRateLimitResponse struct{}

var FfiConverterCreateMethodRateLimitResponseINSTANCE = FfiConverterCreateMethodRateLimitResponse{}

func (c FfiConverterCreateMethodRateLimitResponse) Lift(rb RustBufferI) CreateMethodRateLimitResponse {
	return LiftFromRustBuffer[CreateMethodRateLimitResponse](c, rb)
}

func (c FfiConverterCreateMethodRateLimitResponse) Read(reader io.Reader) CreateMethodRateLimitResponse {
	return CreateMethodRateLimitResponse{
		FfiConverterOptionalMethodRateLimiterINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateMethodRateLimitResponse) Lower(value CreateMethodRateLimitResponse) C.RustBuffer {
	return LowerIntoRustBuffer[CreateMethodRateLimitResponse](c, value)
}

func (c FfiConverterCreateMethodRateLimitResponse) LowerExternal(value CreateMethodRateLimitResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateMethodRateLimitResponse](c, value))
}

func (c FfiConverterCreateMethodRateLimitResponse) Write(writer io.Writer, value CreateMethodRateLimitResponse) {
	FfiConverterOptionalMethodRateLimiterINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerCreateMethodRateLimitResponse struct{}

func (_ FfiDestroyerCreateMethodRateLimitResponse) Destroy(value CreateMethodRateLimitResponse) {
	value.Destroy()
}

// Parameters for `create_or_update_ip_custom_header`.
type CreateOrUpdateIpCustomHeaderRequest struct {
	// Header name used to identify the client IP (e.g. `X-Forwarded-For`).
	HeaderName string
}

func (r *CreateOrUpdateIpCustomHeaderRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.HeaderName)
}

type FfiConverterCreateOrUpdateIpCustomHeaderRequest struct{}

var FfiConverterCreateOrUpdateIpCustomHeaderRequestINSTANCE = FfiConverterCreateOrUpdateIpCustomHeaderRequest{}

func (c FfiConverterCreateOrUpdateIpCustomHeaderRequest) Lift(rb RustBufferI) CreateOrUpdateIpCustomHeaderRequest {
	return LiftFromRustBuffer[CreateOrUpdateIpCustomHeaderRequest](c, rb)
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderRequest) Read(reader io.Reader) CreateOrUpdateIpCustomHeaderRequest {
	return CreateOrUpdateIpCustomHeaderRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderRequest) Lower(value CreateOrUpdateIpCustomHeaderRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateOrUpdateIpCustomHeaderRequest](c, value)
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderRequest) LowerExternal(value CreateOrUpdateIpCustomHeaderRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateOrUpdateIpCustomHeaderRequest](c, value))
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderRequest) Write(writer io.Writer, value CreateOrUpdateIpCustomHeaderRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.HeaderName)
}

type FfiDestroyerCreateOrUpdateIpCustomHeaderRequest struct{}

func (_ FfiDestroyerCreateOrUpdateIpCustomHeaderRequest) Destroy(value CreateOrUpdateIpCustomHeaderRequest) {
	value.Destroy()
}

// Response from `create_or_update_ip_custom_header`.
type CreateOrUpdateIpCustomHeaderResponse struct {
	// Stored header configuration.
	Data *IpCustomHeaderData
	// Error message when the request did not succeed.
	Error *string
}

func (r *CreateOrUpdateIpCustomHeaderResponse) Destroy() {
	FfiDestroyerOptionalIpCustomHeaderData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterCreateOrUpdateIpCustomHeaderResponse struct{}

var FfiConverterCreateOrUpdateIpCustomHeaderResponseINSTANCE = FfiConverterCreateOrUpdateIpCustomHeaderResponse{}

func (c FfiConverterCreateOrUpdateIpCustomHeaderResponse) Lift(rb RustBufferI) CreateOrUpdateIpCustomHeaderResponse {
	return LiftFromRustBuffer[CreateOrUpdateIpCustomHeaderResponse](c, rb)
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderResponse) Read(reader io.Reader) CreateOrUpdateIpCustomHeaderResponse {
	return CreateOrUpdateIpCustomHeaderResponse{
		FfiConverterOptionalIpCustomHeaderDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderResponse) Lower(value CreateOrUpdateIpCustomHeaderResponse) C.RustBuffer {
	return LowerIntoRustBuffer[CreateOrUpdateIpCustomHeaderResponse](c, value)
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderResponse) LowerExternal(value CreateOrUpdateIpCustomHeaderResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateOrUpdateIpCustomHeaderResponse](c, value))
}

func (c FfiConverterCreateOrUpdateIpCustomHeaderResponse) Write(writer io.Writer, value CreateOrUpdateIpCustomHeaderResponse) {
	FfiConverterOptionalIpCustomHeaderDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerCreateOrUpdateIpCustomHeaderResponse struct{}

func (_ FfiDestroyerCreateOrUpdateIpCustomHeaderResponse) Destroy(value CreateOrUpdateIpCustomHeaderResponse) {
	value.Destroy()
}

// Parameters for `create_referrer`.
type CreateReferrerRequest struct {
	// Allowed referrer URL or domain.
	Referrer string
}

func (r *CreateReferrerRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Referrer)
}

type FfiConverterCreateReferrerRequest struct{}

var FfiConverterCreateReferrerRequestINSTANCE = FfiConverterCreateReferrerRequest{}

func (c FfiConverterCreateReferrerRequest) Lift(rb RustBufferI) CreateReferrerRequest {
	return LiftFromRustBuffer[CreateReferrerRequest](c, rb)
}

func (c FfiConverterCreateReferrerRequest) Read(reader io.Reader) CreateReferrerRequest {
	return CreateReferrerRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateReferrerRequest) Lower(value CreateReferrerRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateReferrerRequest](c, value)
}

func (c FfiConverterCreateReferrerRequest) LowerExternal(value CreateReferrerRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateReferrerRequest](c, value))
}

func (c FfiConverterCreateReferrerRequest) Write(writer io.Writer, value CreateReferrerRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Referrer)
}

type FfiDestroyerCreateReferrerRequest struct{}

func (_ FfiDestroyerCreateReferrerRequest) Destroy(value CreateReferrerRequest) {
	value.Destroy()
}

// Data wrapper for a created request filter.
type CreateRequestFilterData struct {
	// Identifier of the newly created request filter.
	Id string
}

func (r *CreateRequestFilterData) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
}

type FfiConverterCreateRequestFilterData struct{}

var FfiConverterCreateRequestFilterDataINSTANCE = FfiConverterCreateRequestFilterData{}

func (c FfiConverterCreateRequestFilterData) Lift(rb RustBufferI) CreateRequestFilterData {
	return LiftFromRustBuffer[CreateRequestFilterData](c, rb)
}

func (c FfiConverterCreateRequestFilterData) Read(reader io.Reader) CreateRequestFilterData {
	return CreateRequestFilterData{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateRequestFilterData) Lower(value CreateRequestFilterData) C.RustBuffer {
	return LowerIntoRustBuffer[CreateRequestFilterData](c, value)
}

func (c FfiConverterCreateRequestFilterData) LowerExternal(value CreateRequestFilterData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateRequestFilterData](c, value))
}

func (c FfiConverterCreateRequestFilterData) Write(writer io.Writer, value CreateRequestFilterData) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
}

type FfiDestroyerCreateRequestFilterData struct{}

func (_ FfiDestroyerCreateRequestFilterData) Destroy(value CreateRequestFilterData) {
	value.Destroy()
}

// Parameters for `create_request_filter`.
type CreateRequestFilterRequest struct {
	// Whitelisted RPC methods; other methods will be blocked.
	Method []string
}

func (r *CreateRequestFilterRequest) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Method)
}

type FfiConverterCreateRequestFilterRequest struct{}

var FfiConverterCreateRequestFilterRequestINSTANCE = FfiConverterCreateRequestFilterRequest{}

func (c FfiConverterCreateRequestFilterRequest) Lift(rb RustBufferI) CreateRequestFilterRequest {
	return LiftFromRustBuffer[CreateRequestFilterRequest](c, rb)
}

func (c FfiConverterCreateRequestFilterRequest) Read(reader io.Reader) CreateRequestFilterRequest {
	return CreateRequestFilterRequest{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateRequestFilterRequest) Lower(value CreateRequestFilterRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateRequestFilterRequest](c, value)
}

func (c FfiConverterCreateRequestFilterRequest) LowerExternal(value CreateRequestFilterRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateRequestFilterRequest](c, value))
}

func (c FfiConverterCreateRequestFilterRequest) Write(writer io.Writer, value CreateRequestFilterRequest) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Method)
}

type FfiDestroyerCreateRequestFilterRequest struct{}

func (_ FfiDestroyerCreateRequestFilterRequest) Destroy(value CreateRequestFilterRequest) {
	value.Destroy()
}

// Response from `create_request_filter`.
type CreateRequestFilterResponse struct {
	// The created filter payload.
	Data *CreateRequestFilterData
	// Error message when the request did not succeed.
	Error *string
}

func (r *CreateRequestFilterResponse) Destroy() {
	FfiDestroyerOptionalCreateRequestFilterData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterCreateRequestFilterResponse struct{}

var FfiConverterCreateRequestFilterResponseINSTANCE = FfiConverterCreateRequestFilterResponse{}

func (c FfiConverterCreateRequestFilterResponse) Lift(rb RustBufferI) CreateRequestFilterResponse {
	return LiftFromRustBuffer[CreateRequestFilterResponse](c, rb)
}

func (c FfiConverterCreateRequestFilterResponse) Read(reader io.Reader) CreateRequestFilterResponse {
	return CreateRequestFilterResponse{
		FfiConverterOptionalCreateRequestFilterDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateRequestFilterResponse) Lower(value CreateRequestFilterResponse) C.RustBuffer {
	return LowerIntoRustBuffer[CreateRequestFilterResponse](c, value)
}

func (c FfiConverterCreateRequestFilterResponse) LowerExternal(value CreateRequestFilterResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateRequestFilterResponse](c, value))
}

func (c FfiConverterCreateRequestFilterResponse) Write(writer io.Writer, value CreateRequestFilterResponse) {
	FfiConverterOptionalCreateRequestFilterDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerCreateRequestFilterResponse struct{}

func (_ FfiDestroyerCreateRequestFilterResponse) Destroy(value CreateRequestFilterResponse) {
	value.Destroy()
}

// Parameters for `create_set`.
type CreateSetParams struct {
	// Unique key identifying the set.
	Key string
	// String value stored under the key.
	Value string
}

func (r *CreateSetParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Key)
	FfiDestroyerString{}.Destroy(r.Value)
}

type FfiConverterCreateSetParams struct{}

var FfiConverterCreateSetParamsINSTANCE = FfiConverterCreateSetParams{}

func (c FfiConverterCreateSetParams) Lift(rb RustBufferI) CreateSetParams {
	return LiftFromRustBuffer[CreateSetParams](c, rb)
}

func (c FfiConverterCreateSetParams) Read(reader io.Reader) CreateSetParams {
	return CreateSetParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateSetParams) Lower(value CreateSetParams) C.RustBuffer {
	return LowerIntoRustBuffer[CreateSetParams](c, value)
}

func (c FfiConverterCreateSetParams) LowerExternal(value CreateSetParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateSetParams](c, value))
}

func (c FfiConverterCreateSetParams) Write(writer io.Writer, value CreateSetParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Key)
	FfiConverterStringINSTANCE.Write(writer, value.Value)
}

type FfiDestroyerCreateSetParams struct{}

func (_ FfiDestroyerCreateSetParams) Destroy(value CreateSetParams) {
	value.Destroy()
}

// Parameters for creating a new stream.
type CreateStreamParams struct {
	// Human-readable label identifying the stream.
	Name string
	// Geographic region where the stream runs.
	Region StreamRegion
	// Blockchain network to stream from (e.g. `ethereum-mainnet`).
	Network string
	// Type of on-chain data to stream.
	Dataset StreamDataset
	// Block number to begin streaming from.
	StartRange int64
	// Block number to stop streaming at; `-1` for continuous operation.
	EndRange int64
	// Destination-specific configuration (webhook URL, S3 bucket, DB credentials, etc.).
	DestinationAttributes DestinationAttributes
	// Billing plan associated with the stream. Optional; the server applies the account default when omitted.
	Plan *string
	// Buffer size used by the stream fetcher before delivery. Optional; the server applies its default when omitted.
	ThresholdFetchBuffer *int64
	// Number of blocks grouped together per delivered batch. Required by the API.
	DatasetBatchSize int64
	// Upper bound on batch size when elastic batching is enabled.
	MaxBatchSize *int64
	// Maximum number of buffered blocks waiting to be processed.
	MaxBufferRangeSize *int64
	// Maximum number of worker threads processing buffered batches.
	MaxBufferProcessingWorkers *int64
	// Number of blocks to stay behind the chain tip to reduce exposure to reorgs.
	KeepDistanceFromTip *int64
	// Base64-encoded filter function applied to each batch before delivery.
	FilterFunction *string
	// Language the filter function is written in.
	FilterLanguage *FilterLanguage
	// Optional address book to evaluate the filter against.
	AddressBookConfig *AddressBookConfig
	// Where to include stream metadata in delivered payloads.
	IncludeStreamMetadata *StreamMetadataLocation
	// Billing product type the stream is associated with.
	ProductType *ProductType
	// Initial stream state (`active` or `paused`). Defaults to `active` when omitted.
	Status *StreamStatus
	// Email address that receives stream termination or failure alerts.
	NotificationEmail *string
	// Minimum charge cap applied to the stream's billing.
	ChargeMinCap *int32
	// Flag (0 or 1) enabling automatic re-streaming of blocks affected by chain reorganizations.
	FixBlockReorgs *int32
	// When enabled, batch size is reduced toward 1 as the stream catches up to the chain tip. Required by the API.
	ElasticBatchEnabled bool
	// Additional destinations that receive the same batches alongside the primary.
	ExtraDestinations *[]DestinationAttributes
}

func (r *CreateStreamParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerStreamRegion{}.Destroy(r.Region)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerStreamDataset{}.Destroy(r.Dataset)
	FfiDestroyerInt64{}.Destroy(r.StartRange)
	FfiDestroyerInt64{}.Destroy(r.EndRange)
	FfiDestroyerDestinationAttributes{}.Destroy(r.DestinationAttributes)
	FfiDestroyerOptionalString{}.Destroy(r.Plan)
	FfiDestroyerOptionalInt64{}.Destroy(r.ThresholdFetchBuffer)
	FfiDestroyerInt64{}.Destroy(r.DatasetBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferRangeSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferProcessingWorkers)
	FfiDestroyerOptionalInt64{}.Destroy(r.KeepDistanceFromTip)
	FfiDestroyerOptionalString{}.Destroy(r.FilterFunction)
	FfiDestroyerOptionalFilterLanguage{}.Destroy(r.FilterLanguage)
	FfiDestroyerOptionalAddressBookConfig{}.Destroy(r.AddressBookConfig)
	FfiDestroyerOptionalStreamMetadataLocation{}.Destroy(r.IncludeStreamMetadata)
	FfiDestroyerOptionalProductType{}.Destroy(r.ProductType)
	FfiDestroyerOptionalStreamStatus{}.Destroy(r.Status)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalInt32{}.Destroy(r.ChargeMinCap)
	FfiDestroyerOptionalInt32{}.Destroy(r.FixBlockReorgs)
	FfiDestroyerBool{}.Destroy(r.ElasticBatchEnabled)
	FfiDestroyerOptionalSequenceDestinationAttributes{}.Destroy(r.ExtraDestinations)
}

type FfiConverterCreateStreamParams struct{}

var FfiConverterCreateStreamParamsINSTANCE = FfiConverterCreateStreamParams{}

func (c FfiConverterCreateStreamParams) Lift(rb RustBufferI) CreateStreamParams {
	return LiftFromRustBuffer[CreateStreamParams](c, rb)
}

func (c FfiConverterCreateStreamParams) Read(reader io.Reader) CreateStreamParams {
	return CreateStreamParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStreamRegionINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStreamDatasetINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterDestinationAttributesINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalFilterLanguageINSTANCE.Read(reader),
		FfiConverterOptionalAddressBookConfigINSTANCE.Read(reader),
		FfiConverterOptionalStreamMetadataLocationINSTANCE.Read(reader),
		FfiConverterOptionalProductTypeINSTANCE.Read(reader),
		FfiConverterOptionalStreamStatusINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
		FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateStreamParams) Lower(value CreateStreamParams) C.RustBuffer {
	return LowerIntoRustBuffer[CreateStreamParams](c, value)
}

func (c FfiConverterCreateStreamParams) LowerExternal(value CreateStreamParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateStreamParams](c, value))
}

func (c FfiConverterCreateStreamParams) Write(writer io.Writer, value CreateStreamParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStreamRegionINSTANCE.Write(writer, value.Region)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterStreamDatasetINSTANCE.Write(writer, value.Dataset)
	FfiConverterInt64INSTANCE.Write(writer, value.StartRange)
	FfiConverterInt64INSTANCE.Write(writer, value.EndRange)
	FfiConverterDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Plan)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.ThresholdFetchBuffer)
	FfiConverterInt64INSTANCE.Write(writer, value.DatasetBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferRangeSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferProcessingWorkers)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.KeepDistanceFromTip)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FilterFunction)
	FfiConverterOptionalFilterLanguageINSTANCE.Write(writer, value.FilterLanguage)
	FfiConverterOptionalAddressBookConfigINSTANCE.Write(writer, value.AddressBookConfig)
	FfiConverterOptionalStreamMetadataLocationINSTANCE.Write(writer, value.IncludeStreamMetadata)
	FfiConverterOptionalProductTypeINSTANCE.Write(writer, value.ProductType)
	FfiConverterOptionalStreamStatusINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.ChargeMinCap)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.FixBlockReorgs)
	FfiConverterBoolINSTANCE.Write(writer, value.ElasticBatchEnabled)
	FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Write(writer, value.ExtraDestinations)
}

type FfiDestroyerCreateStreamParams struct{}

func (_ FfiDestroyerCreateStreamParams) Destroy(value CreateStreamParams) {
	value.Destroy()
}

// Parameters for `create_tag` (on a specific endpoint).
type CreateTagRequest struct {
	// Label for the new tag. Maximum 25 characters.
	Label *string
}

func (r *CreateTagRequest) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Label)
}

type FfiConverterCreateTagRequest struct{}

var FfiConverterCreateTagRequestINSTANCE = FfiConverterCreateTagRequest{}

func (c FfiConverterCreateTagRequest) Lift(rb RustBufferI) CreateTagRequest {
	return LiftFromRustBuffer[CreateTagRequest](c, rb)
}

func (c FfiConverterCreateTagRequest) Read(reader io.Reader) CreateTagRequest {
	return CreateTagRequest{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateTagRequest) Lower(value CreateTagRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateTagRequest](c, value)
}

func (c FfiConverterCreateTagRequest) LowerExternal(value CreateTagRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateTagRequest](c, value))
}

func (c FfiConverterCreateTagRequest) Write(writer io.Writer, value CreateTagRequest) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerCreateTagRequest struct{}

func (_ FfiDestroyerCreateTagRequest) Destroy(value CreateTagRequest) {
	value.Destroy()
}

// Inner data for `create_team` responses.
type CreateTeamData struct {
	// Team identifier.
	Id int64
	// Team name.
	Name string
	// Default role for newly invited members.
	DefaultRole *string
	// Initial member count.
	MembersCount *int64
}

func (r *CreateTeamData) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.DefaultRole)
	FfiDestroyerOptionalInt64{}.Destroy(r.MembersCount)
}

type FfiConverterCreateTeamData struct{}

var FfiConverterCreateTeamDataINSTANCE = FfiConverterCreateTeamData{}

func (c FfiConverterCreateTeamData) Lift(rb RustBufferI) CreateTeamData {
	return LiftFromRustBuffer[CreateTeamData](c, rb)
}

func (c FfiConverterCreateTeamData) Read(reader io.Reader) CreateTeamData {
	return CreateTeamData{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateTeamData) Lower(value CreateTeamData) C.RustBuffer {
	return LowerIntoRustBuffer[CreateTeamData](c, value)
}

func (c FfiConverterCreateTeamData) LowerExternal(value CreateTeamData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateTeamData](c, value))
}

func (c FfiConverterCreateTeamData) Write(writer io.Writer, value CreateTeamData) {
	FfiConverterInt64INSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.DefaultRole)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MembersCount)
}

type FfiDestroyerCreateTeamData struct{}

func (_ FfiDestroyerCreateTeamData) Destroy(value CreateTeamData) {
	value.Destroy()
}

// Parameters for `create_team`.
type CreateTeamRequest struct {
	// Team name.
	Name string
}

func (r *CreateTeamRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
}

type FfiConverterCreateTeamRequest struct{}

var FfiConverterCreateTeamRequestINSTANCE = FfiConverterCreateTeamRequest{}

func (c FfiConverterCreateTeamRequest) Lift(rb RustBufferI) CreateTeamRequest {
	return LiftFromRustBuffer[CreateTeamRequest](c, rb)
}

func (c FfiConverterCreateTeamRequest) Read(reader io.Reader) CreateTeamRequest {
	return CreateTeamRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateTeamRequest) Lower(value CreateTeamRequest) C.RustBuffer {
	return LowerIntoRustBuffer[CreateTeamRequest](c, value)
}

func (c FfiConverterCreateTeamRequest) LowerExternal(value CreateTeamRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateTeamRequest](c, value))
}

func (c FfiConverterCreateTeamRequest) Write(writer io.Writer, value CreateTeamRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
}

type FfiDestroyerCreateTeamRequest struct{}

func (_ FfiDestroyerCreateTeamRequest) Destroy(value CreateTeamRequest) {
	value.Destroy()
}

// Response from `create_team`.
type CreateTeamResponse struct {
	// The newly created team.
	Data *CreateTeamData
	// Error message when the request did not succeed.
	Error *string
}

func (r *CreateTeamResponse) Destroy() {
	FfiDestroyerOptionalCreateTeamData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterCreateTeamResponse struct{}

var FfiConverterCreateTeamResponseINSTANCE = FfiConverterCreateTeamResponse{}

func (c FfiConverterCreateTeamResponse) Lift(rb RustBufferI) CreateTeamResponse {
	return LiftFromRustBuffer[CreateTeamResponse](c, rb)
}

func (c FfiConverterCreateTeamResponse) Read(reader io.Reader) CreateTeamResponse {
	return CreateTeamResponse{
		FfiConverterOptionalCreateTeamDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateTeamResponse) Lower(value CreateTeamResponse) C.RustBuffer {
	return LowerIntoRustBuffer[CreateTeamResponse](c, value)
}

func (c FfiConverterCreateTeamResponse) LowerExternal(value CreateTeamResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateTeamResponse](c, value))
}

func (c FfiConverterCreateTeamResponse) Write(writer io.Writer, value CreateTeamResponse) {
	FfiConverterOptionalCreateTeamDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerCreateTeamResponse struct{}

func (_ FfiDestroyerCreateTeamResponse) Destroy(value CreateTeamResponse) {
	value.Destroy()
}

// Parameters for `create_webhook_from_template`.
type CreateWebhookFromTemplateParams struct {
	// Human-readable label for the webhook.
	Name string
	// Blockchain network to watch (e.g. `ethereum-mainnet`).
	Network string
	// Optional email that receives alerts if the webhook terminates.
	NotificationEmail *string
	// Destination configuration for delivered payloads.
	DestinationAttributes WebhookDestinationAttributes
	// Filter template identifier and its arguments.
	TemplateArgs TemplateArgs
}

func (r *CreateWebhookFromTemplateParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerWebhookDestinationAttributes{}.Destroy(r.DestinationAttributes)
	FfiDestroyerTemplateArgs{}.Destroy(r.TemplateArgs)
}

type FfiConverterCreateWebhookFromTemplateParams struct{}

var FfiConverterCreateWebhookFromTemplateParamsINSTANCE = FfiConverterCreateWebhookFromTemplateParams{}

func (c FfiConverterCreateWebhookFromTemplateParams) Lift(rb RustBufferI) CreateWebhookFromTemplateParams {
	return LiftFromRustBuffer[CreateWebhookFromTemplateParams](c, rb)
}

func (c FfiConverterCreateWebhookFromTemplateParams) Read(reader io.Reader) CreateWebhookFromTemplateParams {
	return CreateWebhookFromTemplateParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterWebhookDestinationAttributesINSTANCE.Read(reader),
		FfiConverterTemplateArgsINSTANCE.Read(reader),
	}
}

func (c FfiConverterCreateWebhookFromTemplateParams) Lower(value CreateWebhookFromTemplateParams) C.RustBuffer {
	return LowerIntoRustBuffer[CreateWebhookFromTemplateParams](c, value)
}

func (c FfiConverterCreateWebhookFromTemplateParams) LowerExternal(value CreateWebhookFromTemplateParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CreateWebhookFromTemplateParams](c, value))
}

func (c FfiConverterCreateWebhookFromTemplateParams) Write(writer io.Writer, value CreateWebhookFromTemplateParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterWebhookDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
	FfiConverterTemplateArgsINSTANCE.Write(writer, value.TemplateArgs)
}

type FfiDestroyerCreateWebhookFromTemplateParams struct{}

func (_ FfiDestroyerCreateWebhookFromTemplateParams) Destroy(value CreateWebhookFromTemplateParams) {
	value.Destroy()
}

// Inner data for `delete_account_tag`.
type DeleteAccountTagData struct {
	// `true` when the tag was deleted.
	Success bool
}

func (r *DeleteAccountTagData) Destroy() {
	FfiDestroyerBool{}.Destroy(r.Success)
}

type FfiConverterDeleteAccountTagData struct{}

var FfiConverterDeleteAccountTagDataINSTANCE = FfiConverterDeleteAccountTagData{}

func (c FfiConverterDeleteAccountTagData) Lift(rb RustBufferI) DeleteAccountTagData {
	return LiftFromRustBuffer[DeleteAccountTagData](c, rb)
}

func (c FfiConverterDeleteAccountTagData) Read(reader io.Reader) DeleteAccountTagData {
	return DeleteAccountTagData{
		FfiConverterBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterDeleteAccountTagData) Lower(value DeleteAccountTagData) C.RustBuffer {
	return LowerIntoRustBuffer[DeleteAccountTagData](c, value)
}

func (c FfiConverterDeleteAccountTagData) LowerExternal(value DeleteAccountTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DeleteAccountTagData](c, value))
}

func (c FfiConverterDeleteAccountTagData) Write(writer io.Writer, value DeleteAccountTagData) {
	FfiConverterBoolINSTANCE.Write(writer, value.Success)
}

type FfiDestroyerDeleteAccountTagData struct{}

func (_ FfiDestroyerDeleteAccountTagData) Destroy(value DeleteAccountTagData) {
	value.Destroy()
}

// Response from `delete_account_tag`.
type DeleteAccountTagResponse struct {
	// Deletion result.
	Data *DeleteAccountTagData
	// Error message when the request did not succeed.
	Error *string
}

func (r *DeleteAccountTagResponse) Destroy() {
	FfiDestroyerOptionalDeleteAccountTagData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterDeleteAccountTagResponse struct{}

var FfiConverterDeleteAccountTagResponseINSTANCE = FfiConverterDeleteAccountTagResponse{}

func (c FfiConverterDeleteAccountTagResponse) Lift(rb RustBufferI) DeleteAccountTagResponse {
	return LiftFromRustBuffer[DeleteAccountTagResponse](c, rb)
}

func (c FfiConverterDeleteAccountTagResponse) Read(reader io.Reader) DeleteAccountTagResponse {
	return DeleteAccountTagResponse{
		FfiConverterOptionalDeleteAccountTagDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterDeleteAccountTagResponse) Lower(value DeleteAccountTagResponse) C.RustBuffer {
	return LowerIntoRustBuffer[DeleteAccountTagResponse](c, value)
}

func (c FfiConverterDeleteAccountTagResponse) LowerExternal(value DeleteAccountTagResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DeleteAccountTagResponse](c, value))
}

func (c FfiConverterDeleteAccountTagResponse) Write(writer io.Writer, value DeleteAccountTagResponse) {
	FfiConverterOptionalDeleteAccountTagDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerDeleteAccountTagResponse struct{}

func (_ FfiDestroyerDeleteAccountTagResponse) Destroy(value DeleteAccountTagResponse) {
	value.Destroy()
}

// Response wrapper for delete operations that return a boolean success flag.
type DeleteBoolResponse struct {
	// `true` when the deletion succeeded.
	Data *bool
	// Error message when the request did not succeed.
	Error *string
}

func (r *DeleteBoolResponse) Destroy() {
	FfiDestroyerOptionalBool{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterDeleteBoolResponse struct{}

var FfiConverterDeleteBoolResponseINSTANCE = FfiConverterDeleteBoolResponse{}

func (c FfiConverterDeleteBoolResponse) Lift(rb RustBufferI) DeleteBoolResponse {
	return LiftFromRustBuffer[DeleteBoolResponse](c, rb)
}

func (c FfiConverterDeleteBoolResponse) Read(reader io.Reader) DeleteBoolResponse {
	return DeleteBoolResponse{
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterDeleteBoolResponse) Lower(value DeleteBoolResponse) C.RustBuffer {
	return LowerIntoRustBuffer[DeleteBoolResponse](c, value)
}

func (c FfiConverterDeleteBoolResponse) LowerExternal(value DeleteBoolResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DeleteBoolResponse](c, value))
}

func (c FfiConverterDeleteBoolResponse) Write(writer io.Writer, value DeleteBoolResponse) {
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerDeleteBoolResponse struct{}

func (_ FfiDestroyerDeleteBoolResponse) Destroy(value DeleteBoolResponse) {
	value.Destroy()
}

// Inner data for `delete_team` responses.
type DeleteTeamData struct {
	// Human-readable confirmation message.
	Message *string
}

func (r *DeleteTeamData) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Message)
}

type FfiConverterDeleteTeamData struct{}

var FfiConverterDeleteTeamDataINSTANCE = FfiConverterDeleteTeamData{}

func (c FfiConverterDeleteTeamData) Lift(rb RustBufferI) DeleteTeamData {
	return LiftFromRustBuffer[DeleteTeamData](c, rb)
}

func (c FfiConverterDeleteTeamData) Read(reader io.Reader) DeleteTeamData {
	return DeleteTeamData{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterDeleteTeamData) Lower(value DeleteTeamData) C.RustBuffer {
	return LowerIntoRustBuffer[DeleteTeamData](c, value)
}

func (c FfiConverterDeleteTeamData) LowerExternal(value DeleteTeamData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DeleteTeamData](c, value))
}

func (c FfiConverterDeleteTeamData) Write(writer io.Writer, value DeleteTeamData) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Message)
}

type FfiDestroyerDeleteTeamData struct{}

func (_ FfiDestroyerDeleteTeamData) Destroy(value DeleteTeamData) {
	value.Destroy()
}

// Response from `delete_team`.
type DeleteTeamResponse struct {
	// Deletion result payload.
	Data *DeleteTeamData
	// Error message when the request did not succeed.
	Error *string
}

func (r *DeleteTeamResponse) Destroy() {
	FfiDestroyerOptionalDeleteTeamData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterDeleteTeamResponse struct{}

var FfiConverterDeleteTeamResponseINSTANCE = FfiConverterDeleteTeamResponse{}

func (c FfiConverterDeleteTeamResponse) Lift(rb RustBufferI) DeleteTeamResponse {
	return LiftFromRustBuffer[DeleteTeamResponse](c, rb)
}

func (c FfiConverterDeleteTeamResponse) Read(reader io.Reader) DeleteTeamResponse {
	return DeleteTeamResponse{
		FfiConverterOptionalDeleteTeamDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterDeleteTeamResponse) Lower(value DeleteTeamResponse) C.RustBuffer {
	return LowerIntoRustBuffer[DeleteTeamResponse](c, value)
}

func (c FfiConverterDeleteTeamResponse) LowerExternal(value DeleteTeamResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DeleteTeamResponse](c, value))
}

func (c FfiConverterDeleteTeamResponse) Write(writer io.Writer, value DeleteTeamResponse) {
	FfiConverterOptionalDeleteTeamDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerDeleteTeamResponse struct{}

func (_ FfiDestroyerDeleteTeamResponse) Destroy(value DeleteTeamResponse) {
	value.Destroy()
}

// Result of `get_enabled_count`.
type EnabledCountResponse struct {
	// Total count of currently enabled streams.
	Total int64
}

func (r *EnabledCountResponse) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Total)
}

type FfiConverterEnabledCountResponse struct{}

var FfiConverterEnabledCountResponseINSTANCE = FfiConverterEnabledCountResponse{}

func (c FfiConverterEnabledCountResponse) Lift(rb RustBufferI) EnabledCountResponse {
	return LiftFromRustBuffer[EnabledCountResponse](c, rb)
}

func (c FfiConverterEnabledCountResponse) Read(reader io.Reader) EnabledCountResponse {
	return EnabledCountResponse{
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterEnabledCountResponse) Lower(value EnabledCountResponse) C.RustBuffer {
	return LowerIntoRustBuffer[EnabledCountResponse](c, value)
}

func (c FfiConverterEnabledCountResponse) LowerExternal(value EnabledCountResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EnabledCountResponse](c, value))
}

func (c FfiConverterEnabledCountResponse) Write(writer io.Writer, value EnabledCountResponse) {
	FfiConverterInt64INSTANCE.Write(writer, value.Total)
}

type FfiDestroyerEnabledCountResponse struct{}

func (_ FfiDestroyerEnabledCountResponse) Destroy(value EnabledCountResponse) {
	value.Destroy()
}

// Summary representation of an endpoint in list responses.
type Endpoint struct {
	// Unique endpoint identifier.
	Id string
	// Quicknode-assigned subdomain.
	Name string
	// Human-readable label.
	Label *string
	// Current operational status (e.g. `active`, `paused`).
	Status string
	// Blockchain the endpoint serves (e.g. `ethereum`).
	Chain string
	// Specific network within the chain (e.g. `mainnet`).
	Network string
	// Whether the endpoint is dedicated.
	IsDedicated bool
	// Whether the endpoint is billed at a flat rate.
	IsFlatRate bool
	// HTTP RPC URL.
	HttpUrl string
	// WebSocket RPC URL, when available.
	WssUrl *string
	// Tags applied to the endpoint.
	Tags []EndpointTag
	// Whether the endpoint is configured to serve multiple chains/networks.
	IsMultichain bool
}

func (r *Endpoint) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.Label)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.Chain)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerBool{}.Destroy(r.IsDedicated)
	FfiDestroyerBool{}.Destroy(r.IsFlatRate)
	FfiDestroyerString{}.Destroy(r.HttpUrl)
	FfiDestroyerOptionalString{}.Destroy(r.WssUrl)
	FfiDestroyerSequenceEndpointTag{}.Destroy(r.Tags)
	FfiDestroyerBool{}.Destroy(r.IsMultichain)
}

type FfiConverterEndpoint struct{}

var FfiConverterEndpointINSTANCE = FfiConverterEndpoint{}

func (c FfiConverterEndpoint) Lift(rb RustBufferI) Endpoint {
	return LiftFromRustBuffer[Endpoint](c, rb)
}

func (c FfiConverterEndpoint) Read(reader io.Reader) Endpoint {
	return Endpoint{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterSequenceEndpointTagINSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpoint) Lower(value Endpoint) C.RustBuffer {
	return LowerIntoRustBuffer[Endpoint](c, value)
}

func (c FfiConverterEndpoint) LowerExternal(value Endpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Endpoint](c, value))
}

func (c FfiConverterEndpoint) Write(writer io.Writer, value Endpoint) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.Chain)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterBoolINSTANCE.Write(writer, value.IsDedicated)
	FfiConverterBoolINSTANCE.Write(writer, value.IsFlatRate)
	FfiConverterStringINSTANCE.Write(writer, value.HttpUrl)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.WssUrl)
	FfiConverterSequenceEndpointTagINSTANCE.Write(writer, value.Tags)
	FfiConverterBoolINSTANCE.Write(writer, value.IsMultichain)
}

type FfiDestroyerEndpoint struct{}

func (_ FfiDestroyerEndpoint) Destroy(value Endpoint) {
	value.Destroy()
}

// Domain mask configured on an endpoint.
type EndpointDomainMask struct {
	// Domain mask identifier.
	Id string
	// Masking domain.
	Domain string
}

func (r *EndpointDomainMask) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Domain)
}

type FfiConverterEndpointDomainMask struct{}

var FfiConverterEndpointDomainMaskINSTANCE = FfiConverterEndpointDomainMask{}

func (c FfiConverterEndpointDomainMask) Lift(rb RustBufferI) EndpointDomainMask {
	return LiftFromRustBuffer[EndpointDomainMask](c, rb)
}

func (c FfiConverterEndpointDomainMask) Read(reader io.Reader) EndpointDomainMask {
	return EndpointDomainMask{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointDomainMask) Lower(value EndpointDomainMask) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointDomainMask](c, value)
}

func (c FfiConverterEndpointDomainMask) LowerExternal(value EndpointDomainMask) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointDomainMask](c, value))
}

func (c FfiConverterEndpointDomainMask) Write(writer io.Writer, value EndpointDomainMask) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Domain)
}

type FfiDestroyerEndpointDomainMask struct{}

func (_ FfiDestroyerEndpointDomainMask) Destroy(value EndpointDomainMask) {
	value.Destroy()
}

// Whitelisted IP address on an endpoint.
type EndpointIp struct {
	// IP entry identifier.
	Id string
	// Whitelisted IP address.
	Ip string
}

func (r *EndpointIp) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Ip)
}

type FfiConverterEndpointIp struct{}

var FfiConverterEndpointIpINSTANCE = FfiConverterEndpointIp{}

func (c FfiConverterEndpointIp) Lift(rb RustBufferI) EndpointIp {
	return LiftFromRustBuffer[EndpointIp](c, rb)
}

func (c FfiConverterEndpointIp) Read(reader io.Reader) EndpointIp {
	return EndpointIp{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointIp) Lower(value EndpointIp) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointIp](c, value)
}

func (c FfiConverterEndpointIp) LowerExternal(value EndpointIp) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointIp](c, value))
}

func (c FfiConverterEndpointIp) Write(writer io.Writer, value EndpointIp) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Ip)
}

type FfiDestroyerEndpointIp struct{}

func (_ FfiDestroyerEndpointIp) Destroy(value EndpointIp) {
	value.Destroy()
}

// Custom header option value for IP identification.
type EndpointIpCustomHeaderOption struct {
	// Header name (e.g. `X-Forwarded-For`).
	Value *string
}

func (r *EndpointIpCustomHeaderOption) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Value)
}

type FfiConverterEndpointIpCustomHeaderOption struct{}

var FfiConverterEndpointIpCustomHeaderOptionINSTANCE = FfiConverterEndpointIpCustomHeaderOption{}

func (c FfiConverterEndpointIpCustomHeaderOption) Lift(rb RustBufferI) EndpointIpCustomHeaderOption {
	return LiftFromRustBuffer[EndpointIpCustomHeaderOption](c, rb)
}

func (c FfiConverterEndpointIpCustomHeaderOption) Read(reader io.Reader) EndpointIpCustomHeaderOption {
	return EndpointIpCustomHeaderOption{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointIpCustomHeaderOption) Lower(value EndpointIpCustomHeaderOption) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointIpCustomHeaderOption](c, value)
}

func (c FfiConverterEndpointIpCustomHeaderOption) LowerExternal(value EndpointIpCustomHeaderOption) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointIpCustomHeaderOption](c, value))
}

func (c FfiConverterEndpointIpCustomHeaderOption) Write(writer io.Writer, value EndpointIpCustomHeaderOption) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Value)
}

type FfiDestroyerEndpointIpCustomHeaderOption struct{}

func (_ FfiDestroyerEndpointIpCustomHeaderOption) Destroy(value EndpointIpCustomHeaderOption) {
	value.Destroy()
}

// JWT configured on an endpoint for signed-request authentication.
type EndpointJwt struct {
	// JWT identifier.
	Id string
	// Public key used to verify signed JWTs.
	PublicKey string
	// Key identifier (`kid`) embedded in JWT headers.
	Kid string
	// Human-readable name.
	Name string
}

func (r *EndpointJwt) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.PublicKey)
	FfiDestroyerString{}.Destroy(r.Kid)
	FfiDestroyerString{}.Destroy(r.Name)
}

type FfiConverterEndpointJwt struct{}

var FfiConverterEndpointJwtINSTANCE = FfiConverterEndpointJwt{}

func (c FfiConverterEndpointJwt) Lift(rb RustBufferI) EndpointJwt {
	return LiftFromRustBuffer[EndpointJwt](c, rb)
}

func (c FfiConverterEndpointJwt) Read(reader io.Reader) EndpointJwt {
	return EndpointJwt{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointJwt) Lower(value EndpointJwt) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointJwt](c, value)
}

func (c FfiConverterEndpointJwt) LowerExternal(value EndpointJwt) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointJwt](c, value))
}

func (c FfiConverterEndpointJwt) Write(writer io.Writer, value EndpointJwt) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.PublicKey)
	FfiConverterStringINSTANCE.Write(writer, value.Kid)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
}

type FfiDestroyerEndpointJwt struct{}

func (_ FfiDestroyerEndpointJwt) Destroy(value EndpointJwt) {
	value.Destroy()
}

// A single endpoint log entry.
type EndpointLog struct {
	// Time the request was received.
	Timestamp string
	// RPC method called (e.g. `eth_blockNumber`).
	Method *string
	// Network the request was routed to.
	Network *string
	// HTTP verb (e.g. `POST`).
	HttpMethod *string
	// Response HTTP status code.
	Status *int32
	// JSON-RPC error code, when present.
	ErrorCode *int64
	// Request URL.
	Url *string
	// Request UUID used to fetch full log details.
	RequestId *string
	// Full payloads, included when requested.
	Details *LogDetails
}

func (r *EndpointLog) Destroy() {
	FfiDestroyerString{}.Destroy(r.Timestamp)
	FfiDestroyerOptionalString{}.Destroy(r.Method)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
	FfiDestroyerOptionalString{}.Destroy(r.HttpMethod)
	FfiDestroyerOptionalInt32{}.Destroy(r.Status)
	FfiDestroyerOptionalInt64{}.Destroy(r.ErrorCode)
	FfiDestroyerOptionalString{}.Destroy(r.Url)
	FfiDestroyerOptionalString{}.Destroy(r.RequestId)
	FfiDestroyerOptionalLogDetails{}.Destroy(r.Details)
}

type FfiConverterEndpointLog struct{}

var FfiConverterEndpointLogINSTANCE = FfiConverterEndpointLog{}

func (c FfiConverterEndpointLog) Lift(rb RustBufferI) EndpointLog {
	return LiftFromRustBuffer[EndpointLog](c, rb)
}

func (c FfiConverterEndpointLog) Read(reader io.Reader) EndpointLog {
	return EndpointLog{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalLogDetailsINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointLog) Lower(value EndpointLog) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointLog](c, value)
}

func (c FfiConverterEndpointLog) LowerExternal(value EndpointLog) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointLog](c, value))
}

func (c FfiConverterEndpointLog) Write(writer io.Writer, value EndpointLog) {
	FfiConverterStringINSTANCE.Write(writer, value.Timestamp)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Method)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.HttpMethod)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Status)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.ErrorCode)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Url)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.RequestId)
	FfiConverterOptionalLogDetailsINSTANCE.Write(writer, value.Details)
}

type FfiDestroyerEndpointLog struct{}

func (_ FfiDestroyerEndpointLog) Destroy(value EndpointLog) {
	value.Destroy()
}

// A single metric series, consisting of a descriptive tag and timestamped data points.
type EndpointMetric struct {
	// Data points, each as `[timestamp, value]`.
	Data [][]int64
	// Tag identifying the series. Single-axis metrics return a one-element
	// vector (e.g. `["total"]`, `["p95"]`); multi-axis metrics return the
	// key/value pair (e.g. `["network", "arbitrum-mainnet"]`).
	Tag []string
}

func (r *EndpointMetric) Destroy() {
	FfiDestroyerSequenceSequenceInt64{}.Destroy(r.Data)
	FfiDestroyerSequenceString{}.Destroy(r.Tag)
}

type FfiConverterEndpointMetric struct{}

var FfiConverterEndpointMetricINSTANCE = FfiConverterEndpointMetric{}

func (c FfiConverterEndpointMetric) Lift(rb RustBufferI) EndpointMetric {
	return LiftFromRustBuffer[EndpointMetric](c, rb)
}

func (c FfiConverterEndpointMetric) Read(reader io.Reader) EndpointMetric {
	return EndpointMetric{
		FfiConverterSequenceSequenceInt64INSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointMetric) Lower(value EndpointMetric) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointMetric](c, value)
}

func (c FfiConverterEndpointMetric) LowerExternal(value EndpointMetric) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointMetric](c, value))
}

func (c FfiConverterEndpointMetric) Write(writer io.Writer, value EndpointMetric) {
	FfiConverterSequenceSequenceInt64INSTANCE.Write(writer, value.Data)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Tag)
}

type FfiDestroyerEndpointMetric struct{}

func (_ FfiDestroyerEndpointMetric) Destroy(value EndpointMetric) {
	value.Destroy()
}

// Rate limits applied to an endpoint.
type EndpointRateLimits struct {
	// Whether rate limits are applied per client IP instead of per endpoint.
	RateLimitByIp *bool
	// Account-level rate limit, when applicable.
	Account *int32
	// Requests per second.
	Rps *int32
	// Requests per minute.
	Rpm *int32
	// Requests per day.
	Rpd *int32
}

func (r *EndpointRateLimits) Destroy() {
	FfiDestroyerOptionalBool{}.Destroy(r.RateLimitByIp)
	FfiDestroyerOptionalInt32{}.Destroy(r.Account)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rps)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rpm)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rpd)
}

type FfiConverterEndpointRateLimits struct{}

var FfiConverterEndpointRateLimitsINSTANCE = FfiConverterEndpointRateLimits{}

func (c FfiConverterEndpointRateLimits) Lift(rb RustBufferI) EndpointRateLimits {
	return LiftFromRustBuffer[EndpointRateLimits](c, rb)
}

func (c FfiConverterEndpointRateLimits) Read(reader io.Reader) EndpointRateLimits {
	return EndpointRateLimits{
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointRateLimits) Lower(value EndpointRateLimits) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointRateLimits](c, value)
}

func (c FfiConverterEndpointRateLimits) LowerExternal(value EndpointRateLimits) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointRateLimits](c, value))
}

func (c FfiConverterEndpointRateLimits) Write(writer io.Writer, value EndpointRateLimits) {
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.RateLimitByIp)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Account)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rps)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rpm)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rpd)
}

type FfiDestroyerEndpointRateLimits struct{}

func (_ FfiDestroyerEndpointRateLimits) Destroy(value EndpointRateLimits) {
	value.Destroy()
}

// Allowed referrer entry for request-origin validation.
type EndpointReferrer struct {
	// Referrer entry identifier.
	Id string
	// Allowed referrer URL or domain.
	Referrer *string
}

func (r *EndpointReferrer) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerOptionalString{}.Destroy(r.Referrer)
}

type FfiConverterEndpointReferrer struct{}

var FfiConverterEndpointReferrerINSTANCE = FfiConverterEndpointReferrer{}

func (c FfiConverterEndpointReferrer) Lift(rb RustBufferI) EndpointReferrer {
	return LiftFromRustBuffer[EndpointReferrer](c, rb)
}

func (c FfiConverterEndpointReferrer) Read(reader io.Reader) EndpointReferrer {
	return EndpointReferrer{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointReferrer) Lower(value EndpointReferrer) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointReferrer](c, value)
}

func (c FfiConverterEndpointReferrer) LowerExternal(value EndpointReferrer) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointReferrer](c, value))
}

func (c FfiConverterEndpointReferrer) Write(writer io.Writer, value EndpointReferrer) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Referrer)
}

type FfiDestroyerEndpointReferrer struct{}

func (_ FfiDestroyerEndpointReferrer) Destroy(value EndpointReferrer) {
	value.Destroy()
}

// Request (method) filter configured on an endpoint.
type EndpointRequestFilter struct {
	// Filter identifier.
	Id string
	// Whitelisted RPC methods.
	Method []string
}

func (r *EndpointRequestFilter) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerSequenceString{}.Destroy(r.Method)
}

type FfiConverterEndpointRequestFilter struct{}

var FfiConverterEndpointRequestFilterINSTANCE = FfiConverterEndpointRequestFilter{}

func (c FfiConverterEndpointRequestFilter) Lift(rb RustBufferI) EndpointRequestFilter {
	return LiftFromRustBuffer[EndpointRequestFilter](c, rb)
}

func (c FfiConverterEndpointRequestFilter) Read(reader io.Reader) EndpointRequestFilter {
	return EndpointRequestFilter{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointRequestFilter) Lower(value EndpointRequestFilter) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointRequestFilter](c, value)
}

func (c FfiConverterEndpointRequestFilter) LowerExternal(value EndpointRequestFilter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointRequestFilter](c, value))
}

func (c FfiConverterEndpointRequestFilter) Write(writer io.Writer, value EndpointRequestFilter) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Method)
}

type FfiDestroyerEndpointRequestFilter struct{}

func (_ FfiDestroyerEndpointRequestFilter) Destroy(value EndpointRequestFilter) {
	value.Destroy()
}

// Security configuration for an endpoint — the aggregate of tokens, JWTs,
// referrers, domain masks, IPs, and request filters plus their enabled
// toggles.
type EndpointSecurity struct {
	// Per-feature enabled/disabled toggles.
	Options *EndpointSecurityOptions
	// Authentication tokens configured on the endpoint.
	Tokens *[]EndpointToken
	// JWTs configured on the endpoint.
	Jwts *[]EndpointJwt
	// Allowed referrer URLs/domains.
	Referrers *[]EndpointReferrer
	// Configured domain masks.
	DomainMasks *[]EndpointDomainMask
	// Whitelisted IP addresses.
	Ips *[]EndpointIp
	// Request (method) filters.
	RequestFilters *[]EndpointRequestFilter
}

func (r *EndpointSecurity) Destroy() {
	FfiDestroyerOptionalEndpointSecurityOptions{}.Destroy(r.Options)
	FfiDestroyerOptionalSequenceEndpointToken{}.Destroy(r.Tokens)
	FfiDestroyerOptionalSequenceEndpointJwt{}.Destroy(r.Jwts)
	FfiDestroyerOptionalSequenceEndpointReferrer{}.Destroy(r.Referrers)
	FfiDestroyerOptionalSequenceEndpointDomainMask{}.Destroy(r.DomainMasks)
	FfiDestroyerOptionalSequenceEndpointIp{}.Destroy(r.Ips)
	FfiDestroyerOptionalSequenceEndpointRequestFilter{}.Destroy(r.RequestFilters)
}

type FfiConverterEndpointSecurity struct{}

var FfiConverterEndpointSecurityINSTANCE = FfiConverterEndpointSecurity{}

func (c FfiConverterEndpointSecurity) Lift(rb RustBufferI) EndpointSecurity {
	return LiftFromRustBuffer[EndpointSecurity](c, rb)
}

func (c FfiConverterEndpointSecurity) Read(reader io.Reader) EndpointSecurity {
	return EndpointSecurity{
		FfiConverterOptionalEndpointSecurityOptionsINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointTokenINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointJwtINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointReferrerINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointDomainMaskINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointIpINSTANCE.Read(reader),
		FfiConverterOptionalSequenceEndpointRequestFilterINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointSecurity) Lower(value EndpointSecurity) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointSecurity](c, value)
}

func (c FfiConverterEndpointSecurity) LowerExternal(value EndpointSecurity) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointSecurity](c, value))
}

func (c FfiConverterEndpointSecurity) Write(writer io.Writer, value EndpointSecurity) {
	FfiConverterOptionalEndpointSecurityOptionsINSTANCE.Write(writer, value.Options)
	FfiConverterOptionalSequenceEndpointTokenINSTANCE.Write(writer, value.Tokens)
	FfiConverterOptionalSequenceEndpointJwtINSTANCE.Write(writer, value.Jwts)
	FfiConverterOptionalSequenceEndpointReferrerINSTANCE.Write(writer, value.Referrers)
	FfiConverterOptionalSequenceEndpointDomainMaskINSTANCE.Write(writer, value.DomainMasks)
	FfiConverterOptionalSequenceEndpointIpINSTANCE.Write(writer, value.Ips)
	FfiConverterOptionalSequenceEndpointRequestFilterINSTANCE.Write(writer, value.RequestFilters)
}

type FfiDestroyerEndpointSecurity struct{}

func (_ FfiDestroyerEndpointSecurity) Destroy(value EndpointSecurity) {
	value.Destroy()
}

// Boolean toggles controlling which security features are enabled.
type EndpointSecurityOptions struct {
	// Whether token authentication is enforced.
	Tokens *bool
	// Whether JWT validation is enforced.
	Jwts *bool
	// Whether domain masking is enabled.
	DomainMasks *bool
	// Whether IP whitelisting is enforced.
	Ips *bool
	// Whether referrer validation is enforced.
	Referrers *bool
	// Whether request (method) filtering is enforced.
	RequestFilters *bool
	// Custom header used to identify the client IP.
	IpCustomHeader *EndpointIpCustomHeaderOption
}

func (r *EndpointSecurityOptions) Destroy() {
	FfiDestroyerOptionalBool{}.Destroy(r.Tokens)
	FfiDestroyerOptionalBool{}.Destroy(r.Jwts)
	FfiDestroyerOptionalBool{}.Destroy(r.DomainMasks)
	FfiDestroyerOptionalBool{}.Destroy(r.Ips)
	FfiDestroyerOptionalBool{}.Destroy(r.Referrers)
	FfiDestroyerOptionalBool{}.Destroy(r.RequestFilters)
	FfiDestroyerOptionalEndpointIpCustomHeaderOption{}.Destroy(r.IpCustomHeader)
}

type FfiConverterEndpointSecurityOptions struct{}

var FfiConverterEndpointSecurityOptionsINSTANCE = FfiConverterEndpointSecurityOptions{}

func (c FfiConverterEndpointSecurityOptions) Lift(rb RustBufferI) EndpointSecurityOptions {
	return LiftFromRustBuffer[EndpointSecurityOptions](c, rb)
}

func (c FfiConverterEndpointSecurityOptions) Read(reader io.Reader) EndpointSecurityOptions {
	return EndpointSecurityOptions{
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalEndpointIpCustomHeaderOptionINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointSecurityOptions) Lower(value EndpointSecurityOptions) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointSecurityOptions](c, value)
}

func (c FfiConverterEndpointSecurityOptions) LowerExternal(value EndpointSecurityOptions) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointSecurityOptions](c, value))
}

func (c FfiConverterEndpointSecurityOptions) Write(writer io.Writer, value EndpointSecurityOptions) {
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Tokens)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Jwts)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.DomainMasks)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Ips)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Referrers)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.RequestFilters)
	FfiConverterOptionalEndpointIpCustomHeaderOptionINSTANCE.Write(writer, value.IpCustomHeader)
}

type FfiDestroyerEndpointSecurityOptions struct{}

func (_ FfiDestroyerEndpointSecurityOptions) Destroy(value EndpointSecurityOptions) {
	value.Destroy()
}

// Tag reference as returned on an endpoint.
type EndpointTag struct {
	// Tag identifier.
	TagId int32
	// Tag label.
	Label string
}

func (r *EndpointTag) Destroy() {
	FfiDestroyerInt32{}.Destroy(r.TagId)
	FfiDestroyerString{}.Destroy(r.Label)
}

type FfiConverterEndpointTag struct{}

var FfiConverterEndpointTagINSTANCE = FfiConverterEndpointTag{}

func (c FfiConverterEndpointTag) Lift(rb RustBufferI) EndpointTag {
	return LiftFromRustBuffer[EndpointTag](c, rb)
}

func (c FfiConverterEndpointTag) Read(reader io.Reader) EndpointTag {
	return EndpointTag{
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointTag) Lower(value EndpointTag) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointTag](c, value)
}

func (c FfiConverterEndpointTag) LowerExternal(value EndpointTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointTag](c, value))
}

func (c FfiConverterEndpointTag) Write(writer io.Writer, value EndpointTag) {
	FfiConverterInt32INSTANCE.Write(writer, value.TagId)
	FfiConverterStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerEndpointTag struct{}

func (_ FfiDestroyerEndpointTag) Destroy(value EndpointTag) {
	value.Destroy()
}

// Authentication token configured on an endpoint.
type EndpointToken struct {
	// Token identifier.
	Id string
	// Token secret.
	Token string
}

func (r *EndpointToken) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Token)
}

type FfiConverterEndpointToken struct{}

var FfiConverterEndpointTokenINSTANCE = FfiConverterEndpointToken{}

func (c FfiConverterEndpointToken) Lift(rb RustBufferI) EndpointToken {
	return LiftFromRustBuffer[EndpointToken](c, rb)
}

func (c FfiConverterEndpointToken) Read(reader io.Reader) EndpointToken {
	return EndpointToken{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointToken) Lower(value EndpointToken) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointToken](c, value)
}

func (c FfiConverterEndpointToken) LowerExternal(value EndpointToken) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointToken](c, value))
}

func (c FfiConverterEndpointToken) Write(writer io.Writer, value EndpointToken) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Token)
}

type FfiDestroyerEndpointToken struct{}

func (_ FfiDestroyerEndpointToken) Destroy(value EndpointToken) {
	value.Destroy()
}

// HTTP/WSS URL pair for a single network on a multichain endpoint.
type EndpointUrl struct {
	// HTTP RPC URL.
	HttpUrl string
	// WebSocket RPC URL, when available.
	WssUrl *string
}

func (r *EndpointUrl) Destroy() {
	FfiDestroyerString{}.Destroy(r.HttpUrl)
	FfiDestroyerOptionalString{}.Destroy(r.WssUrl)
}

type FfiConverterEndpointUrl struct{}

var FfiConverterEndpointUrlINSTANCE = FfiConverterEndpointUrl{}

func (c FfiConverterEndpointUrl) Lift(rb RustBufferI) EndpointUrl {
	return LiftFromRustBuffer[EndpointUrl](c, rb)
}

func (c FfiConverterEndpointUrl) Read(reader io.Reader) EndpointUrl {
	return EndpointUrl{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointUrl) Lower(value EndpointUrl) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointUrl](c, value)
}

func (c FfiConverterEndpointUrl) LowerExternal(value EndpointUrl) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointUrl](c, value))
}

func (c FfiConverterEndpointUrl) Write(writer io.Writer, value EndpointUrl) {
	FfiConverterStringINSTANCE.Write(writer, value.HttpUrl)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.WssUrl)
}

type FfiDestroyerEndpointUrl struct{}

func (_ FfiDestroyerEndpointUrl) Destroy(value EndpointUrl) {
	value.Destroy()
}

// Per-endpoint usage row.
type EndpointUsage struct {
	// Endpoint subdomain.
	Name string
	// Blockchain the endpoint serves.
	Chain *string
	// Network within the chain.
	Network *string
	// Operational status during the window.
	Status *string
	// Total credits consumed by this endpoint.
	CreditsUsed int64
	// Human-readable label.
	Label *string
	// Per-method credit breakdown.
	MethodsBreakdown []MethodUsage
	// Request count during the window.
	Requests *int64
}

func (r *EndpointUsage) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.Chain)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
	FfiDestroyerOptionalString{}.Destroy(r.Status)
	FfiDestroyerInt64{}.Destroy(r.CreditsUsed)
	FfiDestroyerOptionalString{}.Destroy(r.Label)
	FfiDestroyerSequenceMethodUsage{}.Destroy(r.MethodsBreakdown)
	FfiDestroyerOptionalInt64{}.Destroy(r.Requests)
}

type FfiConverterEndpointUsage struct{}

var FfiConverterEndpointUsageINSTANCE = FfiConverterEndpointUsage{}

func (c FfiConverterEndpointUsage) Lift(rb RustBufferI) EndpointUsage {
	return LiftFromRustBuffer[EndpointUsage](c, rb)
}

func (c FfiConverterEndpointUsage) Read(reader io.Reader) EndpointUsage {
	return EndpointUsage{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterSequenceMethodUsageINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterEndpointUsage) Lower(value EndpointUsage) C.RustBuffer {
	return LowerIntoRustBuffer[EndpointUsage](c, value)
}

func (c FfiConverterEndpointUsage) LowerExternal(value EndpointUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EndpointUsage](c, value))
}

func (c FfiConverterEndpointUsage) Write(writer io.Writer, value EndpointUsage) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Chain)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Status)
	FfiConverterInt64INSTANCE.Write(writer, value.CreditsUsed)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
	FfiConverterSequenceMethodUsageINSTANCE.Write(writer, value.MethodsBreakdown)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Requests)
}

type FfiDestroyerEndpointUsage struct{}

func (_ FfiDestroyerEndpointUsage) Destroy(value EndpointUsage) {
	value.Destroy()
}

// ByList form of `EvmAbiFilterTemplate` — carries the ABI inline (the only
// non-list shape this template has) and optionally references a pre-created
// contracts list. Note the wire key is `abiJson`, distinct from the inline
// variant's `abi`.
type EvmAbiFilterByListTemplate struct {
	// JSON-encoded contract ABI used to decode event data.
	AbiJson string
	// Optional name of a pre-created contracts list; when omitted, the ABI
	// is applied to all contracts.
	ContractsListName *string
}

func (r *EvmAbiFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.AbiJson)
	FfiDestroyerOptionalString{}.Destroy(r.ContractsListName)
}

type FfiConverterEvmAbiFilterByListTemplate struct{}

var FfiConverterEvmAbiFilterByListTemplateINSTANCE = FfiConverterEvmAbiFilterByListTemplate{}

func (c FfiConverterEvmAbiFilterByListTemplate) Lift(rb RustBufferI) EvmAbiFilterByListTemplate {
	return LiftFromRustBuffer[EvmAbiFilterByListTemplate](c, rb)
}

func (c FfiConverterEvmAbiFilterByListTemplate) Read(reader io.Reader) EvmAbiFilterByListTemplate {
	return EvmAbiFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmAbiFilterByListTemplate) Lower(value EvmAbiFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmAbiFilterByListTemplate](c, value)
}

func (c FfiConverterEvmAbiFilterByListTemplate) LowerExternal(value EvmAbiFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmAbiFilterByListTemplate](c, value))
}

func (c FfiConverterEvmAbiFilterByListTemplate) Write(writer io.Writer, value EvmAbiFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.AbiJson)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.ContractsListName)
}

type FfiDestroyerEvmAbiFilterByListTemplate struct{}

func (_ FfiDestroyerEvmAbiFilterByListTemplate) Destroy(value EvmAbiFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for an EVM ABI filter: decodes and filters events for a
// set of contracts using a provided ABI.
type EvmAbiFilterTemplate struct {
	// JSON-encoded contract ABI used to decode event data.
	Abi string
	// Contract addresses to watch for events.
	Contracts []string
}

func (r *EvmAbiFilterTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.Abi)
	FfiDestroyerSequenceString{}.Destroy(r.Contracts)
}

type FfiConverterEvmAbiFilterTemplate struct{}

var FfiConverterEvmAbiFilterTemplateINSTANCE = FfiConverterEvmAbiFilterTemplate{}

func (c FfiConverterEvmAbiFilterTemplate) Lift(rb RustBufferI) EvmAbiFilterTemplate {
	return LiftFromRustBuffer[EvmAbiFilterTemplate](c, rb)
}

func (c FfiConverterEvmAbiFilterTemplate) Read(reader io.Reader) EvmAbiFilterTemplate {
	return EvmAbiFilterTemplate{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmAbiFilterTemplate) Lower(value EvmAbiFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmAbiFilterTemplate](c, value)
}

func (c FfiConverterEvmAbiFilterTemplate) LowerExternal(value EvmAbiFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmAbiFilterTemplate](c, value))
}

func (c FfiConverterEvmAbiFilterTemplate) Write(writer io.Writer, value EvmAbiFilterTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.Abi)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Contracts)
}

type FfiDestroyerEvmAbiFilterTemplate struct{}

func (_ FfiDestroyerEvmAbiFilterTemplate) Destroy(value EvmAbiFilterTemplate) {
	value.Destroy()
}

// ByList form of `EvmContractEventsTemplate` — references pre-created
// contract and (optionally) event-hash lists by name. Omitting
// `event_hashes_list_name` matches all events from the listed contracts.
type EvmContractEventsByListTemplate struct {
	// Name of the pre-created contracts list.
	ContractsListName string
	// Optional name of a pre-created event-hashes list; when omitted, all
	// events from the listed contracts match.
	EventHashesListName *string
}

func (r *EvmContractEventsByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.ContractsListName)
	FfiDestroyerOptionalString{}.Destroy(r.EventHashesListName)
}

type FfiConverterEvmContractEventsByListTemplate struct{}

var FfiConverterEvmContractEventsByListTemplateINSTANCE = FfiConverterEvmContractEventsByListTemplate{}

func (c FfiConverterEvmContractEventsByListTemplate) Lift(rb RustBufferI) EvmContractEventsByListTemplate {
	return LiftFromRustBuffer[EvmContractEventsByListTemplate](c, rb)
}

func (c FfiConverterEvmContractEventsByListTemplate) Read(reader io.Reader) EvmContractEventsByListTemplate {
	return EvmContractEventsByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmContractEventsByListTemplate) Lower(value EvmContractEventsByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmContractEventsByListTemplate](c, value)
}

func (c FfiConverterEvmContractEventsByListTemplate) LowerExternal(value EvmContractEventsByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmContractEventsByListTemplate](c, value))
}

func (c FfiConverterEvmContractEventsByListTemplate) Write(writer io.Writer, value EvmContractEventsByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.ContractsListName)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.EventHashesListName)
}

type FfiDestroyerEvmContractEventsByListTemplate struct{}

func (_ FfiDestroyerEvmContractEventsByListTemplate) Destroy(value EvmContractEventsByListTemplate) {
	value.Destroy()
}

// Template arguments for filtering EVM contract events, scoped to a specific
// set of event topic hashes.
type EvmContractEventsTemplate struct {
	// Contract addresses to watch for events.
	Contracts []string
	// Event topic hashes to restrict the filter to specific events.
	EventHashes []string
}

func (r *EvmContractEventsTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Contracts)
	FfiDestroyerSequenceString{}.Destroy(r.EventHashes)
}

type FfiConverterEvmContractEventsTemplate struct{}

var FfiConverterEvmContractEventsTemplateINSTANCE = FfiConverterEvmContractEventsTemplate{}

func (c FfiConverterEvmContractEventsTemplate) Lift(rb RustBufferI) EvmContractEventsTemplate {
	return LiftFromRustBuffer[EvmContractEventsTemplate](c, rb)
}

func (c FfiConverterEvmContractEventsTemplate) Read(reader io.Reader) EvmContractEventsTemplate {
	return EvmContractEventsTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmContractEventsTemplate) Lower(value EvmContractEventsTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmContractEventsTemplate](c, value)
}

func (c FfiConverterEvmContractEventsTemplate) LowerExternal(value EvmContractEventsTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmContractEventsTemplate](c, value))
}

func (c FfiConverterEvmContractEventsTemplate) Write(writer io.Writer, value EvmContractEventsTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Contracts)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.EventHashes)
}

type FfiDestroyerEvmContractEventsTemplate struct{}

func (_ FfiDestroyerEvmContractEventsTemplate) Destroy(value EvmContractEventsTemplate) {
	value.Destroy()
}

// ByList form of `EvmWalletFilterTemplate` — references a pre-created
// wallets list by name instead of inlining the addresses.
type EvmWalletFilterByListTemplate struct {
	// Name of the pre-created wallets list.
	WalletsListName string
}

func (r *EvmWalletFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.WalletsListName)
}

type FfiConverterEvmWalletFilterByListTemplate struct{}

var FfiConverterEvmWalletFilterByListTemplateINSTANCE = FfiConverterEvmWalletFilterByListTemplate{}

func (c FfiConverterEvmWalletFilterByListTemplate) Lift(rb RustBufferI) EvmWalletFilterByListTemplate {
	return LiftFromRustBuffer[EvmWalletFilterByListTemplate](c, rb)
}

func (c FfiConverterEvmWalletFilterByListTemplate) Read(reader io.Reader) EvmWalletFilterByListTemplate {
	return EvmWalletFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmWalletFilterByListTemplate) Lower(value EvmWalletFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmWalletFilterByListTemplate](c, value)
}

func (c FfiConverterEvmWalletFilterByListTemplate) LowerExternal(value EvmWalletFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmWalletFilterByListTemplate](c, value))
}

func (c FfiConverterEvmWalletFilterByListTemplate) Write(writer io.Writer, value EvmWalletFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.WalletsListName)
}

type FfiDestroyerEvmWalletFilterByListTemplate struct{}

func (_ FfiDestroyerEvmWalletFilterByListTemplate) Destroy(value EvmWalletFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for an EVM wallet filter: matches activity for a list of
// wallet addresses.
type EvmWalletFilterTemplate struct {
	// Wallet addresses to match against.
	Wallets []string
}

func (r *EvmWalletFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Wallets)
}

type FfiConverterEvmWalletFilterTemplate struct{}

var FfiConverterEvmWalletFilterTemplateINSTANCE = FfiConverterEvmWalletFilterTemplate{}

func (c FfiConverterEvmWalletFilterTemplate) Lift(rb RustBufferI) EvmWalletFilterTemplate {
	return LiftFromRustBuffer[EvmWalletFilterTemplate](c, rb)
}

func (c FfiConverterEvmWalletFilterTemplate) Read(reader io.Reader) EvmWalletFilterTemplate {
	return EvmWalletFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterEvmWalletFilterTemplate) Lower(value EvmWalletFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[EvmWalletFilterTemplate](c, value)
}

func (c FfiConverterEvmWalletFilterTemplate) LowerExternal(value EvmWalletFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmWalletFilterTemplate](c, value))
}

func (c FfiConverterEvmWalletFilterTemplate) Write(writer io.Writer, value EvmWalletFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Wallets)
}

type FfiDestroyerEvmWalletFilterTemplate struct{}

func (_ FfiDestroyerEvmWalletFilterTemplate) Destroy(value EvmWalletFilterTemplate) {
	value.Destroy()
}

// Parameters for `get_account_metrics`.
type GetAccountMetricsRequest struct {
	// Time period (`hour`, `day`, `week`, or `month`).
	Period string
	// Metric name (e.g. `method_calls_over_time`, `credits_over_time`).
	Metric string
	// Optional percentile for latency metrics (e.g. `p50`, `p95`, `p99`).
	Percentile *string
}

func (r *GetAccountMetricsRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Period)
	FfiDestroyerString{}.Destroy(r.Metric)
	FfiDestroyerOptionalString{}.Destroy(r.Percentile)
}

type FfiConverterGetAccountMetricsRequest struct{}

var FfiConverterGetAccountMetricsRequestINSTANCE = FfiConverterGetAccountMetricsRequest{}

func (c FfiConverterGetAccountMetricsRequest) Lift(rb RustBufferI) GetAccountMetricsRequest {
	return LiftFromRustBuffer[GetAccountMetricsRequest](c, rb)
}

func (c FfiConverterGetAccountMetricsRequest) Read(reader io.Reader) GetAccountMetricsRequest {
	return GetAccountMetricsRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetAccountMetricsRequest) Lower(value GetAccountMetricsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[GetAccountMetricsRequest](c, value)
}

func (c FfiConverterGetAccountMetricsRequest) LowerExternal(value GetAccountMetricsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetAccountMetricsRequest](c, value))
}

func (c FfiConverterGetAccountMetricsRequest) Write(writer io.Writer, value GetAccountMetricsRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Period)
	FfiConverterStringINSTANCE.Write(writer, value.Metric)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Percentile)
}

type FfiDestroyerGetAccountMetricsRequest struct{}

func (_ FfiDestroyerGetAccountMetricsRequest) Destroy(value GetAccountMetricsRequest) {
	value.Destroy()
}

// Response from `get_account_metrics`.
type GetAccountMetricsResponse struct {
	// Metric series returned for the account.
	Data []EndpointMetric
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetAccountMetricsResponse) Destroy() {
	FfiDestroyerSequenceEndpointMetric{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetAccountMetricsResponse struct{}

var FfiConverterGetAccountMetricsResponseINSTANCE = FfiConverterGetAccountMetricsResponse{}

func (c FfiConverterGetAccountMetricsResponse) Lift(rb RustBufferI) GetAccountMetricsResponse {
	return LiftFromRustBuffer[GetAccountMetricsResponse](c, rb)
}

func (c FfiConverterGetAccountMetricsResponse) Read(reader io.Reader) GetAccountMetricsResponse {
	return GetAccountMetricsResponse{
		FfiConverterSequenceEndpointMetricINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetAccountMetricsResponse) Lower(value GetAccountMetricsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetAccountMetricsResponse](c, value)
}

func (c FfiConverterGetAccountMetricsResponse) LowerExternal(value GetAccountMetricsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetAccountMetricsResponse](c, value))
}

func (c FfiConverterGetAccountMetricsResponse) Write(writer io.Writer, value GetAccountMetricsResponse) {
	FfiConverterSequenceEndpointMetricINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetAccountMetricsResponse struct{}

func (_ FfiDestroyerGetAccountMetricsResponse) Destroy(value GetAccountMetricsResponse) {
	value.Destroy()
}

// Parameters for `get_endpoint_logs`.
type GetEndpointLogsRequest struct {
	// Start of the query window (timestamp).
	From string
	// End of the query window (timestamp).
	To string
	// When true, include full request/response payloads in each entry.
	IncludeDetails *bool
	// Maximum number of log entries returned.
	Limit *int32
	// Cursor returned by a previous page; pass to fetch the next page.
	NextAt *string
}

func (r *GetEndpointLogsRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.From)
	FfiDestroyerString{}.Destroy(r.To)
	FfiDestroyerOptionalBool{}.Destroy(r.IncludeDetails)
	FfiDestroyerOptionalInt32{}.Destroy(r.Limit)
	FfiDestroyerOptionalString{}.Destroy(r.NextAt)
}

type FfiConverterGetEndpointLogsRequest struct{}

var FfiConverterGetEndpointLogsRequestINSTANCE = FfiConverterGetEndpointLogsRequest{}

func (c FfiConverterGetEndpointLogsRequest) Lift(rb RustBufferI) GetEndpointLogsRequest {
	return LiftFromRustBuffer[GetEndpointLogsRequest](c, rb)
}

func (c FfiConverterGetEndpointLogsRequest) Read(reader io.Reader) GetEndpointLogsRequest {
	return GetEndpointLogsRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointLogsRequest) Lower(value GetEndpointLogsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointLogsRequest](c, value)
}

func (c FfiConverterGetEndpointLogsRequest) LowerExternal(value GetEndpointLogsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointLogsRequest](c, value))
}

func (c FfiConverterGetEndpointLogsRequest) Write(writer io.Writer, value GetEndpointLogsRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.From)
	FfiConverterStringINSTANCE.Write(writer, value.To)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.IncludeDetails)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NextAt)
}

type FfiDestroyerGetEndpointLogsRequest struct{}

func (_ FfiDestroyerGetEndpointLogsRequest) Destroy(value GetEndpointLogsRequest) {
	value.Destroy()
}

// Response from `get_endpoint_logs`.
type GetEndpointLogsResponse struct {
	// Log entries on the current page.
	Data []EndpointLog
	// Cursor for the next page; `None` when there are no more entries.
	NextAt *string
}

func (r *GetEndpointLogsResponse) Destroy() {
	FfiDestroyerSequenceEndpointLog{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.NextAt)
}

type FfiConverterGetEndpointLogsResponse struct{}

var FfiConverterGetEndpointLogsResponseINSTANCE = FfiConverterGetEndpointLogsResponse{}

func (c FfiConverterGetEndpointLogsResponse) Lift(rb RustBufferI) GetEndpointLogsResponse {
	return LiftFromRustBuffer[GetEndpointLogsResponse](c, rb)
}

func (c FfiConverterGetEndpointLogsResponse) Read(reader io.Reader) GetEndpointLogsResponse {
	return GetEndpointLogsResponse{
		FfiConverterSequenceEndpointLogINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointLogsResponse) Lower(value GetEndpointLogsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointLogsResponse](c, value)
}

func (c FfiConverterGetEndpointLogsResponse) LowerExternal(value GetEndpointLogsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointLogsResponse](c, value))
}

func (c FfiConverterGetEndpointLogsResponse) Write(writer io.Writer, value GetEndpointLogsResponse) {
	FfiConverterSequenceEndpointLogINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NextAt)
}

type FfiDestroyerGetEndpointLogsResponse struct{}

func (_ FfiDestroyerGetEndpointLogsResponse) Destroy(value GetEndpointLogsResponse) {
	value.Destroy()
}

// Parameters for `get_endpoint_metrics`.
type GetEndpointMetricsRequest struct {
	// Time period (`hour`, `day`, `week`, or `month`).
	Period string
	// Metric name (e.g. `method_calls_over_time`, `response_status_breakdown`).
	Metric string
}

func (r *GetEndpointMetricsRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Period)
	FfiDestroyerString{}.Destroy(r.Metric)
}

type FfiConverterGetEndpointMetricsRequest struct{}

var FfiConverterGetEndpointMetricsRequestINSTANCE = FfiConverterGetEndpointMetricsRequest{}

func (c FfiConverterGetEndpointMetricsRequest) Lift(rb RustBufferI) GetEndpointMetricsRequest {
	return LiftFromRustBuffer[GetEndpointMetricsRequest](c, rb)
}

func (c FfiConverterGetEndpointMetricsRequest) Read(reader io.Reader) GetEndpointMetricsRequest {
	return GetEndpointMetricsRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointMetricsRequest) Lower(value GetEndpointMetricsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointMetricsRequest](c, value)
}

func (c FfiConverterGetEndpointMetricsRequest) LowerExternal(value GetEndpointMetricsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointMetricsRequest](c, value))
}

func (c FfiConverterGetEndpointMetricsRequest) Write(writer io.Writer, value GetEndpointMetricsRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Period)
	FfiConverterStringINSTANCE.Write(writer, value.Metric)
}

type FfiDestroyerGetEndpointMetricsRequest struct{}

func (_ FfiDestroyerGetEndpointMetricsRequest) Destroy(value GetEndpointMetricsRequest) {
	value.Destroy()
}

// Response from `get_endpoint_metrics`.
type GetEndpointMetricsResponse struct {
	// Metric series returned for the endpoint.
	Data []EndpointMetric
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetEndpointMetricsResponse) Destroy() {
	FfiDestroyerSequenceEndpointMetric{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetEndpointMetricsResponse struct{}

var FfiConverterGetEndpointMetricsResponseINSTANCE = FfiConverterGetEndpointMetricsResponse{}

func (c FfiConverterGetEndpointMetricsResponse) Lift(rb RustBufferI) GetEndpointMetricsResponse {
	return LiftFromRustBuffer[GetEndpointMetricsResponse](c, rb)
}

func (c FfiConverterGetEndpointMetricsResponse) Read(reader io.Reader) GetEndpointMetricsResponse {
	return GetEndpointMetricsResponse{
		FfiConverterSequenceEndpointMetricINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointMetricsResponse) Lower(value GetEndpointMetricsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointMetricsResponse](c, value)
}

func (c FfiConverterGetEndpointMetricsResponse) LowerExternal(value GetEndpointMetricsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointMetricsResponse](c, value))
}

func (c FfiConverterGetEndpointMetricsResponse) Write(writer io.Writer, value GetEndpointMetricsResponse) {
	FfiConverterSequenceEndpointMetricINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetEndpointMetricsResponse struct{}

func (_ FfiDestroyerGetEndpointMetricsResponse) Destroy(value GetEndpointMetricsResponse) {
	value.Destroy()
}

// Response from `get_endpoint_security`.
type GetEndpointSecurityResponse struct {
	// The endpoint's security configuration.
	Data *EndpointSecurity
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetEndpointSecurityResponse) Destroy() {
	FfiDestroyerOptionalEndpointSecurity{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetEndpointSecurityResponse struct{}

var FfiConverterGetEndpointSecurityResponseINSTANCE = FfiConverterGetEndpointSecurityResponse{}

func (c FfiConverterGetEndpointSecurityResponse) Lift(rb RustBufferI) GetEndpointSecurityResponse {
	return LiftFromRustBuffer[GetEndpointSecurityResponse](c, rb)
}

func (c FfiConverterGetEndpointSecurityResponse) Read(reader io.Reader) GetEndpointSecurityResponse {
	return GetEndpointSecurityResponse{
		FfiConverterOptionalEndpointSecurityINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointSecurityResponse) Lower(value GetEndpointSecurityResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointSecurityResponse](c, value)
}

func (c FfiConverterGetEndpointSecurityResponse) LowerExternal(value GetEndpointSecurityResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointSecurityResponse](c, value))
}

func (c FfiConverterGetEndpointSecurityResponse) Write(writer io.Writer, value GetEndpointSecurityResponse) {
	FfiConverterOptionalEndpointSecurityINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetEndpointSecurityResponse struct{}

func (_ FfiDestroyerGetEndpointSecurityResponse) Destroy(value GetEndpointSecurityResponse) {
	value.Destroy()
}

// Inner data for `get_endpoint_urls` — the http/wss URLs for the endpoint and,
// when the endpoint is multichain, a per-network map of additional URLs.
type GetEndpointUrlsData struct {
	// HTTP RPC URL.
	HttpUrl string
	// WebSocket RPC URL, when available.
	WssUrl *string
	// Per-network URLs for multichain endpoints; `None` for single-chain endpoints.
	MultichainUrls *map[string]EndpointUrl
}

func (r *GetEndpointUrlsData) Destroy() {
	FfiDestroyerString{}.Destroy(r.HttpUrl)
	FfiDestroyerOptionalString{}.Destroy(r.WssUrl)
	FfiDestroyerOptionalMapStringEndpointUrl{}.Destroy(r.MultichainUrls)
}

type FfiConverterGetEndpointUrlsData struct{}

var FfiConverterGetEndpointUrlsDataINSTANCE = FfiConverterGetEndpointUrlsData{}

func (c FfiConverterGetEndpointUrlsData) Lift(rb RustBufferI) GetEndpointUrlsData {
	return LiftFromRustBuffer[GetEndpointUrlsData](c, rb)
}

func (c FfiConverterGetEndpointUrlsData) Read(reader io.Reader) GetEndpointUrlsData {
	return GetEndpointUrlsData{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalMapStringEndpointUrlINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointUrlsData) Lower(value GetEndpointUrlsData) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointUrlsData](c, value)
}

func (c FfiConverterGetEndpointUrlsData) LowerExternal(value GetEndpointUrlsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointUrlsData](c, value))
}

func (c FfiConverterGetEndpointUrlsData) Write(writer io.Writer, value GetEndpointUrlsData) {
	FfiConverterStringINSTANCE.Write(writer, value.HttpUrl)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.WssUrl)
	FfiConverterOptionalMapStringEndpointUrlINSTANCE.Write(writer, value.MultichainUrls)
}

type FfiDestroyerGetEndpointUrlsData struct{}

func (_ FfiDestroyerGetEndpointUrlsData) Destroy(value GetEndpointUrlsData) {
	value.Destroy()
}

// Response from `get_endpoint_urls`.
type GetEndpointUrlsResponse struct {
	// URLs for the endpoint.
	Data *GetEndpointUrlsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetEndpointUrlsResponse) Destroy() {
	FfiDestroyerOptionalGetEndpointUrlsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetEndpointUrlsResponse struct{}

var FfiConverterGetEndpointUrlsResponseINSTANCE = FfiConverterGetEndpointUrlsResponse{}

func (c FfiConverterGetEndpointUrlsResponse) Lift(rb RustBufferI) GetEndpointUrlsResponse {
	return LiftFromRustBuffer[GetEndpointUrlsResponse](c, rb)
}

func (c FfiConverterGetEndpointUrlsResponse) Read(reader io.Reader) GetEndpointUrlsResponse {
	return GetEndpointUrlsResponse{
		FfiConverterOptionalGetEndpointUrlsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointUrlsResponse) Lower(value GetEndpointUrlsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointUrlsResponse](c, value)
}

func (c FfiConverterGetEndpointUrlsResponse) LowerExternal(value GetEndpointUrlsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointUrlsResponse](c, value))
}

func (c FfiConverterGetEndpointUrlsResponse) Write(writer io.Writer, value GetEndpointUrlsResponse) {
	FfiConverterOptionalGetEndpointUrlsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetEndpointUrlsResponse struct{}

func (_ FfiDestroyerGetEndpointUrlsResponse) Destroy(value GetEndpointUrlsResponse) {
	value.Destroy()
}

// Parameters for `get_endpoints`.
type GetEndpointsRequest struct {
	// Maximum number of endpoints returned.
	Limit *int32
	// Starting index into the result set.
	Offset *int32
	// Search by subdomain or label.
	Search *string
	// Field to sort results by.
	SortBy *string
	// Sort direction (`asc` or `desc`).
	SortDirection *string
	// Filter results to endpoints on these networks.
	Networks *[]string
	// Filter results to endpoints in these statuses.
	Statuses *[]string
	// Filter results by label.
	Labels *[]string
	// When true, return only dedicated endpoints.
	Dedicated *bool
	// When true, return only flat-rate endpoints.
	IsFlatRate *bool
	// Filter results by associated tag ids.
	TagIds *[]int32
	// Filter results by associated tag labels.
	TagLabels *[]string
}

func (r *GetEndpointsRequest) Destroy() {
	FfiDestroyerOptionalInt32{}.Destroy(r.Limit)
	FfiDestroyerOptionalInt32{}.Destroy(r.Offset)
	FfiDestroyerOptionalString{}.Destroy(r.Search)
	FfiDestroyerOptionalString{}.Destroy(r.SortBy)
	FfiDestroyerOptionalString{}.Destroy(r.SortDirection)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.Networks)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.Statuses)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.Labels)
	FfiDestroyerOptionalBool{}.Destroy(r.Dedicated)
	FfiDestroyerOptionalBool{}.Destroy(r.IsFlatRate)
	FfiDestroyerOptionalSequenceInt32{}.Destroy(r.TagIds)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.TagLabels)
}

type FfiConverterGetEndpointsRequest struct{}

var FfiConverterGetEndpointsRequestINSTANCE = FfiConverterGetEndpointsRequest{}

func (c FfiConverterGetEndpointsRequest) Lift(rb RustBufferI) GetEndpointsRequest {
	return LiftFromRustBuffer[GetEndpointsRequest](c, rb)
}

func (c FfiConverterGetEndpointsRequest) Read(reader io.Reader) GetEndpointsRequest {
	return GetEndpointsRequest{
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalSequenceInt32INSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointsRequest) Lower(value GetEndpointsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointsRequest](c, value)
}

func (c FfiConverterGetEndpointsRequest) LowerExternal(value GetEndpointsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointsRequest](c, value))
}

func (c FfiConverterGetEndpointsRequest) Write(writer io.Writer, value GetEndpointsRequest) {
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Offset)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Search)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.SortBy)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.SortDirection)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Networks)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Statuses)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Labels)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Dedicated)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.IsFlatRate)
	FfiConverterOptionalSequenceInt32INSTANCE.Write(writer, value.TagIds)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.TagLabels)
}

type FfiDestroyerGetEndpointsRequest struct{}

func (_ FfiDestroyerGetEndpointsRequest) Destroy(value GetEndpointsRequest) {
	value.Destroy()
}

// Response from `get_endpoints`.
type GetEndpointsResponse struct {
	// Endpoints on the current page.
	Data []Endpoint
	// Pagination metadata for the response.
	Pagination *Pagination
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetEndpointsResponse) Destroy() {
	FfiDestroyerSequenceEndpoint{}.Destroy(r.Data)
	FfiDestroyerOptionalPagination{}.Destroy(r.Pagination)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetEndpointsResponse struct{}

var FfiConverterGetEndpointsResponseINSTANCE = FfiConverterGetEndpointsResponse{}

func (c FfiConverterGetEndpointsResponse) Lift(rb RustBufferI) GetEndpointsResponse {
	return LiftFromRustBuffer[GetEndpointsResponse](c, rb)
}

func (c FfiConverterGetEndpointsResponse) Read(reader io.Reader) GetEndpointsResponse {
	return GetEndpointsResponse{
		FfiConverterSequenceEndpointINSTANCE.Read(reader),
		FfiConverterOptionalPaginationINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetEndpointsResponse) Lower(value GetEndpointsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetEndpointsResponse](c, value)
}

func (c FfiConverterGetEndpointsResponse) LowerExternal(value GetEndpointsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetEndpointsResponse](c, value))
}

func (c FfiConverterGetEndpointsResponse) Write(writer io.Writer, value GetEndpointsResponse) {
	FfiConverterSequenceEndpointINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalPaginationINSTANCE.Write(writer, value.Pagination)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetEndpointsResponse struct{}

func (_ FfiDestroyerGetEndpointsResponse) Destroy(value GetEndpointsResponse) {
	value.Destroy()
}

// Inner data for `get_list` responses.
type GetListData struct {
	// Items in the list on the current page.
	Items []string
}

func (r *GetListData) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Items)
}

type FfiConverterGetListData struct{}

var FfiConverterGetListDataINSTANCE = FfiConverterGetListData{}

func (c FfiConverterGetListData) Lift(rb RustBufferI) GetListData {
	return LiftFromRustBuffer[GetListData](c, rb)
}

func (c FfiConverterGetListData) Read(reader io.Reader) GetListData {
	return GetListData{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListData) Lower(value GetListData) C.RustBuffer {
	return LowerIntoRustBuffer[GetListData](c, value)
}

func (c FfiConverterGetListData) LowerExternal(value GetListData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListData](c, value))
}

func (c FfiConverterGetListData) Write(writer io.Writer, value GetListData) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Items)
}

type FfiDestroyerGetListData struct{}

func (_ FfiDestroyerGetListData) Destroy(value GetListData) {
	value.Destroy()
}

// Parameters for `get_list`.
type GetListParams struct {
	// Maximum number of items returned.
	Limit *int64
	// Cursor returned by a previous page; pass to fetch the next page.
	Cursor *string
}

func (r *GetListParams) Destroy() {
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalString{}.Destroy(r.Cursor)
}

type FfiConverterGetListParams struct{}

var FfiConverterGetListParamsINSTANCE = FfiConverterGetListParams{}

func (c FfiConverterGetListParams) Lift(rb RustBufferI) GetListParams {
	return LiftFromRustBuffer[GetListParams](c, rb)
}

func (c FfiConverterGetListParams) Read(reader io.Reader) GetListParams {
	return GetListParams{
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListParams) Lower(value GetListParams) C.RustBuffer {
	return LowerIntoRustBuffer[GetListParams](c, value)
}

func (c FfiConverterGetListParams) LowerExternal(value GetListParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListParams](c, value))
}

func (c FfiConverterGetListParams) Write(writer io.Writer, value GetListParams) {
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetListParams struct{}

func (_ FfiDestroyerGetListParams) Destroy(value GetListParams) {
	value.Destroy()
}

// Response from `get_list`.
type GetListResponse struct {
	// Items for the list on the current page.
	Data GetListData
	// Cursor for the next page; empty string when there are no more pages.
	Cursor string
}

func (r *GetListResponse) Destroy() {
	FfiDestroyerGetListData{}.Destroy(r.Data)
	FfiDestroyerString{}.Destroy(r.Cursor)
}

type FfiConverterGetListResponse struct{}

var FfiConverterGetListResponseINSTANCE = FfiConverterGetListResponse{}

func (c FfiConverterGetListResponse) Lift(rb RustBufferI) GetListResponse {
	return LiftFromRustBuffer[GetListResponse](c, rb)
}

func (c FfiConverterGetListResponse) Read(reader io.Reader) GetListResponse {
	return GetListResponse{
		FfiConverterGetListDataINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListResponse) Lower(value GetListResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetListResponse](c, value)
}

func (c FfiConverterGetListResponse) LowerExternal(value GetListResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListResponse](c, value))
}

func (c FfiConverterGetListResponse) Write(writer io.Writer, value GetListResponse) {
	FfiConverterGetListDataINSTANCE.Write(writer, value.Data)
	FfiConverterStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetListResponse struct{}

func (_ FfiDestroyerGetListResponse) Destroy(value GetListResponse) {
	value.Destroy()
}

// Inner data for `get_lists` responses.
type GetListsData struct {
	// List keys on the current page.
	Keys []string
}

func (r *GetListsData) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Keys)
}

type FfiConverterGetListsData struct{}

var FfiConverterGetListsDataINSTANCE = FfiConverterGetListsData{}

func (c FfiConverterGetListsData) Lift(rb RustBufferI) GetListsData {
	return LiftFromRustBuffer[GetListsData](c, rb)
}

func (c FfiConverterGetListsData) Read(reader io.Reader) GetListsData {
	return GetListsData{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListsData) Lower(value GetListsData) C.RustBuffer {
	return LowerIntoRustBuffer[GetListsData](c, value)
}

func (c FfiConverterGetListsData) LowerExternal(value GetListsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListsData](c, value))
}

func (c FfiConverterGetListsData) Write(writer io.Writer, value GetListsData) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Keys)
}

type FfiDestroyerGetListsData struct{}

func (_ FfiDestroyerGetListsData) Destroy(value GetListsData) {
	value.Destroy()
}

// Parameters for `get_lists`.
type GetListsParams struct {
	// Maximum number of list keys returned.
	Limit *int64
	// Cursor returned by a previous page; pass to fetch the next page.
	Cursor *string
}

func (r *GetListsParams) Destroy() {
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalString{}.Destroy(r.Cursor)
}

type FfiConverterGetListsParams struct{}

var FfiConverterGetListsParamsINSTANCE = FfiConverterGetListsParams{}

func (c FfiConverterGetListsParams) Lift(rb RustBufferI) GetListsParams {
	return LiftFromRustBuffer[GetListsParams](c, rb)
}

func (c FfiConverterGetListsParams) Read(reader io.Reader) GetListsParams {
	return GetListsParams{
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListsParams) Lower(value GetListsParams) C.RustBuffer {
	return LowerIntoRustBuffer[GetListsParams](c, value)
}

func (c FfiConverterGetListsParams) LowerExternal(value GetListsParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListsParams](c, value))
}

func (c FfiConverterGetListsParams) Write(writer io.Writer, value GetListsParams) {
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetListsParams struct{}

func (_ FfiDestroyerGetListsParams) Destroy(value GetListsParams) {
	value.Destroy()
}

// Response from `get_lists`.
type GetListsResponse struct {
	// List keys on the current page.
	Data GetListsData
	// Cursor for the next page; empty string when there are no more pages.
	Cursor string
}

func (r *GetListsResponse) Destroy() {
	FfiDestroyerGetListsData{}.Destroy(r.Data)
	FfiDestroyerString{}.Destroy(r.Cursor)
}

type FfiConverterGetListsResponse struct{}

var FfiConverterGetListsResponseINSTANCE = FfiConverterGetListsResponse{}

func (c FfiConverterGetListsResponse) Lift(rb RustBufferI) GetListsResponse {
	return LiftFromRustBuffer[GetListsResponse](c, rb)
}

func (c FfiConverterGetListsResponse) Read(reader io.Reader) GetListsResponse {
	return GetListsResponse{
		FfiConverterGetListsDataINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetListsResponse) Lower(value GetListsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetListsResponse](c, value)
}

func (c FfiConverterGetListsResponse) LowerExternal(value GetListsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetListsResponse](c, value))
}

func (c FfiConverterGetListsResponse) Write(writer io.Writer, value GetListsResponse) {
	FfiConverterGetListsDataINSTANCE.Write(writer, value.Data)
	FfiConverterStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetListsResponse struct{}

func (_ FfiDestroyerGetListsResponse) Destroy(value GetListsResponse) {
	value.Destroy()
}

// Response from `get_log_details`.
type GetLogDetailsResponse struct {
	// Raw request and response payloads for the log entry.
	Data *LogDetails
}

func (r *GetLogDetailsResponse) Destroy() {
	FfiDestroyerOptionalLogDetails{}.Destroy(r.Data)
}

type FfiConverterGetLogDetailsResponse struct{}

var FfiConverterGetLogDetailsResponseINSTANCE = FfiConverterGetLogDetailsResponse{}

func (c FfiConverterGetLogDetailsResponse) Lift(rb RustBufferI) GetLogDetailsResponse {
	return LiftFromRustBuffer[GetLogDetailsResponse](c, rb)
}

func (c FfiConverterGetLogDetailsResponse) Read(reader io.Reader) GetLogDetailsResponse {
	return GetLogDetailsResponse{
		FfiConverterOptionalLogDetailsINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetLogDetailsResponse) Lower(value GetLogDetailsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetLogDetailsResponse](c, value)
}

func (c FfiConverterGetLogDetailsResponse) LowerExternal(value GetLogDetailsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetLogDetailsResponse](c, value))
}

func (c FfiConverterGetLogDetailsResponse) Write(writer io.Writer, value GetLogDetailsResponse) {
	FfiConverterOptionalLogDetailsINSTANCE.Write(writer, value.Data)
}

type FfiDestroyerGetLogDetailsResponse struct{}

func (_ FfiDestroyerGetLogDetailsResponse) Destroy(value GetLogDetailsResponse) {
	value.Destroy()
}

// Inner data for `get_method_rate_limits`.
type GetMethodRateLimitsData struct {
	// Rate limiters configured on the endpoint.
	RateLimiters []MethodRateLimiter
}

func (r *GetMethodRateLimitsData) Destroy() {
	FfiDestroyerSequenceMethodRateLimiter{}.Destroy(r.RateLimiters)
}

type FfiConverterGetMethodRateLimitsData struct{}

var FfiConverterGetMethodRateLimitsDataINSTANCE = FfiConverterGetMethodRateLimitsData{}

func (c FfiConverterGetMethodRateLimitsData) Lift(rb RustBufferI) GetMethodRateLimitsData {
	return LiftFromRustBuffer[GetMethodRateLimitsData](c, rb)
}

func (c FfiConverterGetMethodRateLimitsData) Read(reader io.Reader) GetMethodRateLimitsData {
	return GetMethodRateLimitsData{
		FfiConverterSequenceMethodRateLimiterINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetMethodRateLimitsData) Lower(value GetMethodRateLimitsData) C.RustBuffer {
	return LowerIntoRustBuffer[GetMethodRateLimitsData](c, value)
}

func (c FfiConverterGetMethodRateLimitsData) LowerExternal(value GetMethodRateLimitsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetMethodRateLimitsData](c, value))
}

func (c FfiConverterGetMethodRateLimitsData) Write(writer io.Writer, value GetMethodRateLimitsData) {
	FfiConverterSequenceMethodRateLimiterINSTANCE.Write(writer, value.RateLimiters)
}

type FfiDestroyerGetMethodRateLimitsData struct{}

func (_ FfiDestroyerGetMethodRateLimitsData) Destroy(value GetMethodRateLimitsData) {
	value.Destroy()
}

// Response from `get_method_rate_limits`.
type GetMethodRateLimitsResponse struct {
	// Rate limiters payload.
	Data *GetMethodRateLimitsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetMethodRateLimitsResponse) Destroy() {
	FfiDestroyerOptionalGetMethodRateLimitsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetMethodRateLimitsResponse struct{}

var FfiConverterGetMethodRateLimitsResponseINSTANCE = FfiConverterGetMethodRateLimitsResponse{}

func (c FfiConverterGetMethodRateLimitsResponse) Lift(rb RustBufferI) GetMethodRateLimitsResponse {
	return LiftFromRustBuffer[GetMethodRateLimitsResponse](c, rb)
}

func (c FfiConverterGetMethodRateLimitsResponse) Read(reader io.Reader) GetMethodRateLimitsResponse {
	return GetMethodRateLimitsResponse{
		FfiConverterOptionalGetMethodRateLimitsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetMethodRateLimitsResponse) Lower(value GetMethodRateLimitsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetMethodRateLimitsResponse](c, value)
}

func (c FfiConverterGetMethodRateLimitsResponse) LowerExternal(value GetMethodRateLimitsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetMethodRateLimitsResponse](c, value))
}

func (c FfiConverterGetMethodRateLimitsResponse) Write(writer io.Writer, value GetMethodRateLimitsResponse) {
	FfiConverterOptionalGetMethodRateLimitsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetMethodRateLimitsResponse struct{}

func (_ FfiDestroyerGetMethodRateLimitsResponse) Destroy(value GetMethodRateLimitsResponse) {
	value.Destroy()
}

// Inner data for `get_rate_limits`.
type GetRateLimitsData struct {
	// One row per enforced bucket.
	RateLimits []RateLimitEntry
}

func (r *GetRateLimitsData) Destroy() {
	FfiDestroyerSequenceRateLimitEntry{}.Destroy(r.RateLimits)
}

type FfiConverterGetRateLimitsData struct{}

var FfiConverterGetRateLimitsDataINSTANCE = FfiConverterGetRateLimitsData{}

func (c FfiConverterGetRateLimitsData) Lift(rb RustBufferI) GetRateLimitsData {
	return LiftFromRustBuffer[GetRateLimitsData](c, rb)
}

func (c FfiConverterGetRateLimitsData) Read(reader io.Reader) GetRateLimitsData {
	return GetRateLimitsData{
		FfiConverterSequenceRateLimitEntryINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetRateLimitsData) Lower(value GetRateLimitsData) C.RustBuffer {
	return LowerIntoRustBuffer[GetRateLimitsData](c, value)
}

func (c FfiConverterGetRateLimitsData) LowerExternal(value GetRateLimitsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetRateLimitsData](c, value))
}

func (c FfiConverterGetRateLimitsData) Write(writer io.Writer, value GetRateLimitsData) {
	FfiConverterSequenceRateLimitEntryINSTANCE.Write(writer, value.RateLimits)
}

type FfiDestroyerGetRateLimitsData struct{}

func (_ FfiDestroyerGetRateLimitsData) Destroy(value GetRateLimitsData) {
	value.Destroy()
}

// Response from `get_rate_limits`.
type GetRateLimitsResponse struct {
	// Rate-limit rows with their source.
	Data *GetRateLimitsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetRateLimitsResponse) Destroy() {
	FfiDestroyerOptionalGetRateLimitsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetRateLimitsResponse struct{}

var FfiConverterGetRateLimitsResponseINSTANCE = FfiConverterGetRateLimitsResponse{}

func (c FfiConverterGetRateLimitsResponse) Lift(rb RustBufferI) GetRateLimitsResponse {
	return LiftFromRustBuffer[GetRateLimitsResponse](c, rb)
}

func (c FfiConverterGetRateLimitsResponse) Read(reader io.Reader) GetRateLimitsResponse {
	return GetRateLimitsResponse{
		FfiConverterOptionalGetRateLimitsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetRateLimitsResponse) Lower(value GetRateLimitsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetRateLimitsResponse](c, value)
}

func (c FfiConverterGetRateLimitsResponse) LowerExternal(value GetRateLimitsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetRateLimitsResponse](c, value))
}

func (c FfiConverterGetRateLimitsResponse) Write(writer io.Writer, value GetRateLimitsResponse) {
	FfiConverterOptionalGetRateLimitsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetRateLimitsResponse struct{}

func (_ FfiDestroyerGetRateLimitsResponse) Destroy(value GetRateLimitsResponse) {
	value.Destroy()
}

// Response from `get_security_options`.
type GetSecurityOptionsResponse struct {
	// Security options on the endpoint.
	Data []SecurityOption
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetSecurityOptionsResponse) Destroy() {
	FfiDestroyerSequenceSecurityOption{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetSecurityOptionsResponse struct{}

var FfiConverterGetSecurityOptionsResponseINSTANCE = FfiConverterGetSecurityOptionsResponse{}

func (c FfiConverterGetSecurityOptionsResponse) Lift(rb RustBufferI) GetSecurityOptionsResponse {
	return LiftFromRustBuffer[GetSecurityOptionsResponse](c, rb)
}

func (c FfiConverterGetSecurityOptionsResponse) Read(reader io.Reader) GetSecurityOptionsResponse {
	return GetSecurityOptionsResponse{
		FfiConverterSequenceSecurityOptionINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetSecurityOptionsResponse) Lower(value GetSecurityOptionsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetSecurityOptionsResponse](c, value)
}

func (c FfiConverterGetSecurityOptionsResponse) LowerExternal(value GetSecurityOptionsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetSecurityOptionsResponse](c, value))
}

func (c FfiConverterGetSecurityOptionsResponse) Write(writer io.Writer, value GetSecurityOptionsResponse) {
	FfiConverterSequenceSecurityOptionINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetSecurityOptionsResponse struct{}

func (_ FfiDestroyerGetSecurityOptionsResponse) Destroy(value GetSecurityOptionsResponse) {
	value.Destroy()
}

// Response from `get_set`.
type GetSetResponse struct {
	// Stored string value.
	Value string
}

func (r *GetSetResponse) Destroy() {
	FfiDestroyerString{}.Destroy(r.Value)
}

type FfiConverterGetSetResponse struct{}

var FfiConverterGetSetResponseINSTANCE = FfiConverterGetSetResponse{}

func (c FfiConverterGetSetResponse) Lift(rb RustBufferI) GetSetResponse {
	return LiftFromRustBuffer[GetSetResponse](c, rb)
}

func (c FfiConverterGetSetResponse) Read(reader io.Reader) GetSetResponse {
	return GetSetResponse{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetSetResponse) Lower(value GetSetResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetSetResponse](c, value)
}

func (c FfiConverterGetSetResponse) LowerExternal(value GetSetResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetSetResponse](c, value))
}

func (c FfiConverterGetSetResponse) Write(writer io.Writer, value GetSetResponse) {
	FfiConverterStringINSTANCE.Write(writer, value.Value)
}

type FfiDestroyerGetSetResponse struct{}

func (_ FfiDestroyerGetSetResponse) Destroy(value GetSetResponse) {
	value.Destroy()
}

// Parameters for `get_sets`.
type GetSetsParams struct {
	// Maximum number of entries returned.
	Limit *int64
	// Cursor returned by a previous page; pass to fetch the next page.
	Cursor *string
}

func (r *GetSetsParams) Destroy() {
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalString{}.Destroy(r.Cursor)
}

type FfiConverterGetSetsParams struct{}

var FfiConverterGetSetsParamsINSTANCE = FfiConverterGetSetsParams{}

func (c FfiConverterGetSetsParams) Lift(rb RustBufferI) GetSetsParams {
	return LiftFromRustBuffer[GetSetsParams](c, rb)
}

func (c FfiConverterGetSetsParams) Read(reader io.Reader) GetSetsParams {
	return GetSetsParams{
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetSetsParams) Lower(value GetSetsParams) C.RustBuffer {
	return LowerIntoRustBuffer[GetSetsParams](c, value)
}

func (c FfiConverterGetSetsParams) LowerExternal(value GetSetsParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetSetsParams](c, value))
}

func (c FfiConverterGetSetsParams) Write(writer io.Writer, value GetSetsParams) {
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetSetsParams struct{}

func (_ FfiDestroyerGetSetsParams) Destroy(value GetSetsParams) {
	value.Destroy()
}

// Response from `get_sets`.
type GetSetsResponse struct {
	// Key/value entries on the current page.
	Data []KvSetEntry
	// Cursor for the next page; empty string when there are no more pages.
	Cursor string
}

func (r *GetSetsResponse) Destroy() {
	FfiDestroyerSequenceKvSetEntry{}.Destroy(r.Data)
	FfiDestroyerString{}.Destroy(r.Cursor)
}

type FfiConverterGetSetsResponse struct{}

var FfiConverterGetSetsResponseINSTANCE = FfiConverterGetSetsResponse{}

func (c FfiConverterGetSetsResponse) Lift(rb RustBufferI) GetSetsResponse {
	return LiftFromRustBuffer[GetSetsResponse](c, rb)
}

func (c FfiConverterGetSetsResponse) Read(reader io.Reader) GetSetsResponse {
	return GetSetsResponse{
		FfiConverterSequenceKvSetEntryINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetSetsResponse) Lower(value GetSetsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetSetsResponse](c, value)
}

func (c FfiConverterGetSetsResponse) LowerExternal(value GetSetsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetSetsResponse](c, value))
}

func (c FfiConverterGetSetsResponse) Write(writer io.Writer, value GetSetsResponse) {
	FfiConverterSequenceKvSetEntryINSTANCE.Write(writer, value.Data)
	FfiConverterStringINSTANCE.Write(writer, value.Cursor)
}

type FfiDestroyerGetSetsResponse struct{}

func (_ FfiDestroyerGetSetsResponse) Destroy(value GetSetsResponse) {
	value.Destroy()
}

// Response from `get_team`.
type GetTeamResponse struct {
	// The team's full detail.
	Data *TeamDetail
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetTeamResponse) Destroy() {
	FfiDestroyerOptionalTeamDetail{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetTeamResponse struct{}

var FfiConverterGetTeamResponseINSTANCE = FfiConverterGetTeamResponse{}

func (c FfiConverterGetTeamResponse) Lift(rb RustBufferI) GetTeamResponse {
	return LiftFromRustBuffer[GetTeamResponse](c, rb)
}

func (c FfiConverterGetTeamResponse) Read(reader io.Reader) GetTeamResponse {
	return GetTeamResponse{
		FfiConverterOptionalTeamDetailINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetTeamResponse) Lower(value GetTeamResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetTeamResponse](c, value)
}

func (c FfiConverterGetTeamResponse) LowerExternal(value GetTeamResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetTeamResponse](c, value))
}

func (c FfiConverterGetTeamResponse) Write(writer io.Writer, value GetTeamResponse) {
	FfiConverterOptionalTeamDetailINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetTeamResponse struct{}

func (_ FfiDestroyerGetTeamResponse) Destroy(value GetTeamResponse) {
	value.Destroy()
}

// Response from `get_usage_by_chain`.
type GetUsageByChainResponse struct {
	// Per-chain usage payload.
	Data *UsageByChainData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetUsageByChainResponse) Destroy() {
	FfiDestroyerOptionalUsageByChainData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetUsageByChainResponse struct{}

var FfiConverterGetUsageByChainResponseINSTANCE = FfiConverterGetUsageByChainResponse{}

func (c FfiConverterGetUsageByChainResponse) Lift(rb RustBufferI) GetUsageByChainResponse {
	return LiftFromRustBuffer[GetUsageByChainResponse](c, rb)
}

func (c FfiConverterGetUsageByChainResponse) Read(reader io.Reader) GetUsageByChainResponse {
	return GetUsageByChainResponse{
		FfiConverterOptionalUsageByChainDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageByChainResponse) Lower(value GetUsageByChainResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageByChainResponse](c, value)
}

func (c FfiConverterGetUsageByChainResponse) LowerExternal(value GetUsageByChainResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageByChainResponse](c, value))
}

func (c FfiConverterGetUsageByChainResponse) Write(writer io.Writer, value GetUsageByChainResponse) {
	FfiConverterOptionalUsageByChainDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetUsageByChainResponse struct{}

func (_ FfiDestroyerGetUsageByChainResponse) Destroy(value GetUsageByChainResponse) {
	value.Destroy()
}

// Response from `get_usage_by_endpoint`.
type GetUsageByEndpointResponse struct {
	// Per-endpoint usage payload.
	Data *UsageByEndpointData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetUsageByEndpointResponse) Destroy() {
	FfiDestroyerOptionalUsageByEndpointData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetUsageByEndpointResponse struct{}

var FfiConverterGetUsageByEndpointResponseINSTANCE = FfiConverterGetUsageByEndpointResponse{}

func (c FfiConverterGetUsageByEndpointResponse) Lift(rb RustBufferI) GetUsageByEndpointResponse {
	return LiftFromRustBuffer[GetUsageByEndpointResponse](c, rb)
}

func (c FfiConverterGetUsageByEndpointResponse) Read(reader io.Reader) GetUsageByEndpointResponse {
	return GetUsageByEndpointResponse{
		FfiConverterOptionalUsageByEndpointDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageByEndpointResponse) Lower(value GetUsageByEndpointResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageByEndpointResponse](c, value)
}

func (c FfiConverterGetUsageByEndpointResponse) LowerExternal(value GetUsageByEndpointResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageByEndpointResponse](c, value))
}

func (c FfiConverterGetUsageByEndpointResponse) Write(writer io.Writer, value GetUsageByEndpointResponse) {
	FfiConverterOptionalUsageByEndpointDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetUsageByEndpointResponse struct{}

func (_ FfiDestroyerGetUsageByEndpointResponse) Destroy(value GetUsageByEndpointResponse) {
	value.Destroy()
}

// Response from `get_usage_by_method`.
type GetUsageByMethodResponse struct {
	// Per-method usage payload.
	Data *UsageByMethodData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetUsageByMethodResponse) Destroy() {
	FfiDestroyerOptionalUsageByMethodData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetUsageByMethodResponse struct{}

var FfiConverterGetUsageByMethodResponseINSTANCE = FfiConverterGetUsageByMethodResponse{}

func (c FfiConverterGetUsageByMethodResponse) Lift(rb RustBufferI) GetUsageByMethodResponse {
	return LiftFromRustBuffer[GetUsageByMethodResponse](c, rb)
}

func (c FfiConverterGetUsageByMethodResponse) Read(reader io.Reader) GetUsageByMethodResponse {
	return GetUsageByMethodResponse{
		FfiConverterOptionalUsageByMethodDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageByMethodResponse) Lower(value GetUsageByMethodResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageByMethodResponse](c, value)
}

func (c FfiConverterGetUsageByMethodResponse) LowerExternal(value GetUsageByMethodResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageByMethodResponse](c, value))
}

func (c FfiConverterGetUsageByMethodResponse) Write(writer io.Writer, value GetUsageByMethodResponse) {
	FfiConverterOptionalUsageByMethodDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetUsageByMethodResponse struct{}

func (_ FfiDestroyerGetUsageByMethodResponse) Destroy(value GetUsageByMethodResponse) {
	value.Destroy()
}

// Response from `get_usage_by_tag`.
type GetUsageByTagResponse struct {
	// Per-tag usage payload.
	Data *UsageByTagData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetUsageByTagResponse) Destroy() {
	FfiDestroyerOptionalUsageByTagData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetUsageByTagResponse struct{}

var FfiConverterGetUsageByTagResponseINSTANCE = FfiConverterGetUsageByTagResponse{}

func (c FfiConverterGetUsageByTagResponse) Lift(rb RustBufferI) GetUsageByTagResponse {
	return LiftFromRustBuffer[GetUsageByTagResponse](c, rb)
}

func (c FfiConverterGetUsageByTagResponse) Read(reader io.Reader) GetUsageByTagResponse {
	return GetUsageByTagResponse{
		FfiConverterOptionalUsageByTagDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageByTagResponse) Lower(value GetUsageByTagResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageByTagResponse](c, value)
}

func (c FfiConverterGetUsageByTagResponse) LowerExternal(value GetUsageByTagResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageByTagResponse](c, value))
}

func (c FfiConverterGetUsageByTagResponse) Write(writer io.Writer, value GetUsageByTagResponse) {
	FfiConverterOptionalUsageByTagDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetUsageByTagResponse struct{}

func (_ FfiDestroyerGetUsageByTagResponse) Destroy(value GetUsageByTagResponse) {
	value.Destroy()
}

// Parameters for the account usage methods (`get_usage`, `get_usage_by_*`).
// Both bounds are optional; omit for account-to-date totals.
type GetUsageRequest struct {
	// Start of the query window (Unix timestamp).
	StartTime *int64
	// End of the query window (Unix timestamp).
	EndTime *int64
}

func (r *GetUsageRequest) Destroy() {
	FfiDestroyerOptionalInt64{}.Destroy(r.StartTime)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndTime)
}

type FfiConverterGetUsageRequest struct{}

var FfiConverterGetUsageRequestINSTANCE = FfiConverterGetUsageRequest{}

func (c FfiConverterGetUsageRequest) Lift(rb RustBufferI) GetUsageRequest {
	return LiftFromRustBuffer[GetUsageRequest](c, rb)
}

func (c FfiConverterGetUsageRequest) Read(reader io.Reader) GetUsageRequest {
	return GetUsageRequest{
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageRequest) Lower(value GetUsageRequest) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageRequest](c, value)
}

func (c FfiConverterGetUsageRequest) LowerExternal(value GetUsageRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageRequest](c, value))
}

func (c FfiConverterGetUsageRequest) Write(writer io.Writer, value GetUsageRequest) {
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerGetUsageRequest struct{}

func (_ FfiDestroyerGetUsageRequest) Destroy(value GetUsageRequest) {
	value.Destroy()
}

// Response from `get_usage`.
type GetUsageResponse struct {
	// Aggregate usage payload.
	Data *UsageData
	// Error message when the request did not succeed.
	Error *string
}

func (r *GetUsageResponse) Destroy() {
	FfiDestroyerOptionalUsageData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterGetUsageResponse struct{}

var FfiConverterGetUsageResponseINSTANCE = FfiConverterGetUsageResponse{}

func (c FfiConverterGetUsageResponse) Lift(rb RustBufferI) GetUsageResponse {
	return LiftFromRustBuffer[GetUsageResponse](c, rb)
}

func (c FfiConverterGetUsageResponse) Read(reader io.Reader) GetUsageResponse {
	return GetUsageResponse{
		FfiConverterOptionalUsageDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterGetUsageResponse) Lower(value GetUsageResponse) C.RustBuffer {
	return LowerIntoRustBuffer[GetUsageResponse](c, value)
}

func (c FfiConverterGetUsageResponse) LowerExternal(value GetUsageResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetUsageResponse](c, value))
}

func (c FfiConverterGetUsageResponse) Write(writer io.Writer, value GetUsageResponse) {
	FfiConverterOptionalUsageDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerGetUsageResponse struct{}

func (_ FfiDestroyerGetUsageResponse) Destroy(value GetUsageResponse) {
	value.Destroy()
}

// Parameters for `list_webhooks`.
type GetWebhooksParams struct {
	// Maximum number of webhooks returned.
	Limit *int64
	// Starting index into the result set.
	Offset *int64
}

func (r *GetWebhooksParams) Destroy() {
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalInt64{}.Destroy(r.Offset)
}

type FfiConverterGetWebhooksParams struct{}

var FfiConverterGetWebhooksParamsINSTANCE = FfiConverterGetWebhooksParams{}

func (c FfiConverterGetWebhooksParams) Lift(rb RustBufferI) GetWebhooksParams {
	return LiftFromRustBuffer[GetWebhooksParams](c, rb)
}

func (c FfiConverterGetWebhooksParams) Read(reader io.Reader) GetWebhooksParams {
	return GetWebhooksParams{
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterGetWebhooksParams) Lower(value GetWebhooksParams) C.RustBuffer {
	return LowerIntoRustBuffer[GetWebhooksParams](c, value)
}

func (c FfiConverterGetWebhooksParams) LowerExternal(value GetWebhooksParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[GetWebhooksParams](c, value))
}

func (c FfiConverterGetWebhooksParams) Write(writer io.Writer, value GetWebhooksParams) {
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Offset)
}

type FfiDestroyerGetWebhooksParams struct{}

func (_ FfiDestroyerGetWebhooksParams) Destroy(value GetWebhooksParams) {
	value.Destroy()
}

// ByList form of `HyperliquidWalletEventsFilterTemplate`.
type HyperliquidWalletEventsFilterByListTemplate struct {
	// Name of the pre-created wallets list.
	WalletsListName string
}

func (r *HyperliquidWalletEventsFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.WalletsListName)
}

type FfiConverterHyperliquidWalletEventsFilterByListTemplate struct{}

var FfiConverterHyperliquidWalletEventsFilterByListTemplateINSTANCE = FfiConverterHyperliquidWalletEventsFilterByListTemplate{}

func (c FfiConverterHyperliquidWalletEventsFilterByListTemplate) Lift(rb RustBufferI) HyperliquidWalletEventsFilterByListTemplate {
	return LiftFromRustBuffer[HyperliquidWalletEventsFilterByListTemplate](c, rb)
}

func (c FfiConverterHyperliquidWalletEventsFilterByListTemplate) Read(reader io.Reader) HyperliquidWalletEventsFilterByListTemplate {
	return HyperliquidWalletEventsFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterHyperliquidWalletEventsFilterByListTemplate) Lower(value HyperliquidWalletEventsFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[HyperliquidWalletEventsFilterByListTemplate](c, value)
}

func (c FfiConverterHyperliquidWalletEventsFilterByListTemplate) LowerExternal(value HyperliquidWalletEventsFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[HyperliquidWalletEventsFilterByListTemplate](c, value))
}

func (c FfiConverterHyperliquidWalletEventsFilterByListTemplate) Write(writer io.Writer, value HyperliquidWalletEventsFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.WalletsListName)
}

type FfiDestroyerHyperliquidWalletEventsFilterByListTemplate struct{}

func (_ FfiDestroyerHyperliquidWalletEventsFilterByListTemplate) Destroy(value HyperliquidWalletEventsFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for a Hyperliquid wallet-events filter.
type HyperliquidWalletEventsFilterTemplate struct {
	// Hyperliquid wallet addresses to match against.
	Wallets []string
}

func (r *HyperliquidWalletEventsFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Wallets)
}

type FfiConverterHyperliquidWalletEventsFilterTemplate struct{}

var FfiConverterHyperliquidWalletEventsFilterTemplateINSTANCE = FfiConverterHyperliquidWalletEventsFilterTemplate{}

func (c FfiConverterHyperliquidWalletEventsFilterTemplate) Lift(rb RustBufferI) HyperliquidWalletEventsFilterTemplate {
	return LiftFromRustBuffer[HyperliquidWalletEventsFilterTemplate](c, rb)
}

func (c FfiConverterHyperliquidWalletEventsFilterTemplate) Read(reader io.Reader) HyperliquidWalletEventsFilterTemplate {
	return HyperliquidWalletEventsFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterHyperliquidWalletEventsFilterTemplate) Lower(value HyperliquidWalletEventsFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[HyperliquidWalletEventsFilterTemplate](c, value)
}

func (c FfiConverterHyperliquidWalletEventsFilterTemplate) LowerExternal(value HyperliquidWalletEventsFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[HyperliquidWalletEventsFilterTemplate](c, value))
}

func (c FfiConverterHyperliquidWalletEventsFilterTemplate) Write(writer io.Writer, value HyperliquidWalletEventsFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Wallets)
}

type FfiDestroyerHyperliquidWalletEventsFilterTemplate struct{}

func (_ FfiDestroyerHyperliquidWalletEventsFilterTemplate) Destroy(value HyperliquidWalletEventsFilterTemplate) {
	value.Destroy()
}

// Parameters for `invite_team_member`.
type InviteTeamMemberRequest struct {
	// Email address to invite.
	Email string
	// Full name (required for new users).
	FullName *string
	// Team role (`admin`, `viewer`, or `billing`); required for new users.
	Role *string
}

func (r *InviteTeamMemberRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Email)
	FfiDestroyerOptionalString{}.Destroy(r.FullName)
	FfiDestroyerOptionalString{}.Destroy(r.Role)
}

type FfiConverterInviteTeamMemberRequest struct{}

var FfiConverterInviteTeamMemberRequestINSTANCE = FfiConverterInviteTeamMemberRequest{}

func (c FfiConverterInviteTeamMemberRequest) Lift(rb RustBufferI) InviteTeamMemberRequest {
	return LiftFromRustBuffer[InviteTeamMemberRequest](c, rb)
}

func (c FfiConverterInviteTeamMemberRequest) Read(reader io.Reader) InviteTeamMemberRequest {
	return InviteTeamMemberRequest{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterInviteTeamMemberRequest) Lower(value InviteTeamMemberRequest) C.RustBuffer {
	return LowerIntoRustBuffer[InviteTeamMemberRequest](c, value)
}

func (c FfiConverterInviteTeamMemberRequest) LowerExternal(value InviteTeamMemberRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[InviteTeamMemberRequest](c, value))
}

func (c FfiConverterInviteTeamMemberRequest) Write(writer io.Writer, value InviteTeamMemberRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Email)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FullName)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Role)
}

type FfiDestroyerInviteTeamMemberRequest struct{}

func (_ FfiDestroyerInviteTeamMemberRequest) Destroy(value InviteTeamMemberRequest) {
	value.Destroy()
}

// Response from `invite_team_member`.
type InviteTeamMemberResponse struct {
	// The invited user and their invitation status.
	Data *TeamUser
	// Error message when the request did not succeed.
	Error *string
}

func (r *InviteTeamMemberResponse) Destroy() {
	FfiDestroyerOptionalTeamUser{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterInviteTeamMemberResponse struct{}

var FfiConverterInviteTeamMemberResponseINSTANCE = FfiConverterInviteTeamMemberResponse{}

func (c FfiConverterInviteTeamMemberResponse) Lift(rb RustBufferI) InviteTeamMemberResponse {
	return LiftFromRustBuffer[InviteTeamMemberResponse](c, rb)
}

func (c FfiConverterInviteTeamMemberResponse) Read(reader io.Reader) InviteTeamMemberResponse {
	return InviteTeamMemberResponse{
		FfiConverterOptionalTeamUserINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterInviteTeamMemberResponse) Lower(value InviteTeamMemberResponse) C.RustBuffer {
	return LowerIntoRustBuffer[InviteTeamMemberResponse](c, value)
}

func (c FfiConverterInviteTeamMemberResponse) LowerExternal(value InviteTeamMemberResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[InviteTeamMemberResponse](c, value))
}

func (c FfiConverterInviteTeamMemberResponse) Write(writer io.Writer, value InviteTeamMemberResponse) {
	FfiConverterOptionalTeamUserINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerInviteTeamMemberResponse struct{}

func (_ FfiDestroyerInviteTeamMemberResponse) Destroy(value InviteTeamMemberResponse) {
	value.Destroy()
}

// An invoice issued to the account.
type Invoice struct {
	// Unique invoice identifier.
	Id string
	// Payment status (e.g. `paid`, `open`).
	Status string
	// Reason the invoice was generated (e.g. `subscription_cycle`).
	BillingReason string
	// Line items contributing to the invoice total.
	Lines []InvoiceLine
	// Amount due in the smallest currency unit.
	AmountDue int64
	// Amount already paid in the smallest currency unit.
	AmountPaid int64
	// Start of the billing period (Unix timestamp).
	PeriodStart int64
	// End of the billing period (Unix timestamp).
	PeriodEnd int64
	// Timestamp when the invoice was created (Unix timestamp).
	Created int64
	// Subtotal before taxes and adjustments.
	Subtotal int64
}

func (r *Invoice) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.BillingReason)
	FfiDestroyerSequenceInvoiceLine{}.Destroy(r.Lines)
	FfiDestroyerInt64{}.Destroy(r.AmountDue)
	FfiDestroyerInt64{}.Destroy(r.AmountPaid)
	FfiDestroyerInt64{}.Destroy(r.PeriodStart)
	FfiDestroyerInt64{}.Destroy(r.PeriodEnd)
	FfiDestroyerInt64{}.Destroy(r.Created)
	FfiDestroyerInt64{}.Destroy(r.Subtotal)
}

type FfiConverterInvoice struct{}

var FfiConverterInvoiceINSTANCE = FfiConverterInvoice{}

func (c FfiConverterInvoice) Lift(rb RustBufferI) Invoice {
	return LiftFromRustBuffer[Invoice](c, rb)
}

func (c FfiConverterInvoice) Read(reader io.Reader) Invoice {
	return Invoice{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceInvoiceLineINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterInvoice) Lower(value Invoice) C.RustBuffer {
	return LowerIntoRustBuffer[Invoice](c, value)
}

func (c FfiConverterInvoice) LowerExternal(value Invoice) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Invoice](c, value))
}

func (c FfiConverterInvoice) Write(writer io.Writer, value Invoice) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.BillingReason)
	FfiConverterSequenceInvoiceLineINSTANCE.Write(writer, value.Lines)
	FfiConverterInt64INSTANCE.Write(writer, value.AmountDue)
	FfiConverterInt64INSTANCE.Write(writer, value.AmountPaid)
	FfiConverterInt64INSTANCE.Write(writer, value.PeriodStart)
	FfiConverterInt64INSTANCE.Write(writer, value.PeriodEnd)
	FfiConverterInt64INSTANCE.Write(writer, value.Created)
	FfiConverterInt64INSTANCE.Write(writer, value.Subtotal)
}

type FfiDestroyerInvoice struct{}

func (_ FfiDestroyerInvoice) Destroy(value Invoice) {
	value.Destroy()
}

// A single line item on an invoice.
type InvoiceLine struct {
	// Human-readable description of the line item.
	Description string
	// Line item amount in the smallest currency unit.
	Amount int64
}

func (r *InvoiceLine) Destroy() {
	FfiDestroyerString{}.Destroy(r.Description)
	FfiDestroyerInt64{}.Destroy(r.Amount)
}

type FfiConverterInvoiceLine struct{}

var FfiConverterInvoiceLineINSTANCE = FfiConverterInvoiceLine{}

func (c FfiConverterInvoiceLine) Lift(rb RustBufferI) InvoiceLine {
	return LiftFromRustBuffer[InvoiceLine](c, rb)
}

func (c FfiConverterInvoiceLine) Read(reader io.Reader) InvoiceLine {
	return InvoiceLine{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterInvoiceLine) Lower(value InvoiceLine) C.RustBuffer {
	return LowerIntoRustBuffer[InvoiceLine](c, value)
}

func (c FfiConverterInvoiceLine) LowerExternal(value InvoiceLine) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[InvoiceLine](c, value))
}

func (c FfiConverterInvoiceLine) Write(writer io.Writer, value InvoiceLine) {
	FfiConverterStringINSTANCE.Write(writer, value.Description)
	FfiConverterInt64INSTANCE.Write(writer, value.Amount)
}

type FfiDestroyerInvoiceLine struct{}

func (_ FfiDestroyerInvoiceLine) Destroy(value InvoiceLine) {
	value.Destroy()
}

// Data wrapper for the IP custom header configuration.
type IpCustomHeaderData struct {
	// Configured header name.
	HeaderName string
}

func (r *IpCustomHeaderData) Destroy() {
	FfiDestroyerString{}.Destroy(r.HeaderName)
}

type FfiConverterIpCustomHeaderData struct{}

var FfiConverterIpCustomHeaderDataINSTANCE = FfiConverterIpCustomHeaderData{}

func (c FfiConverterIpCustomHeaderData) Lift(rb RustBufferI) IpCustomHeaderData {
	return LiftFromRustBuffer[IpCustomHeaderData](c, rb)
}

func (c FfiConverterIpCustomHeaderData) Read(reader io.Reader) IpCustomHeaderData {
	return IpCustomHeaderData{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterIpCustomHeaderData) Lower(value IpCustomHeaderData) C.RustBuffer {
	return LowerIntoRustBuffer[IpCustomHeaderData](c, value)
}

func (c FfiConverterIpCustomHeaderData) LowerExternal(value IpCustomHeaderData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[IpCustomHeaderData](c, value))
}

func (c FfiConverterIpCustomHeaderData) Write(writer io.Writer, value IpCustomHeaderData) {
	FfiConverterStringINSTANCE.Write(writer, value.HeaderName)
}

type FfiDestroyerIpCustomHeaderData struct{}

func (_ FfiDestroyerIpCustomHeaderData) Destroy(value IpCustomHeaderData) {
	value.Destroy()
}

// Configuration for delivering stream batches to a Kafka topic.
type KafkaAttributes struct {
	// Comma-separated list of Kafka broker addresses (host:port).
	BootstrapServers string
	// Destination topic.
	TopicName string
	// Compression codec applied to produced messages (e.g. `none`, `gzip`).
	CompressionType string
	// Maximum number of messages grouped per produce request.
	BatchSize int32
	// Milliseconds the producer waits to batch additional messages.
	LingerMs int32
	// Maximum size in bytes of a single Kafka message (`max_message_bytes`).
	MaxMessageBytes int32
	// Request timeout in seconds.
	TimeoutSec int32
	// Maximum number of retry attempts for a failed produce.
	MaxRetry int32
	// Seconds to wait between retry attempts.
	RetryIntervalSec int32
	// Optional SASL username.
	Username *string
	// Optional SASL password.
	Password *string
	// Optional security protocol (e.g. `SASL_SSL`).
	Protocol *string
	// Optional SASL mechanism (e.g. `PLAIN`, `SCRAM-SHA-256`).
	Mechanisms *string
}

func (r *KafkaAttributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.BootstrapServers)
	FfiDestroyerString{}.Destroy(r.TopicName)
	FfiDestroyerString{}.Destroy(r.CompressionType)
	FfiDestroyerInt32{}.Destroy(r.BatchSize)
	FfiDestroyerInt32{}.Destroy(r.LingerMs)
	FfiDestroyerInt32{}.Destroy(r.MaxMessageBytes)
	FfiDestroyerInt32{}.Destroy(r.TimeoutSec)
	FfiDestroyerInt32{}.Destroy(r.MaxRetry)
	FfiDestroyerInt32{}.Destroy(r.RetryIntervalSec)
	FfiDestroyerOptionalString{}.Destroy(r.Username)
	FfiDestroyerOptionalString{}.Destroy(r.Password)
	FfiDestroyerOptionalString{}.Destroy(r.Protocol)
	FfiDestroyerOptionalString{}.Destroy(r.Mechanisms)
}

type FfiConverterKafkaAttributes struct{}

var FfiConverterKafkaAttributesINSTANCE = FfiConverterKafkaAttributes{}

func (c FfiConverterKafkaAttributes) Lift(rb RustBufferI) KafkaAttributes {
	return LiftFromRustBuffer[KafkaAttributes](c, rb)
}

func (c FfiConverterKafkaAttributes) Read(reader io.Reader) KafkaAttributes {
	return KafkaAttributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterKafkaAttributes) Lower(value KafkaAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[KafkaAttributes](c, value)
}

func (c FfiConverterKafkaAttributes) LowerExternal(value KafkaAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[KafkaAttributes](c, value))
}

func (c FfiConverterKafkaAttributes) Write(writer io.Writer, value KafkaAttributes) {
	FfiConverterStringINSTANCE.Write(writer, value.BootstrapServers)
	FfiConverterStringINSTANCE.Write(writer, value.TopicName)
	FfiConverterStringINSTANCE.Write(writer, value.CompressionType)
	FfiConverterInt32INSTANCE.Write(writer, value.BatchSize)
	FfiConverterInt32INSTANCE.Write(writer, value.LingerMs)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxMessageBytes)
	FfiConverterInt32INSTANCE.Write(writer, value.TimeoutSec)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxRetry)
	FfiConverterInt32INSTANCE.Write(writer, value.RetryIntervalSec)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Username)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Password)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Protocol)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Mechanisms)
}

type FfiDestroyerKafkaAttributes struct{}

func (_ FfiDestroyerKafkaAttributes) Destroy(value KafkaAttributes) {
	value.Destroy()
}

// A single key/value entry returned by `get_sets`.
type KvSetEntry struct {
	// Key identifying the set.
	Key string
	// Stored string value.
	Value string
}

func (r *KvSetEntry) Destroy() {
	FfiDestroyerString{}.Destroy(r.Key)
	FfiDestroyerString{}.Destroy(r.Value)
}

type FfiConverterKvSetEntry struct{}

var FfiConverterKvSetEntryINSTANCE = FfiConverterKvSetEntry{}

func (c FfiConverterKvSetEntry) Lift(rb RustBufferI) KvSetEntry {
	return LiftFromRustBuffer[KvSetEntry](c, rb)
}

func (c FfiConverterKvSetEntry) Read(reader io.Reader) KvSetEntry {
	return KvSetEntry{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterKvSetEntry) Lower(value KvSetEntry) C.RustBuffer {
	return LowerIntoRustBuffer[KvSetEntry](c, value)
}

func (c FfiConverterKvSetEntry) LowerExternal(value KvSetEntry) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[KvSetEntry](c, value))
}

func (c FfiConverterKvSetEntry) Write(writer io.Writer, value KvSetEntry) {
	FfiConverterStringINSTANCE.Write(writer, value.Key)
	FfiConverterStringINSTANCE.Write(writer, value.Value)
}

type FfiDestroyerKvSetEntry struct{}

func (_ FfiDestroyerKvSetEntry) Destroy(value KvSetEntry) {
	value.Destroy()
}

// Response from `list_chains`.
type ListChainsResponse struct {
	// Supported chains and their networks.
	Data []Chain
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListChainsResponse) Destroy() {
	FfiDestroyerSequenceChain{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListChainsResponse struct{}

var FfiConverterListChainsResponseINSTANCE = FfiConverterListChainsResponse{}

func (c FfiConverterListChainsResponse) Lift(rb RustBufferI) ListChainsResponse {
	return LiftFromRustBuffer[ListChainsResponse](c, rb)
}

func (c FfiConverterListChainsResponse) Read(reader io.Reader) ListChainsResponse {
	return ListChainsResponse{
		FfiConverterSequenceChainINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListChainsResponse) Lower(value ListChainsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListChainsResponse](c, value)
}

func (c FfiConverterListChainsResponse) LowerExternal(value ListChainsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListChainsResponse](c, value))
}

func (c FfiConverterListChainsResponse) Write(writer io.Writer, value ListChainsResponse) {
	FfiConverterSequenceChainINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListChainsResponse struct{}

func (_ FfiDestroyerListChainsResponse) Destroy(value ListChainsResponse) {
	value.Destroy()
}

// Response from `list_contains_item`.
type ListContainsItemResponse struct {
	// `true` when the item is present in the list.
	Exists bool
}

func (r *ListContainsItemResponse) Destroy() {
	FfiDestroyerBool{}.Destroy(r.Exists)
}

type FfiConverterListContainsItemResponse struct{}

var FfiConverterListContainsItemResponseINSTANCE = FfiConverterListContainsItemResponse{}

func (c FfiConverterListContainsItemResponse) Lift(rb RustBufferI) ListContainsItemResponse {
	return LiftFromRustBuffer[ListContainsItemResponse](c, rb)
}

func (c FfiConverterListContainsItemResponse) Read(reader io.Reader) ListContainsItemResponse {
	return ListContainsItemResponse{
		FfiConverterBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterListContainsItemResponse) Lower(value ListContainsItemResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListContainsItemResponse](c, value)
}

func (c FfiConverterListContainsItemResponse) LowerExternal(value ListContainsItemResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListContainsItemResponse](c, value))
}

func (c FfiConverterListContainsItemResponse) Write(writer io.Writer, value ListContainsItemResponse) {
	FfiConverterBoolINSTANCE.Write(writer, value.Exists)
}

type FfiDestroyerListContainsItemResponse struct{}

func (_ FfiDestroyerListContainsItemResponse) Destroy(value ListContainsItemResponse) {
	value.Destroy()
}

// Invoice list wrapper.
type ListInvoicesData struct {
	// Invoices on the account.
	Invoices []Invoice
}

func (r *ListInvoicesData) Destroy() {
	FfiDestroyerSequenceInvoice{}.Destroy(r.Invoices)
}

type FfiConverterListInvoicesData struct{}

var FfiConverterListInvoicesDataINSTANCE = FfiConverterListInvoicesData{}

func (c FfiConverterListInvoicesData) Lift(rb RustBufferI) ListInvoicesData {
	return LiftFromRustBuffer[ListInvoicesData](c, rb)
}

func (c FfiConverterListInvoicesData) Read(reader io.Reader) ListInvoicesData {
	return ListInvoicesData{
		FfiConverterSequenceInvoiceINSTANCE.Read(reader),
	}
}

func (c FfiConverterListInvoicesData) Lower(value ListInvoicesData) C.RustBuffer {
	return LowerIntoRustBuffer[ListInvoicesData](c, value)
}

func (c FfiConverterListInvoicesData) LowerExternal(value ListInvoicesData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListInvoicesData](c, value))
}

func (c FfiConverterListInvoicesData) Write(writer io.Writer, value ListInvoicesData) {
	FfiConverterSequenceInvoiceINSTANCE.Write(writer, value.Invoices)
}

type FfiDestroyerListInvoicesData struct{}

func (_ FfiDestroyerListInvoicesData) Destroy(value ListInvoicesData) {
	value.Destroy()
}

// Response from `list_invoices`.
type ListInvoicesResponse struct {
	// Invoice data payload.
	Data *ListInvoicesData
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListInvoicesResponse) Destroy() {
	FfiDestroyerOptionalListInvoicesData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListInvoicesResponse struct{}

var FfiConverterListInvoicesResponseINSTANCE = FfiConverterListInvoicesResponse{}

func (c FfiConverterListInvoicesResponse) Lift(rb RustBufferI) ListInvoicesResponse {
	return LiftFromRustBuffer[ListInvoicesResponse](c, rb)
}

func (c FfiConverterListInvoicesResponse) Read(reader io.Reader) ListInvoicesResponse {
	return ListInvoicesResponse{
		FfiConverterOptionalListInvoicesDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListInvoicesResponse) Lower(value ListInvoicesResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListInvoicesResponse](c, value)
}

func (c FfiConverterListInvoicesResponse) LowerExternal(value ListInvoicesResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListInvoicesResponse](c, value))
}

func (c FfiConverterListInvoicesResponse) Write(writer io.Writer, value ListInvoicesResponse) {
	FfiConverterOptionalListInvoicesDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListInvoicesResponse struct{}

func (_ FfiDestroyerListInvoicesResponse) Destroy(value ListInvoicesResponse) {
	value.Destroy()
}

// Payment list wrapper.
type ListPaymentsData struct {
	// Payments on the account.
	Payments []Payment
}

func (r *ListPaymentsData) Destroy() {
	FfiDestroyerSequencePayment{}.Destroy(r.Payments)
}

type FfiConverterListPaymentsData struct{}

var FfiConverterListPaymentsDataINSTANCE = FfiConverterListPaymentsData{}

func (c FfiConverterListPaymentsData) Lift(rb RustBufferI) ListPaymentsData {
	return LiftFromRustBuffer[ListPaymentsData](c, rb)
}

func (c FfiConverterListPaymentsData) Read(reader io.Reader) ListPaymentsData {
	return ListPaymentsData{
		FfiConverterSequencePaymentINSTANCE.Read(reader),
	}
}

func (c FfiConverterListPaymentsData) Lower(value ListPaymentsData) C.RustBuffer {
	return LowerIntoRustBuffer[ListPaymentsData](c, value)
}

func (c FfiConverterListPaymentsData) LowerExternal(value ListPaymentsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListPaymentsData](c, value))
}

func (c FfiConverterListPaymentsData) Write(writer io.Writer, value ListPaymentsData) {
	FfiConverterSequencePaymentINSTANCE.Write(writer, value.Payments)
}

type FfiDestroyerListPaymentsData struct{}

func (_ FfiDestroyerListPaymentsData) Destroy(value ListPaymentsData) {
	value.Destroy()
}

// Response from `list_payments`.
type ListPaymentsResponse struct {
	// Payment data payload.
	Data *ListPaymentsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListPaymentsResponse) Destroy() {
	FfiDestroyerOptionalListPaymentsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListPaymentsResponse struct{}

var FfiConverterListPaymentsResponseINSTANCE = FfiConverterListPaymentsResponse{}

func (c FfiConverterListPaymentsResponse) Lift(rb RustBufferI) ListPaymentsResponse {
	return LiftFromRustBuffer[ListPaymentsResponse](c, rb)
}

func (c FfiConverterListPaymentsResponse) Read(reader io.Reader) ListPaymentsResponse {
	return ListPaymentsResponse{
		FfiConverterOptionalListPaymentsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListPaymentsResponse) Lower(value ListPaymentsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListPaymentsResponse](c, value)
}

func (c FfiConverterListPaymentsResponse) LowerExternal(value ListPaymentsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListPaymentsResponse](c, value))
}

func (c FfiConverterListPaymentsResponse) Write(writer io.Writer, value ListPaymentsResponse) {
	FfiConverterOptionalListPaymentsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListPaymentsResponse struct{}

func (_ FfiDestroyerListPaymentsResponse) Destroy(value ListPaymentsResponse) {
	value.Destroy()
}

// Parameters for `list_streams`.
type ListStreamsParams struct {
	// Filter results by stream type.
	StreamType *string
	// Starting index into the result set; defaults to 0.
	Offset *int64
	// Maximum number of streams returned.
	Limit *int64
	// Field to sort results by.
	OrderBy *string
	// Sort direction (`asc` or `desc`).
	OrderDirection *string
}

func (r *ListStreamsParams) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.StreamType)
	FfiDestroyerOptionalInt64{}.Destroy(r.Offset)
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalString{}.Destroy(r.OrderBy)
	FfiDestroyerOptionalString{}.Destroy(r.OrderDirection)
}

type FfiConverterListStreamsParams struct{}

var FfiConverterListStreamsParamsINSTANCE = FfiConverterListStreamsParams{}

func (c FfiConverterListStreamsParams) Lift(rb RustBufferI) ListStreamsParams {
	return LiftFromRustBuffer[ListStreamsParams](c, rb)
}

func (c FfiConverterListStreamsParams) Read(reader io.Reader) ListStreamsParams {
	return ListStreamsParams{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListStreamsParams) Lower(value ListStreamsParams) C.RustBuffer {
	return LowerIntoRustBuffer[ListStreamsParams](c, value)
}

func (c FfiConverterListStreamsParams) LowerExternal(value ListStreamsParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListStreamsParams](c, value))
}

func (c FfiConverterListStreamsParams) Write(writer io.Writer, value ListStreamsParams) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.StreamType)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Offset)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.OrderBy)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.OrderDirection)
}

type FfiDestroyerListStreamsParams struct{}

func (_ FfiDestroyerListStreamsParams) Destroy(value ListStreamsParams) {
	value.Destroy()
}

// Paginated response from `list_streams`.
type ListStreamsResponse struct {
	// Streams on the current page.
	Data []Stream
	// Pagination metadata for the response.
	PageInfo PageInfo
}

func (r *ListStreamsResponse) Destroy() {
	FfiDestroyerSequenceStream{}.Destroy(r.Data)
	FfiDestroyerPageInfo{}.Destroy(r.PageInfo)
}

type FfiConverterListStreamsResponse struct{}

var FfiConverterListStreamsResponseINSTANCE = FfiConverterListStreamsResponse{}

func (c FfiConverterListStreamsResponse) Lift(rb RustBufferI) ListStreamsResponse {
	return LiftFromRustBuffer[ListStreamsResponse](c, rb)
}

func (c FfiConverterListStreamsResponse) Read(reader io.Reader) ListStreamsResponse {
	return ListStreamsResponse{
		FfiConverterSequenceStreamINSTANCE.Read(reader),
		FfiConverterPageInfoINSTANCE.Read(reader),
	}
}

func (c FfiConverterListStreamsResponse) Lower(value ListStreamsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListStreamsResponse](c, value)
}

func (c FfiConverterListStreamsResponse) LowerExternal(value ListStreamsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListStreamsResponse](c, value))
}

func (c FfiConverterListStreamsResponse) Write(writer io.Writer, value ListStreamsResponse) {
	FfiConverterSequenceStreamINSTANCE.Write(writer, value.Data)
	FfiConverterPageInfoINSTANCE.Write(writer, value.PageInfo)
}

type FfiDestroyerListStreamsResponse struct{}

func (_ FfiDestroyerListStreamsResponse) Destroy(value ListStreamsResponse) {
	value.Destroy()
}

// Inner data wrapper for `list_tags`.
type ListTagsData struct {
	// Tags on the account.
	Tags []AccountTag
}

func (r *ListTagsData) Destroy() {
	FfiDestroyerSequenceAccountTag{}.Destroy(r.Tags)
}

type FfiConverterListTagsData struct{}

var FfiConverterListTagsDataINSTANCE = FfiConverterListTagsData{}

func (c FfiConverterListTagsData) Lift(rb RustBufferI) ListTagsData {
	return LiftFromRustBuffer[ListTagsData](c, rb)
}

func (c FfiConverterListTagsData) Read(reader io.Reader) ListTagsData {
	return ListTagsData{
		FfiConverterSequenceAccountTagINSTANCE.Read(reader),
	}
}

func (c FfiConverterListTagsData) Lower(value ListTagsData) C.RustBuffer {
	return LowerIntoRustBuffer[ListTagsData](c, value)
}

func (c FfiConverterListTagsData) LowerExternal(value ListTagsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListTagsData](c, value))
}

func (c FfiConverterListTagsData) Write(writer io.Writer, value ListTagsData) {
	FfiConverterSequenceAccountTagINSTANCE.Write(writer, value.Tags)
}

type FfiDestroyerListTagsData struct{}

func (_ FfiDestroyerListTagsData) Destroy(value ListTagsData) {
	value.Destroy()
}

// Response from `list_tags`.
type ListTagsResponse struct {
	// Account tags payload.
	Data *ListTagsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListTagsResponse) Destroy() {
	FfiDestroyerOptionalListTagsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListTagsResponse struct{}

var FfiConverterListTagsResponseINSTANCE = FfiConverterListTagsResponse{}

func (c FfiConverterListTagsResponse) Lift(rb RustBufferI) ListTagsResponse {
	return LiftFromRustBuffer[ListTagsResponse](c, rb)
}

func (c FfiConverterListTagsResponse) Read(reader io.Reader) ListTagsResponse {
	return ListTagsResponse{
		FfiConverterOptionalListTagsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListTagsResponse) Lower(value ListTagsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListTagsResponse](c, value)
}

func (c FfiConverterListTagsResponse) LowerExternal(value ListTagsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListTagsResponse](c, value))
}

func (c FfiConverterListTagsResponse) Write(writer io.Writer, value ListTagsResponse) {
	FfiConverterOptionalListTagsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListTagsResponse struct{}

func (_ FfiDestroyerListTagsResponse) Destroy(value ListTagsResponse) {
	value.Destroy()
}

// Response from `list_team_endpoints`.
type ListTeamEndpointsResponse struct {
	// Endpoints accessible to the team.
	Data []TeamEndpoint
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListTeamEndpointsResponse) Destroy() {
	FfiDestroyerSequenceTeamEndpoint{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListTeamEndpointsResponse struct{}

var FfiConverterListTeamEndpointsResponseINSTANCE = FfiConverterListTeamEndpointsResponse{}

func (c FfiConverterListTeamEndpointsResponse) Lift(rb RustBufferI) ListTeamEndpointsResponse {
	return LiftFromRustBuffer[ListTeamEndpointsResponse](c, rb)
}

func (c FfiConverterListTeamEndpointsResponse) Read(reader io.Reader) ListTeamEndpointsResponse {
	return ListTeamEndpointsResponse{
		FfiConverterSequenceTeamEndpointINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListTeamEndpointsResponse) Lower(value ListTeamEndpointsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListTeamEndpointsResponse](c, value)
}

func (c FfiConverterListTeamEndpointsResponse) LowerExternal(value ListTeamEndpointsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListTeamEndpointsResponse](c, value))
}

func (c FfiConverterListTeamEndpointsResponse) Write(writer io.Writer, value ListTeamEndpointsResponse) {
	FfiConverterSequenceTeamEndpointINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListTeamEndpointsResponse struct{}

func (_ FfiDestroyerListTeamEndpointsResponse) Destroy(value ListTeamEndpointsResponse) {
	value.Destroy()
}

// Response from `list_teams`.
type ListTeamsResponse struct {
	// Teams on the account.
	Data []TeamSummary
	// Error message when the request did not succeed.
	Error *string
}

func (r *ListTeamsResponse) Destroy() {
	FfiDestroyerSequenceTeamSummary{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterListTeamsResponse struct{}

var FfiConverterListTeamsResponseINSTANCE = FfiConverterListTeamsResponse{}

func (c FfiConverterListTeamsResponse) Lift(rb RustBufferI) ListTeamsResponse {
	return LiftFromRustBuffer[ListTeamsResponse](c, rb)
}

func (c FfiConverterListTeamsResponse) Read(reader io.Reader) ListTeamsResponse {
	return ListTeamsResponse{
		FfiConverterSequenceTeamSummaryINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterListTeamsResponse) Lower(value ListTeamsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListTeamsResponse](c, value)
}

func (c FfiConverterListTeamsResponse) LowerExternal(value ListTeamsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListTeamsResponse](c, value))
}

func (c FfiConverterListTeamsResponse) Write(writer io.Writer, value ListTeamsResponse) {
	FfiConverterSequenceTeamSummaryINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerListTeamsResponse struct{}

func (_ FfiDestroyerListTeamsResponse) Destroy(value ListTeamsResponse) {
	value.Destroy()
}

// Response from `list_webhooks`.
type ListWebhooksResponse struct {
	// Webhooks on the current page.
	Data []Webhook
	// Pagination metadata for the response.
	PageInfo WebhookPageInfo
}

func (r *ListWebhooksResponse) Destroy() {
	FfiDestroyerSequenceWebhook{}.Destroy(r.Data)
	FfiDestroyerWebhookPageInfo{}.Destroy(r.PageInfo)
}

type FfiConverterListWebhooksResponse struct{}

var FfiConverterListWebhooksResponseINSTANCE = FfiConverterListWebhooksResponse{}

func (c FfiConverterListWebhooksResponse) Lift(rb RustBufferI) ListWebhooksResponse {
	return LiftFromRustBuffer[ListWebhooksResponse](c, rb)
}

func (c FfiConverterListWebhooksResponse) Read(reader io.Reader) ListWebhooksResponse {
	return ListWebhooksResponse{
		FfiConverterSequenceWebhookINSTANCE.Read(reader),
		FfiConverterWebhookPageInfoINSTANCE.Read(reader),
	}
}

func (c FfiConverterListWebhooksResponse) Lower(value ListWebhooksResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ListWebhooksResponse](c, value)
}

func (c FfiConverterListWebhooksResponse) LowerExternal(value ListWebhooksResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ListWebhooksResponse](c, value))
}

func (c FfiConverterListWebhooksResponse) Write(writer io.Writer, value ListWebhooksResponse) {
	FfiConverterSequenceWebhookINSTANCE.Write(writer, value.Data)
	FfiConverterWebhookPageInfoINSTANCE.Write(writer, value.PageInfo)
}

type FfiDestroyerListWebhooksResponse struct{}

func (_ FfiDestroyerListWebhooksResponse) Destroy(value ListWebhooksResponse) {
	value.Destroy()
}

// Raw request/response payloads attached to a log entry.
type LogDetails struct {
	// JSON-encoded request body (truncated at 2KB).
	Request *string
	// JSON-encoded response body (truncated at 2KB).
	Response *string
}

func (r *LogDetails) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Request)
	FfiDestroyerOptionalString{}.Destroy(r.Response)
}

type FfiConverterLogDetails struct{}

var FfiConverterLogDetailsINSTANCE = FfiConverterLogDetails{}

func (c FfiConverterLogDetails) Lift(rb RustBufferI) LogDetails {
	return LiftFromRustBuffer[LogDetails](c, rb)
}

func (c FfiConverterLogDetails) Read(reader io.Reader) LogDetails {
	return LogDetails{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterLogDetails) Lower(value LogDetails) C.RustBuffer {
	return LowerIntoRustBuffer[LogDetails](c, value)
}

func (c FfiConverterLogDetails) LowerExternal(value LogDetails) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[LogDetails](c, value))
}

func (c FfiConverterLogDetails) Write(writer io.Writer, value LogDetails) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Request)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Response)
}

type FfiDestroyerLogDetails struct{}

func (_ FfiDestroyerLogDetails) Destroy(value LogDetails) {
	value.Destroy()
}

// A per-method rate limiter configured on an endpoint.
type MethodRateLimiter struct {
	// Rate limiter identifier.
	Id string
	// Interval over which the rate applies (e.g. `second`, `minute`).
	Interval string
	// RPC methods the limiter applies to.
	Methods []string
	// Maximum number of calls allowed per interval.
	Rate int32
	// Whether the limiter is `enabled` or `disabled`.
	Status string
	// Creation timestamp.
	Created string
}

func (r *MethodRateLimiter) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Interval)
	FfiDestroyerSequenceString{}.Destroy(r.Methods)
	FfiDestroyerInt32{}.Destroy(r.Rate)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.Created)
}

type FfiConverterMethodRateLimiter struct{}

var FfiConverterMethodRateLimiterINSTANCE = FfiConverterMethodRateLimiter{}

func (c FfiConverterMethodRateLimiter) Lift(rb RustBufferI) MethodRateLimiter {
	return LiftFromRustBuffer[MethodRateLimiter](c, rb)
}

func (c FfiConverterMethodRateLimiter) Read(reader io.Reader) MethodRateLimiter {
	return MethodRateLimiter{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterMethodRateLimiter) Lower(value MethodRateLimiter) C.RustBuffer {
	return LowerIntoRustBuffer[MethodRateLimiter](c, value)
}

func (c FfiConverterMethodRateLimiter) LowerExternal(value MethodRateLimiter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[MethodRateLimiter](c, value))
}

func (c FfiConverterMethodRateLimiter) Write(writer io.Writer, value MethodRateLimiter) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Interval)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Methods)
	FfiConverterInt32INSTANCE.Write(writer, value.Rate)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.Created)
}

type FfiDestroyerMethodRateLimiter struct{}

func (_ FfiDestroyerMethodRateLimiter) Destroy(value MethodRateLimiter) {
	value.Destroy()
}

// Per-method usage row.
type MethodUsage struct {
	// RPC method name.
	MethodName string
	// Credits consumed by this method.
	CreditsUsed int64
	// Whether the call required an archival node.
	Archive *bool
	// Network the calls targeted.
	Network *string
	// Chain the calls targeted.
	Chain *string
}

func (r *MethodUsage) Destroy() {
	FfiDestroyerString{}.Destroy(r.MethodName)
	FfiDestroyerInt64{}.Destroy(r.CreditsUsed)
	FfiDestroyerOptionalBool{}.Destroy(r.Archive)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
	FfiDestroyerOptionalString{}.Destroy(r.Chain)
}

type FfiConverterMethodUsage struct{}

var FfiConverterMethodUsageINSTANCE = FfiConverterMethodUsage{}

func (c FfiConverterMethodUsage) Lift(rb RustBufferI) MethodUsage {
	return LiftFromRustBuffer[MethodUsage](c, rb)
}

func (c FfiConverterMethodUsage) Read(reader io.Reader) MethodUsage {
	return MethodUsage{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterMethodUsage) Lower(value MethodUsage) C.RustBuffer {
	return LowerIntoRustBuffer[MethodUsage](c, value)
}

func (c FfiConverterMethodUsage) LowerExternal(value MethodUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[MethodUsage](c, value))
}

func (c FfiConverterMethodUsage) Write(writer io.Writer, value MethodUsage) {
	FfiConverterStringINSTANCE.Write(writer, value.MethodName)
	FfiConverterInt64INSTANCE.Write(writer, value.CreditsUsed)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Archive)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Chain)
}

type FfiDestroyerMethodUsage struct{}

func (_ FfiDestroyerMethodUsage) Destroy(value MethodUsage) {
	value.Destroy()
}

// Pagination metadata returned alongside a paginated result set.
type PageInfo struct {
	// Page size used for this response.
	Limit int64
	// Starting index of this page within the full result set.
	Offset int64
	// Total number of items matching the query across all pages.
	Total int64
}

func (r *PageInfo) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Limit)
	FfiDestroyerInt64{}.Destroy(r.Offset)
	FfiDestroyerInt64{}.Destroy(r.Total)
}

type FfiConverterPageInfo struct{}

var FfiConverterPageInfoINSTANCE = FfiConverterPageInfo{}

func (c FfiConverterPageInfo) Lift(rb RustBufferI) PageInfo {
	return LiftFromRustBuffer[PageInfo](c, rb)
}

func (c FfiConverterPageInfo) Read(reader io.Reader) PageInfo {
	return PageInfo{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterPageInfo) Lower(value PageInfo) C.RustBuffer {
	return LowerIntoRustBuffer[PageInfo](c, value)
}

func (c FfiConverterPageInfo) LowerExternal(value PageInfo) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[PageInfo](c, value))
}

func (c FfiConverterPageInfo) Write(writer io.Writer, value PageInfo) {
	FfiConverterInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterInt64INSTANCE.Write(writer, value.Offset)
	FfiConverterInt64INSTANCE.Write(writer, value.Total)
}

type FfiDestroyerPageInfo struct{}

func (_ FfiDestroyerPageInfo) Destroy(value PageInfo) {
	value.Destroy()
}

// Pagination metadata for admin list responses.
type Pagination struct {
	// Total number of items matching the query across all pages.
	Total int64
	// Page size used for this response.
	Limit int32
	// Starting index of this page within the full result set.
	Offset int32
}

func (r *Pagination) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Total)
	FfiDestroyerInt32{}.Destroy(r.Limit)
	FfiDestroyerInt32{}.Destroy(r.Offset)
}

type FfiConverterPagination struct{}

var FfiConverterPaginationINSTANCE = FfiConverterPagination{}

func (c FfiConverterPagination) Lift(rb RustBufferI) Pagination {
	return LiftFromRustBuffer[Pagination](c, rb)
}

func (c FfiConverterPagination) Read(reader io.Reader) Pagination {
	return Pagination{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterPagination) Lower(value Pagination) C.RustBuffer {
	return LowerIntoRustBuffer[Pagination](c, value)
}

func (c FfiConverterPagination) LowerExternal(value Pagination) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Pagination](c, value))
}

func (c FfiConverterPagination) Write(writer io.Writer, value Pagination) {
	FfiConverterInt64INSTANCE.Write(writer, value.Total)
	FfiConverterInt32INSTANCE.Write(writer, value.Limit)
	FfiConverterInt32INSTANCE.Write(writer, value.Offset)
}

type FfiDestroyerPagination struct{}

func (_ FfiDestroyerPagination) Destroy(value Pagination) {
	value.Destroy()
}

// A payment recorded on the account.
type Payment struct {
	// Payment amount as a string in the account's currency.
	Amount string
	// Last four digits of the card used for the payment.
	CardLast4 *string
	// Timestamp when the payment was recorded.
	CreatedAt string
	// Currency code (e.g. `usd`).
	Currency string
	// Payment status.
	Status string
	// Portion of the payment attributed to marketplace spending.
	MarketplaceAmount *string
}

func (r *Payment) Destroy() {
	FfiDestroyerString{}.Destroy(r.Amount)
	FfiDestroyerOptionalString{}.Destroy(r.CardLast4)
	FfiDestroyerString{}.Destroy(r.CreatedAt)
	FfiDestroyerString{}.Destroy(r.Currency)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerOptionalString{}.Destroy(r.MarketplaceAmount)
}

type FfiConverterPayment struct{}

var FfiConverterPaymentINSTANCE = FfiConverterPayment{}

func (c FfiConverterPayment) Lift(rb RustBufferI) Payment {
	return LiftFromRustBuffer[Payment](c, rb)
}

func (c FfiConverterPayment) Read(reader io.Reader) Payment {
	return Payment{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterPayment) Lower(value Payment) C.RustBuffer {
	return LowerIntoRustBuffer[Payment](c, value)
}

func (c FfiConverterPayment) LowerExternal(value Payment) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Payment](c, value))
}

func (c FfiConverterPayment) Write(writer io.Writer, value Payment) {
	FfiConverterStringINSTANCE.Write(writer, value.Amount)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.CardLast4)
	FfiConverterStringINSTANCE.Write(writer, value.CreatedAt)
	FfiConverterStringINSTANCE.Write(writer, value.Currency)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.MarketplaceAmount)
}

type FfiDestroyerPayment struct{}

func (_ FfiDestroyerPayment) Destroy(value Payment) {
	value.Destroy()
}

// Configuration for delivering stream batches to a PostgreSQL database.
type PostgresAttributes struct {
	// Database host.
	Host string
	// Database port.
	Port int32
	// Database name.
	Database string
	// Username used to authenticate.
	Username string
	// Password used to authenticate.
	Password string
	// Destination table for inserted rows.
	TableName string
	// Postgres SSL mode. The Quicknode API accepts only `disable` or `require`.
	Sslmode string
	// Maximum number of retry attempts for a failed write.
	MaxRetry int32
	// Seconds to wait between retry attempts.
	RetryIntervalSec int32
}

func (r *PostgresAttributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.Host)
	FfiDestroyerInt32{}.Destroy(r.Port)
	FfiDestroyerString{}.Destroy(r.Database)
	FfiDestroyerString{}.Destroy(r.Username)
	FfiDestroyerString{}.Destroy(r.Password)
	FfiDestroyerString{}.Destroy(r.TableName)
	FfiDestroyerString{}.Destroy(r.Sslmode)
	FfiDestroyerInt32{}.Destroy(r.MaxRetry)
	FfiDestroyerInt32{}.Destroy(r.RetryIntervalSec)
}

type FfiConverterPostgresAttributes struct{}

var FfiConverterPostgresAttributesINSTANCE = FfiConverterPostgresAttributes{}

func (c FfiConverterPostgresAttributes) Lift(rb RustBufferI) PostgresAttributes {
	return LiftFromRustBuffer[PostgresAttributes](c, rb)
}

func (c FfiConverterPostgresAttributes) Read(reader io.Reader) PostgresAttributes {
	return PostgresAttributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterPostgresAttributes) Lower(value PostgresAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[PostgresAttributes](c, value)
}

func (c FfiConverterPostgresAttributes) LowerExternal(value PostgresAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[PostgresAttributes](c, value))
}

func (c FfiConverterPostgresAttributes) Write(writer io.Writer, value PostgresAttributes) {
	FfiConverterStringINSTANCE.Write(writer, value.Host)
	FfiConverterInt32INSTANCE.Write(writer, value.Port)
	FfiConverterStringINSTANCE.Write(writer, value.Database)
	FfiConverterStringINSTANCE.Write(writer, value.Username)
	FfiConverterStringINSTANCE.Write(writer, value.Password)
	FfiConverterStringINSTANCE.Write(writer, value.TableName)
	FfiConverterStringINSTANCE.Write(writer, value.Sslmode)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxRetry)
	FfiConverterInt32INSTANCE.Write(writer, value.RetryIntervalSec)
}

type FfiDestroyerPostgresAttributes struct{}

func (_ FfiDestroyerPostgresAttributes) Destroy(value PostgresAttributes) {
	value.Destroy()
}

// Parameters for `query`.
type QueryParams struct {
	// The SQL query to execute. Pagination is expressed in the SQL itself via
	// `LIMIT`/`OFFSET`; the API caps results at 1000 rows per request.
	Query string
	// The blockchain network identifier (e.g. `"hyperliquid-core-mainnet"`).
	ClusterId string
}

func (r *QueryParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Query)
	FfiDestroyerString{}.Destroy(r.ClusterId)
}

type FfiConverterQueryParams struct{}

var FfiConverterQueryParamsINSTANCE = FfiConverterQueryParams{}

func (c FfiConverterQueryParams) Lift(rb RustBufferI) QueryParams {
	return LiftFromRustBuffer[QueryParams](c, rb)
}

func (c FfiConverterQueryParams) Read(reader io.Reader) QueryParams {
	return QueryParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterQueryParams) Lower(value QueryParams) C.RustBuffer {
	return LowerIntoRustBuffer[QueryParams](c, value)
}

func (c FfiConverterQueryParams) LowerExternal(value QueryParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[QueryParams](c, value))
}

func (c FfiConverterQueryParams) Write(writer io.Writer, value QueryParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Query)
	FfiConverterStringINSTANCE.Write(writer, value.ClusterId)
}

type FfiDestroyerQueryParams struct{}

func (_ FfiDestroyerQueryParams) Destroy(value QueryParams) {
	value.Destroy()
}

// Response from `query`.
type QueryResponse struct {
	// Column metadata for each column in the result set.
	Meta []ColumnMeta
	// Result rows. Each row is a JSON object whose keys are the selected
	// columns; shape varies per query.
	Data []JsonValue
	// Number of rows returned in this response.
	Rows int64
	// Total rows that matched the query before applying `LIMIT`; use for
	// pagination.
	RowsBeforeLimitAtLeast int64
	// Query execution statistics.
	Statistics QueryStatistics
	// Credits consumed by the query.
	Credits int64
}

func (r *QueryResponse) Destroy() {
	FfiDestroyerSequenceColumnMeta{}.Destroy(r.Meta)
	FfiDestroyerSequenceTypeJsonValue{}.Destroy(r.Data)
	FfiDestroyerInt64{}.Destroy(r.Rows)
	FfiDestroyerInt64{}.Destroy(r.RowsBeforeLimitAtLeast)
	FfiDestroyerQueryStatistics{}.Destroy(r.Statistics)
	FfiDestroyerInt64{}.Destroy(r.Credits)
}

type FfiConverterQueryResponse struct{}

var FfiConverterQueryResponseINSTANCE = FfiConverterQueryResponse{}

func (c FfiConverterQueryResponse) Lift(rb RustBufferI) QueryResponse {
	return LiftFromRustBuffer[QueryResponse](c, rb)
}

func (c FfiConverterQueryResponse) Read(reader io.Reader) QueryResponse {
	return QueryResponse{
		FfiConverterSequenceColumnMetaINSTANCE.Read(reader),
		FfiConverterSequenceTypeJsonValueINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterQueryStatisticsINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterQueryResponse) Lower(value QueryResponse) C.RustBuffer {
	return LowerIntoRustBuffer[QueryResponse](c, value)
}

func (c FfiConverterQueryResponse) LowerExternal(value QueryResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[QueryResponse](c, value))
}

func (c FfiConverterQueryResponse) Write(writer io.Writer, value QueryResponse) {
	FfiConverterSequenceColumnMetaINSTANCE.Write(writer, value.Meta)
	FfiConverterSequenceTypeJsonValueINSTANCE.Write(writer, value.Data)
	FfiConverterInt64INSTANCE.Write(writer, value.Rows)
	FfiConverterInt64INSTANCE.Write(writer, value.RowsBeforeLimitAtLeast)
	FfiConverterQueryStatisticsINSTANCE.Write(writer, value.Statistics)
	FfiConverterInt64INSTANCE.Write(writer, value.Credits)
}

type FfiDestroyerQueryResponse struct{}

func (_ FfiDestroyerQueryResponse) Destroy(value QueryResponse) {
	value.Destroy()
}

// Execution statistics returned alongside query results.
type QueryStatistics struct {
	// Total query execution time in seconds.
	Elapsed float64
	// Total number of rows scanned during execution.
	RowsRead int64
	// Total data scanned in bytes.
	BytesRead int64
}

func (r *QueryStatistics) Destroy() {
	FfiDestroyerFloat64{}.Destroy(r.Elapsed)
	FfiDestroyerInt64{}.Destroy(r.RowsRead)
	FfiDestroyerInt64{}.Destroy(r.BytesRead)
}

type FfiConverterQueryStatistics struct{}

var FfiConverterQueryStatisticsINSTANCE = FfiConverterQueryStatistics{}

func (c FfiConverterQueryStatistics) Lift(rb RustBufferI) QueryStatistics {
	return LiftFromRustBuffer[QueryStatistics](c, rb)
}

func (c FfiConverterQueryStatistics) Read(reader io.Reader) QueryStatistics {
	return QueryStatistics{
		FfiConverterFloat64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterQueryStatistics) Lower(value QueryStatistics) C.RustBuffer {
	return LowerIntoRustBuffer[QueryStatistics](c, value)
}

func (c FfiConverterQueryStatistics) LowerExternal(value QueryStatistics) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[QueryStatistics](c, value))
}

func (c FfiConverterQueryStatistics) Write(writer io.Writer, value QueryStatistics) {
	FfiConverterFloat64INSTANCE.Write(writer, value.Elapsed)
	FfiConverterInt64INSTANCE.Write(writer, value.RowsRead)
	FfiConverterInt64INSTANCE.Write(writer, value.BytesRead)
}

type FfiDestroyerQueryStatistics struct{}

func (_ FfiDestroyerQueryStatistics) Destroy(value QueryStatistics) {
	value.Destroy()
}

// A single rate-limit row returned by `get_rate_limits`, identifying the
// bucket (`rps`/`rpm`/`rpd`), the value enforced, and whether the value comes
// from the plan default or a user-set override.
type RateLimitEntry struct {
	// Which bucket this row applies to: `rps`, `rpm`, or `rpd`.
	Bucket string
	// The enforced value for this bucket.
	RateLimit int32
	// Where the value comes from: `plan_default` or `user_override`.
	Source string
	// Row identifier. Present on `user_override` rows — pass it to
	// `delete_rate_limit_override` to remove the override. May be absent on
	// `plan_default` rows and cannot be deleted there.
	Id *string
}

func (r *RateLimitEntry) Destroy() {
	FfiDestroyerString{}.Destroy(r.Bucket)
	FfiDestroyerInt32{}.Destroy(r.RateLimit)
	FfiDestroyerString{}.Destroy(r.Source)
	FfiDestroyerOptionalString{}.Destroy(r.Id)
}

type FfiConverterRateLimitEntry struct{}

var FfiConverterRateLimitEntryINSTANCE = FfiConverterRateLimitEntry{}

func (c FfiConverterRateLimitEntry) Lift(rb RustBufferI) RateLimitEntry {
	return LiftFromRustBuffer[RateLimitEntry](c, rb)
}

func (c FfiConverterRateLimitEntry) Read(reader io.Reader) RateLimitEntry {
	return RateLimitEntry{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterRateLimitEntry) Lower(value RateLimitEntry) C.RustBuffer {
	return LowerIntoRustBuffer[RateLimitEntry](c, value)
}

func (c FfiConverterRateLimitEntry) LowerExternal(value RateLimitEntry) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RateLimitEntry](c, value))
}

func (c FfiConverterRateLimitEntry) Write(writer io.Writer, value RateLimitEntry) {
	FfiConverterStringINSTANCE.Write(writer, value.Bucket)
	FfiConverterInt32INSTANCE.Write(writer, value.RateLimit)
	FfiConverterStringINSTANCE.Write(writer, value.Source)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Id)
}

type FfiDestroyerRateLimitEntry struct{}

func (_ FfiDestroyerRateLimitEntry) Destroy(value RateLimitEntry) {
	value.Destroy()
}

// Endpoint-wide rate limit settings.
type RateLimitSettings struct {
	// Requests per second.
	Rps *int32
	// Requests per minute.
	Rpm *int32
	// Requests per day.
	Rpd *int32
}

func (r *RateLimitSettings) Destroy() {
	FfiDestroyerOptionalInt32{}.Destroy(r.Rps)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rpm)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rpd)
}

type FfiConverterRateLimitSettings struct{}

var FfiConverterRateLimitSettingsINSTANCE = FfiConverterRateLimitSettings{}

func (c FfiConverterRateLimitSettings) Lift(rb RustBufferI) RateLimitSettings {
	return LiftFromRustBuffer[RateLimitSettings](c, rb)
}

func (c FfiConverterRateLimitSettings) Read(reader io.Reader) RateLimitSettings {
	return RateLimitSettings{
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterRateLimitSettings) Lower(value RateLimitSettings) C.RustBuffer {
	return LowerIntoRustBuffer[RateLimitSettings](c, value)
}

func (c FfiConverterRateLimitSettings) LowerExternal(value RateLimitSettings) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RateLimitSettings](c, value))
}

func (c FfiConverterRateLimitSettings) Write(writer io.Writer, value RateLimitSettings) {
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rps)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rpm)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rpd)
}

type FfiDestroyerRateLimitSettings struct{}

func (_ FfiDestroyerRateLimitSettings) Destroy(value RateLimitSettings) {
	value.Destroy()
}

// Parameters for `remove_team_member`.
type RemoveTeamMemberRequest struct {
	// When true, also delete the user entirely rather than just removing them from the team.
	DestroyUser *bool
}

func (r *RemoveTeamMemberRequest) Destroy() {
	FfiDestroyerOptionalBool{}.Destroy(r.DestroyUser)
}

type FfiConverterRemoveTeamMemberRequest struct{}

var FfiConverterRemoveTeamMemberRequestINSTANCE = FfiConverterRemoveTeamMemberRequest{}

func (c FfiConverterRemoveTeamMemberRequest) Lift(rb RustBufferI) RemoveTeamMemberRequest {
	return LiftFromRustBuffer[RemoveTeamMemberRequest](c, rb)
}

func (c FfiConverterRemoveTeamMemberRequest) Read(reader io.Reader) RemoveTeamMemberRequest {
	return RemoveTeamMemberRequest{
		FfiConverterOptionalBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterRemoveTeamMemberRequest) Lower(value RemoveTeamMemberRequest) C.RustBuffer {
	return LowerIntoRustBuffer[RemoveTeamMemberRequest](c, value)
}

func (c FfiConverterRemoveTeamMemberRequest) LowerExternal(value RemoveTeamMemberRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RemoveTeamMemberRequest](c, value))
}

func (c FfiConverterRemoveTeamMemberRequest) Write(writer io.Writer, value RemoveTeamMemberRequest) {
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.DestroyUser)
}

type FfiDestroyerRemoveTeamMemberRequest struct{}

func (_ FfiDestroyerRemoveTeamMemberRequest) Destroy(value RemoveTeamMemberRequest) {
	value.Destroy()
}

// Response from `remove_team_member`.
type RemoveTeamMemberResponse struct {
	// Operation result message.
	Data *TeamMessageData
	// Error message when the request did not succeed.
	Error *string
}

func (r *RemoveTeamMemberResponse) Destroy() {
	FfiDestroyerOptionalTeamMessageData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterRemoveTeamMemberResponse struct{}

var FfiConverterRemoveTeamMemberResponseINSTANCE = FfiConverterRemoveTeamMemberResponse{}

func (c FfiConverterRemoveTeamMemberResponse) Lift(rb RustBufferI) RemoveTeamMemberResponse {
	return LiftFromRustBuffer[RemoveTeamMemberResponse](c, rb)
}

func (c FfiConverterRemoveTeamMemberResponse) Read(reader io.Reader) RemoveTeamMemberResponse {
	return RemoveTeamMemberResponse{
		FfiConverterOptionalTeamMessageDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterRemoveTeamMemberResponse) Lower(value RemoveTeamMemberResponse) C.RustBuffer {
	return LowerIntoRustBuffer[RemoveTeamMemberResponse](c, value)
}

func (c FfiConverterRemoveTeamMemberResponse) LowerExternal(value RemoveTeamMemberResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RemoveTeamMemberResponse](c, value))
}

func (c FfiConverterRemoveTeamMemberResponse) Write(writer io.Writer, value RemoveTeamMemberResponse) {
	FfiConverterOptionalTeamMessageDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerRemoveTeamMemberResponse struct{}

func (_ FfiDestroyerRemoveTeamMemberResponse) Destroy(value RemoveTeamMemberResponse) {
	value.Destroy()
}

// Parameters for `rename_tag`.
type RenameTagRequest struct {
	// New label for the tag.
	Label string
}

func (r *RenameTagRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Label)
}

type FfiConverterRenameTagRequest struct{}

var FfiConverterRenameTagRequestINSTANCE = FfiConverterRenameTagRequest{}

func (c FfiConverterRenameTagRequest) Lift(rb RustBufferI) RenameTagRequest {
	return LiftFromRustBuffer[RenameTagRequest](c, rb)
}

func (c FfiConverterRenameTagRequest) Read(reader io.Reader) RenameTagRequest {
	return RenameTagRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterRenameTagRequest) Lower(value RenameTagRequest) C.RustBuffer {
	return LowerIntoRustBuffer[RenameTagRequest](c, value)
}

func (c FfiConverterRenameTagRequest) LowerExternal(value RenameTagRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RenameTagRequest](c, value))
}

func (c FfiConverterRenameTagRequest) Write(writer io.Writer, value RenameTagRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerRenameTagRequest struct{}

func (_ FfiDestroyerRenameTagRequest) Destroy(value RenameTagRequest) {
	value.Destroy()
}

// Response from `rename_tag`.
type RenameTagResponse struct {
	// The renamed tag.
	Data *AccountTag
	// Error message when the request did not succeed.
	Error *string
}

func (r *RenameTagResponse) Destroy() {
	FfiDestroyerOptionalAccountTag{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterRenameTagResponse struct{}

var FfiConverterRenameTagResponseINSTANCE = FfiConverterRenameTagResponse{}

func (c FfiConverterRenameTagResponse) Lift(rb RustBufferI) RenameTagResponse {
	return LiftFromRustBuffer[RenameTagResponse](c, rb)
}

func (c FfiConverterRenameTagResponse) Read(reader io.Reader) RenameTagResponse {
	return RenameTagResponse{
		FfiConverterOptionalAccountTagINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterRenameTagResponse) Lower(value RenameTagResponse) C.RustBuffer {
	return LowerIntoRustBuffer[RenameTagResponse](c, value)
}

func (c FfiConverterRenameTagResponse) LowerExternal(value RenameTagResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RenameTagResponse](c, value))
}

func (c FfiConverterRenameTagResponse) Write(writer io.Writer, value RenameTagResponse) {
	FfiConverterOptionalAccountTagINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerRenameTagResponse struct{}

func (_ FfiDestroyerRenameTagResponse) Destroy(value RenameTagResponse) {
	value.Destroy()
}

// Response from `resend_team_invite`.
type ResendTeamInviteResponse struct {
	// Operation result message.
	Data *TeamMessageData
	// Error message when the request did not succeed.
	Error *string
}

func (r *ResendTeamInviteResponse) Destroy() {
	FfiDestroyerOptionalTeamMessageData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterResendTeamInviteResponse struct{}

var FfiConverterResendTeamInviteResponseINSTANCE = FfiConverterResendTeamInviteResponse{}

func (c FfiConverterResendTeamInviteResponse) Lift(rb RustBufferI) ResendTeamInviteResponse {
	return LiftFromRustBuffer[ResendTeamInviteResponse](c, rb)
}

func (c FfiConverterResendTeamInviteResponse) Read(reader io.Reader) ResendTeamInviteResponse {
	return ResendTeamInviteResponse{
		FfiConverterOptionalTeamMessageDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterResendTeamInviteResponse) Lower(value ResendTeamInviteResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ResendTeamInviteResponse](c, value)
}

func (c FfiConverterResendTeamInviteResponse) LowerExternal(value ResendTeamInviteResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ResendTeamInviteResponse](c, value))
}

func (c FfiConverterResendTeamInviteResponse) Write(writer io.Writer, value ResendTeamInviteResponse) {
	FfiConverterOptionalTeamMessageDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerResendTeamInviteResponse struct{}

func (_ FfiDestroyerResendTeamInviteResponse) Destroy(value ResendTeamInviteResponse) {
	value.Destroy()
}

// Configuration for delivering stream batches to an S3-compatible object store.
type S3Attributes struct {
	// S3 service endpoint (e.g. `s3.amazonaws.com`).
	Endpoint string
	// Access key used to authenticate with the S3 endpoint.
	AccessKey string
	// Secret key used to authenticate with the S3 endpoint.
	SecretKey string
	// Target bucket name.
	Bucket string
	// Key prefix prepended to each written object.
	ObjectPrefix string
	// Compression applied to written objects (e.g. `none`, `gzip`).
	Compression string
	// File format/extension for written objects (e.g. `.json`).
	FileType string
	// Maximum number of retry attempts for a failed write.
	MaxRetry int32
	// Seconds to wait between retry attempts.
	RetryIntervalSec int32
	// Whether to use TLS when connecting to the endpoint.
	UseSsl *bool
}

func (r *S3Attributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.Endpoint)
	FfiDestroyerString{}.Destroy(r.AccessKey)
	FfiDestroyerString{}.Destroy(r.SecretKey)
	FfiDestroyerString{}.Destroy(r.Bucket)
	FfiDestroyerString{}.Destroy(r.ObjectPrefix)
	FfiDestroyerString{}.Destroy(r.Compression)
	FfiDestroyerString{}.Destroy(r.FileType)
	FfiDestroyerInt32{}.Destroy(r.MaxRetry)
	FfiDestroyerInt32{}.Destroy(r.RetryIntervalSec)
	FfiDestroyerOptionalBool{}.Destroy(r.UseSsl)
}

type FfiConverterS3Attributes struct{}

var FfiConverterS3AttributesINSTANCE = FfiConverterS3Attributes{}

func (c FfiConverterS3Attributes) Lift(rb RustBufferI) S3Attributes {
	return LiftFromRustBuffer[S3Attributes](c, rb)
}

func (c FfiConverterS3Attributes) Read(reader io.Reader) S3Attributes {
	return S3Attributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterS3Attributes) Lower(value S3Attributes) C.RustBuffer {
	return LowerIntoRustBuffer[S3Attributes](c, value)
}

func (c FfiConverterS3Attributes) LowerExternal(value S3Attributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[S3Attributes](c, value))
}

func (c FfiConverterS3Attributes) Write(writer io.Writer, value S3Attributes) {
	FfiConverterStringINSTANCE.Write(writer, value.Endpoint)
	FfiConverterStringINSTANCE.Write(writer, value.AccessKey)
	FfiConverterStringINSTANCE.Write(writer, value.SecretKey)
	FfiConverterStringINSTANCE.Write(writer, value.Bucket)
	FfiConverterStringINSTANCE.Write(writer, value.ObjectPrefix)
	FfiConverterStringINSTANCE.Write(writer, value.Compression)
	FfiConverterStringINSTANCE.Write(writer, value.FileType)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxRetry)
	FfiConverterInt32INSTANCE.Write(writer, value.RetryIntervalSec)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.UseSsl)
}

type FfiDestroyerS3Attributes struct{}

func (_ FfiDestroyerS3Attributes) Destroy(value S3Attributes) {
	value.Destroy()
}

// A single security feature's name, status, and optional value.
type SecurityOption struct {
	// Name of the security feature (e.g. `tokens`, `jwts`, `ips`).
	Option string
	// Whether the feature is `enabled` or `disabled`.
	Status string
	// Optional configuration value associated with the feature.
	Value *string
}

func (r *SecurityOption) Destroy() {
	FfiDestroyerString{}.Destroy(r.Option)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerOptionalString{}.Destroy(r.Value)
}

type FfiConverterSecurityOption struct{}

var FfiConverterSecurityOptionINSTANCE = FfiConverterSecurityOption{}

func (c FfiConverterSecurityOption) Lift(rb RustBufferI) SecurityOption {
	return LiftFromRustBuffer[SecurityOption](c, rb)
}

func (c FfiConverterSecurityOption) Read(reader io.Reader) SecurityOption {
	return SecurityOption{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterSecurityOption) Lower(value SecurityOption) C.RustBuffer {
	return LowerIntoRustBuffer[SecurityOption](c, value)
}

func (c FfiConverterSecurityOption) LowerExternal(value SecurityOption) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SecurityOption](c, value))
}

func (c FfiConverterSecurityOption) Write(writer io.Writer, value SecurityOption) {
	FfiConverterStringINSTANCE.Write(writer, value.Option)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Value)
}

type FfiDestroyerSecurityOption struct{}

func (_ FfiDestroyerSecurityOption) Destroy(value SecurityOption) {
	value.Destroy()
}

// Per-feature toggles for `update_security_options`. Each field accepts
// `enabled` or `disabled`.
type SecurityOptionsUpdate struct {
	// Token authentication toggle.
	Tokens *string
	// Referrer validation toggle.
	Referrers *string
	// JWT validation toggle.
	Jwts *string
	// IP whitelist toggle.
	Ips *string
	// Domain masking toggle.
	DomainMasks *string
	// HSTS (HTTP Strict Transport Security) toggle.
	Hsts *string
	// CORS toggle.
	Cors *string
	// Request (method) filter toggle.
	RequestFilters *string
	// Custom IP header toggle.
	IpCustomHeader *string
}

func (r *SecurityOptionsUpdate) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Tokens)
	FfiDestroyerOptionalString{}.Destroy(r.Referrers)
	FfiDestroyerOptionalString{}.Destroy(r.Jwts)
	FfiDestroyerOptionalString{}.Destroy(r.Ips)
	FfiDestroyerOptionalString{}.Destroy(r.DomainMasks)
	FfiDestroyerOptionalString{}.Destroy(r.Hsts)
	FfiDestroyerOptionalString{}.Destroy(r.Cors)
	FfiDestroyerOptionalString{}.Destroy(r.RequestFilters)
	FfiDestroyerOptionalString{}.Destroy(r.IpCustomHeader)
}

type FfiConverterSecurityOptionsUpdate struct{}

var FfiConverterSecurityOptionsUpdateINSTANCE = FfiConverterSecurityOptionsUpdate{}

func (c FfiConverterSecurityOptionsUpdate) Lift(rb RustBufferI) SecurityOptionsUpdate {
	return LiftFromRustBuffer[SecurityOptionsUpdate](c, rb)
}

func (c FfiConverterSecurityOptionsUpdate) Read(reader io.Reader) SecurityOptionsUpdate {
	return SecurityOptionsUpdate{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterSecurityOptionsUpdate) Lower(value SecurityOptionsUpdate) C.RustBuffer {
	return LowerIntoRustBuffer[SecurityOptionsUpdate](c, value)
}

func (c FfiConverterSecurityOptionsUpdate) LowerExternal(value SecurityOptionsUpdate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SecurityOptionsUpdate](c, value))
}

func (c FfiConverterSecurityOptionsUpdate) Write(writer io.Writer, value SecurityOptionsUpdate) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Tokens)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Referrers)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Jwts)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Ips)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.DomainMasks)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Hsts)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Cors)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.RequestFilters)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.IpCustomHeader)
}

type FfiDestroyerSecurityOptionsUpdate struct{}

func (_ FfiDestroyerSecurityOptionsUpdate) Destroy(value SecurityOptionsUpdate) {
	value.Destroy()
}

// Response from `show_endpoint`.
type ShowEndpointResponse struct {
	// The endpoint, when found.
	Data *SingleEndpoint
	// Error message when the request did not succeed.
	Error *string
}

func (r *ShowEndpointResponse) Destroy() {
	FfiDestroyerOptionalSingleEndpoint{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterShowEndpointResponse struct{}

var FfiConverterShowEndpointResponseINSTANCE = FfiConverterShowEndpointResponse{}

func (c FfiConverterShowEndpointResponse) Lift(rb RustBufferI) ShowEndpointResponse {
	return LiftFromRustBuffer[ShowEndpointResponse](c, rb)
}

func (c FfiConverterShowEndpointResponse) Read(reader io.Reader) ShowEndpointResponse {
	return ShowEndpointResponse{
		FfiConverterOptionalSingleEndpointINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterShowEndpointResponse) Lower(value ShowEndpointResponse) C.RustBuffer {
	return LowerIntoRustBuffer[ShowEndpointResponse](c, value)
}

func (c FfiConverterShowEndpointResponse) LowerExternal(value ShowEndpointResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ShowEndpointResponse](c, value))
}

func (c FfiConverterShowEndpointResponse) Write(writer io.Writer, value ShowEndpointResponse) {
	FfiConverterOptionalSingleEndpointINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerShowEndpointResponse struct{}

func (_ FfiDestroyerShowEndpointResponse) Destroy(value ShowEndpointResponse) {
	value.Destroy()
}

// Full representation of a single endpoint, including its security and rate limits.
type SingleEndpoint struct {
	// Unique endpoint identifier.
	Id string
	// Human-readable label.
	Label *string
	// Current operational status.
	Status *string
	// Blockchain the endpoint serves.
	Chain string
	// Specific network within the chain.
	Network string
	// HTTP RPC URL.
	HttpUrl string
	// WebSocket RPC URL, when available.
	WssUrl *string
	// Endpoint security configuration.
	Security *EndpointSecurity
	// Endpoint rate limits.
	RateLimits *EndpointRateLimits
	// Tags applied to the endpoint.
	Tags []EndpointTag
	// Whether the endpoint is configured to serve multiple chains/networks.
	IsMultichain bool
}

func (r *SingleEndpoint) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerOptionalString{}.Destroy(r.Label)
	FfiDestroyerOptionalString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.Chain)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerString{}.Destroy(r.HttpUrl)
	FfiDestroyerOptionalString{}.Destroy(r.WssUrl)
	FfiDestroyerOptionalEndpointSecurity{}.Destroy(r.Security)
	FfiDestroyerOptionalEndpointRateLimits{}.Destroy(r.RateLimits)
	FfiDestroyerSequenceEndpointTag{}.Destroy(r.Tags)
	FfiDestroyerBool{}.Destroy(r.IsMultichain)
}

type FfiConverterSingleEndpoint struct{}

var FfiConverterSingleEndpointINSTANCE = FfiConverterSingleEndpoint{}

func (c FfiConverterSingleEndpoint) Lift(rb RustBufferI) SingleEndpoint {
	return LiftFromRustBuffer[SingleEndpoint](c, rb)
}

func (c FfiConverterSingleEndpoint) Read(reader io.Reader) SingleEndpoint {
	return SingleEndpoint{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalEndpointSecurityINSTANCE.Read(reader),
		FfiConverterOptionalEndpointRateLimitsINSTANCE.Read(reader),
		FfiConverterSequenceEndpointTagINSTANCE.Read(reader),
		FfiConverterBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterSingleEndpoint) Lower(value SingleEndpoint) C.RustBuffer {
	return LowerIntoRustBuffer[SingleEndpoint](c, value)
}

func (c FfiConverterSingleEndpoint) LowerExternal(value SingleEndpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SingleEndpoint](c, value))
}

func (c FfiConverterSingleEndpoint) Write(writer io.Writer, value SingleEndpoint) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.Chain)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterStringINSTANCE.Write(writer, value.HttpUrl)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.WssUrl)
	FfiConverterOptionalEndpointSecurityINSTANCE.Write(writer, value.Security)
	FfiConverterOptionalEndpointRateLimitsINSTANCE.Write(writer, value.RateLimits)
	FfiConverterSequenceEndpointTagINSTANCE.Write(writer, value.Tags)
	FfiConverterBoolINSTANCE.Write(writer, value.IsMultichain)
}

type FfiDestroyerSingleEndpoint struct{}

func (_ FfiDestroyerSingleEndpoint) Destroy(value SingleEndpoint) {
	value.Destroy()
}

// ByList form of `SolanaWalletFilterTemplate`.
type SolanaWalletFilterByListTemplate struct {
	// Name of the pre-created accounts list.
	AccountsListName string
}

func (r *SolanaWalletFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.AccountsListName)
}

type FfiConverterSolanaWalletFilterByListTemplate struct{}

var FfiConverterSolanaWalletFilterByListTemplateINSTANCE = FfiConverterSolanaWalletFilterByListTemplate{}

func (c FfiConverterSolanaWalletFilterByListTemplate) Lift(rb RustBufferI) SolanaWalletFilterByListTemplate {
	return LiftFromRustBuffer[SolanaWalletFilterByListTemplate](c, rb)
}

func (c FfiConverterSolanaWalletFilterByListTemplate) Read(reader io.Reader) SolanaWalletFilterByListTemplate {
	return SolanaWalletFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterSolanaWalletFilterByListTemplate) Lower(value SolanaWalletFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[SolanaWalletFilterByListTemplate](c, value)
}

func (c FfiConverterSolanaWalletFilterByListTemplate) LowerExternal(value SolanaWalletFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SolanaWalletFilterByListTemplate](c, value))
}

func (c FfiConverterSolanaWalletFilterByListTemplate) Write(writer io.Writer, value SolanaWalletFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.AccountsListName)
}

type FfiDestroyerSolanaWalletFilterByListTemplate struct{}

func (_ FfiDestroyerSolanaWalletFilterByListTemplate) Destroy(value SolanaWalletFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for a Solana wallet filter: matches activity for a list
// of Solana account addresses.
type SolanaWalletFilterTemplate struct {
	// Solana account addresses to match against.
	Accounts []string
}

func (r *SolanaWalletFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Accounts)
}

type FfiConverterSolanaWalletFilterTemplate struct{}

var FfiConverterSolanaWalletFilterTemplateINSTANCE = FfiConverterSolanaWalletFilterTemplate{}

func (c FfiConverterSolanaWalletFilterTemplate) Lift(rb RustBufferI) SolanaWalletFilterTemplate {
	return LiftFromRustBuffer[SolanaWalletFilterTemplate](c, rb)
}

func (c FfiConverterSolanaWalletFilterTemplate) Read(reader io.Reader) SolanaWalletFilterTemplate {
	return SolanaWalletFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterSolanaWalletFilterTemplate) Lower(value SolanaWalletFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[SolanaWalletFilterTemplate](c, value)
}

func (c FfiConverterSolanaWalletFilterTemplate) LowerExternal(value SolanaWalletFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SolanaWalletFilterTemplate](c, value))
}

func (c FfiConverterSolanaWalletFilterTemplate) Write(writer io.Writer, value SolanaWalletFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Accounts)
}

type FfiDestroyerSolanaWalletFilterTemplate struct{}

func (_ FfiDestroyerSolanaWalletFilterTemplate) Destroy(value SolanaWalletFilterTemplate) {
	value.Destroy()
}

// ByList form of `StellarWalletTransactionsFilterTemplate`.
type StellarWalletTransactionsFilterByListTemplate struct {
	// Name of the pre-created wallets list.
	WalletsListName string
}

func (r *StellarWalletTransactionsFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.WalletsListName)
}

type FfiConverterStellarWalletTransactionsFilterByListTemplate struct{}

var FfiConverterStellarWalletTransactionsFilterByListTemplateINSTANCE = FfiConverterStellarWalletTransactionsFilterByListTemplate{}

func (c FfiConverterStellarWalletTransactionsFilterByListTemplate) Lift(rb RustBufferI) StellarWalletTransactionsFilterByListTemplate {
	return LiftFromRustBuffer[StellarWalletTransactionsFilterByListTemplate](c, rb)
}

func (c FfiConverterStellarWalletTransactionsFilterByListTemplate) Read(reader io.Reader) StellarWalletTransactionsFilterByListTemplate {
	return StellarWalletTransactionsFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterStellarWalletTransactionsFilterByListTemplate) Lower(value StellarWalletTransactionsFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[StellarWalletTransactionsFilterByListTemplate](c, value)
}

func (c FfiConverterStellarWalletTransactionsFilterByListTemplate) LowerExternal(value StellarWalletTransactionsFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StellarWalletTransactionsFilterByListTemplate](c, value))
}

func (c FfiConverterStellarWalletTransactionsFilterByListTemplate) Write(writer io.Writer, value StellarWalletTransactionsFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.WalletsListName)
}

type FfiDestroyerStellarWalletTransactionsFilterByListTemplate struct{}

func (_ FfiDestroyerStellarWalletTransactionsFilterByListTemplate) Destroy(value StellarWalletTransactionsFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for a Stellar wallet-transactions filter, matching
// transactions where the given wallets are the source account.
type StellarWalletTransactionsFilterTemplate struct {
	// Stellar wallet addresses to match against.
	Wallets []string
}

func (r *StellarWalletTransactionsFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Wallets)
}

type FfiConverterStellarWalletTransactionsFilterTemplate struct{}

var FfiConverterStellarWalletTransactionsFilterTemplateINSTANCE = FfiConverterStellarWalletTransactionsFilterTemplate{}

func (c FfiConverterStellarWalletTransactionsFilterTemplate) Lift(rb RustBufferI) StellarWalletTransactionsFilterTemplate {
	return LiftFromRustBuffer[StellarWalletTransactionsFilterTemplate](c, rb)
}

func (c FfiConverterStellarWalletTransactionsFilterTemplate) Read(reader io.Reader) StellarWalletTransactionsFilterTemplate {
	return StellarWalletTransactionsFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterStellarWalletTransactionsFilterTemplate) Lower(value StellarWalletTransactionsFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[StellarWalletTransactionsFilterTemplate](c, value)
}

func (c FfiConverterStellarWalletTransactionsFilterTemplate) LowerExternal(value StellarWalletTransactionsFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StellarWalletTransactionsFilterTemplate](c, value))
}

func (c FfiConverterStellarWalletTransactionsFilterTemplate) Write(writer io.Writer, value StellarWalletTransactionsFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Wallets)
}

type FfiDestroyerStellarWalletTransactionsFilterTemplate struct{}

func (_ FfiDestroyerStellarWalletTransactionsFilterTemplate) Destroy(value StellarWalletTransactionsFilterTemplate) {
	value.Destroy()
}

// A stream's full configuration and current state, as returned by the API.
type Stream struct {
	// Unique stream identifier.
	Id string
	// Human-readable stream name.
	Name string
	// Current operational state (e.g. `active`, `paused`).
	Status string
	// Timestamp when the stream was created.
	CreatedAt string
	// Timestamp of the most recent modification.
	UpdatedAt string
	// Sequence number tracking stream progress.
	Sequence int64
	// Blockchain network the stream is reading from.
	Network string
	// Dataset being streamed.
	Dataset string
	// Geographic region where the stream runs.
	Region string
	// Starting block for the stream.
	StartRange int64
	// Ending block for the stream; `-1` indicates continuous operation.
	EndRange int64
	// Billing plan associated with the stream.
	Plan *string
	// Buffer size used by the stream fetcher before delivery.
	ThresholdFetchBuffer *int64
	// Number of blocks grouped together per delivered batch.
	DatasetBatchSize *int64
	// Upper bound on batch size when elastic batching is enabled.
	MaxBatchSize *int64
	// Maximum number of buffered blocks waiting to be processed.
	MaxBufferRangeSize *int64
	// Maximum number of worker threads processing buffered batches.
	MaxBufferProcessingWorkers *int64
	// Number of blocks the stream stays behind the chain tip.
	KeepDistanceFromTip *int64
	// Base64-encoded filter function applied to each batch.
	FilterFunction *string
	// Language the filter function is written in.
	FilterLanguage *string
	// Where stream metadata is included in delivered payloads.
	IncludeStreamMetadata *string
	// Billing product type the stream is associated with.
	ProductType *string
	// Email address notified of stream termination or failure.
	NotificationEmail *string
	// Whether chain-reorg handling is enabled (0 or 1).
	FixBlockReorgs *int32
	// Most recent block hash processed by the stream.
	CurrentHash *string
	// Destination-specific configuration (present on single-stream responses).
	DestinationAttributes *DestinationAttributes
	// Whether elastic batching is active.
	ElasticBatchEnabled *bool
	// Quicknode account ID that owns the stream.
	QnAccountId *string
	// Minimum charge cap applied to the stream's billing.
	ChargeMinCap *int32
	// Free-text memo attached to the stream.
	Memo *string
	// Address book linked to the stream's filter, if any.
	AddressBookConfig *AddressBookConfig
	// Additional destinations receiving the same batches alongside the primary.
	ExtraDestinations *[]DestinationAttributes
}

func (r *Stream) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.CreatedAt)
	FfiDestroyerString{}.Destroy(r.UpdatedAt)
	FfiDestroyerInt64{}.Destroy(r.Sequence)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerString{}.Destroy(r.Dataset)
	FfiDestroyerString{}.Destroy(r.Region)
	FfiDestroyerInt64{}.Destroy(r.StartRange)
	FfiDestroyerInt64{}.Destroy(r.EndRange)
	FfiDestroyerOptionalString{}.Destroy(r.Plan)
	FfiDestroyerOptionalInt64{}.Destroy(r.ThresholdFetchBuffer)
	FfiDestroyerOptionalInt64{}.Destroy(r.DatasetBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferRangeSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferProcessingWorkers)
	FfiDestroyerOptionalInt64{}.Destroy(r.KeepDistanceFromTip)
	FfiDestroyerOptionalString{}.Destroy(r.FilterFunction)
	FfiDestroyerOptionalString{}.Destroy(r.FilterLanguage)
	FfiDestroyerOptionalString{}.Destroy(r.IncludeStreamMetadata)
	FfiDestroyerOptionalString{}.Destroy(r.ProductType)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalInt32{}.Destroy(r.FixBlockReorgs)
	FfiDestroyerOptionalString{}.Destroy(r.CurrentHash)
	FfiDestroyerOptionalDestinationAttributes{}.Destroy(r.DestinationAttributes)
	FfiDestroyerOptionalBool{}.Destroy(r.ElasticBatchEnabled)
	FfiDestroyerOptionalString{}.Destroy(r.QnAccountId)
	FfiDestroyerOptionalInt32{}.Destroy(r.ChargeMinCap)
	FfiDestroyerOptionalString{}.Destroy(r.Memo)
	FfiDestroyerOptionalAddressBookConfig{}.Destroy(r.AddressBookConfig)
	FfiDestroyerOptionalSequenceDestinationAttributes{}.Destroy(r.ExtraDestinations)
}

type FfiConverterStream struct{}

var FfiConverterStreamINSTANCE = FfiConverterStream{}

func (c FfiConverterStream) Lift(rb RustBufferI) Stream {
	return LiftFromRustBuffer[Stream](c, rb)
}

func (c FfiConverterStream) Read(reader io.Reader) Stream {
	return Stream{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalDestinationAttributesINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalAddressBookConfigINSTANCE.Read(reader),
		FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Read(reader),
	}
}

func (c FfiConverterStream) Lower(value Stream) C.RustBuffer {
	return LowerIntoRustBuffer[Stream](c, value)
}

func (c FfiConverterStream) LowerExternal(value Stream) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Stream](c, value))
}

func (c FfiConverterStream) Write(writer io.Writer, value Stream) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.CreatedAt)
	FfiConverterStringINSTANCE.Write(writer, value.UpdatedAt)
	FfiConverterInt64INSTANCE.Write(writer, value.Sequence)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterStringINSTANCE.Write(writer, value.Dataset)
	FfiConverterStringINSTANCE.Write(writer, value.Region)
	FfiConverterInt64INSTANCE.Write(writer, value.StartRange)
	FfiConverterInt64INSTANCE.Write(writer, value.EndRange)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Plan)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.ThresholdFetchBuffer)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.DatasetBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferRangeSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferProcessingWorkers)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.KeepDistanceFromTip)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FilterFunction)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FilterLanguage)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.IncludeStreamMetadata)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.ProductType)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.FixBlockReorgs)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.CurrentHash)
	FfiConverterOptionalDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.ElasticBatchEnabled)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.QnAccountId)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.ChargeMinCap)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Memo)
	FfiConverterOptionalAddressBookConfigINSTANCE.Write(writer, value.AddressBookConfig)
	FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Write(writer, value.ExtraDestinations)
}

type FfiDestroyerStream struct{}

func (_ FfiDestroyerStream) Destroy(value Stream) {
	value.Destroy()
}

// Schema for a single table.
type TableSchema struct {
	// Table name.
	Name string
	// Storage engine backing the table.
	Engine string
	// Approximate total number of rows in the table.
	TotalRows int64
	// Partition key expression; empty string for views.
	PartitionKey string
	// Sorting key columns; empty for views.
	SortingKey []string
	// Columns in the table.
	Columns []ColumnSchema
}

func (r *TableSchema) Destroy() {
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.Engine)
	FfiDestroyerInt64{}.Destroy(r.TotalRows)
	FfiDestroyerString{}.Destroy(r.PartitionKey)
	FfiDestroyerSequenceString{}.Destroy(r.SortingKey)
	FfiDestroyerSequenceColumnSchema{}.Destroy(r.Columns)
}

type FfiConverterTableSchema struct{}

var FfiConverterTableSchemaINSTANCE = FfiConverterTableSchema{}

func (c FfiConverterTableSchema) Lift(rb RustBufferI) TableSchema {
	return LiftFromRustBuffer[TableSchema](c, rb)
}

func (c FfiConverterTableSchema) Read(reader io.Reader) TableSchema {
	return TableSchema{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterSequenceColumnSchemaINSTANCE.Read(reader),
	}
}

func (c FfiConverterTableSchema) Lower(value TableSchema) C.RustBuffer {
	return LowerIntoRustBuffer[TableSchema](c, value)
}

func (c FfiConverterTableSchema) LowerExternal(value TableSchema) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TableSchema](c, value))
}

func (c FfiConverterTableSchema) Write(writer io.Writer, value TableSchema) {
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.Engine)
	FfiConverterInt64INSTANCE.Write(writer, value.TotalRows)
	FfiConverterStringINSTANCE.Write(writer, value.PartitionKey)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.SortingKey)
	FfiConverterSequenceColumnSchemaINSTANCE.Write(writer, value.Columns)
}

type FfiDestroyerTableSchema struct{}

func (_ FfiDestroyerTableSchema) Destroy(value TableSchema) {
	value.Destroy()
}

// Per-tag usage row.
type TagUsage struct {
	// Tag identifier.
	TagId *int32
	// Tag label.
	Label string
	// Credits consumed by endpoints with this tag.
	CreditsUsed int64
	// Request count during the window.
	Requests int64
}

func (r *TagUsage) Destroy() {
	FfiDestroyerOptionalInt32{}.Destroy(r.TagId)
	FfiDestroyerString{}.Destroy(r.Label)
	FfiDestroyerInt64{}.Destroy(r.CreditsUsed)
	FfiDestroyerInt64{}.Destroy(r.Requests)
}

type FfiConverterTagUsage struct{}

var FfiConverterTagUsageINSTANCE = FfiConverterTagUsage{}

func (c FfiConverterTagUsage) Lift(rb RustBufferI) TagUsage {
	return LiftFromRustBuffer[TagUsage](c, rb)
}

func (c FfiConverterTagUsage) Read(reader io.Reader) TagUsage {
	return TagUsage{
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterTagUsage) Lower(value TagUsage) C.RustBuffer {
	return LowerIntoRustBuffer[TagUsage](c, value)
}

func (c FfiConverterTagUsage) LowerExternal(value TagUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TagUsage](c, value))
}

func (c FfiConverterTagUsage) Write(writer io.Writer, value TagUsage) {
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.TagId)
	FfiConverterStringINSTANCE.Write(writer, value.Label)
	FfiConverterInt64INSTANCE.Write(writer, value.CreditsUsed)
	FfiConverterInt64INSTANCE.Write(writer, value.Requests)
}

type FfiDestroyerTagUsage struct{}

func (_ FfiDestroyerTagUsage) Destroy(value TagUsage) {
	value.Destroy()
}

// Full team detail including pending invites.
type TeamDetail struct {
	// Team identifier.
	Id int64
	// Team name.
	Name string
	// Default role assigned to newly invited members.
	DefaultRole *string
	// Current member count.
	MembersCount *int64
	// Active team members.
	Users []TeamUser
	// Invites that have not yet been accepted.
	PendingInvites []TeamUser
}

func (r *TeamDetail) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.DefaultRole)
	FfiDestroyerOptionalInt64{}.Destroy(r.MembersCount)
	FfiDestroyerSequenceTeamUser{}.Destroy(r.Users)
	FfiDestroyerSequenceTeamUser{}.Destroy(r.PendingInvites)
}

type FfiConverterTeamDetail struct{}

var FfiConverterTeamDetailINSTANCE = FfiConverterTeamDetail{}

func (c FfiConverterTeamDetail) Lift(rb RustBufferI) TeamDetail {
	return LiftFromRustBuffer[TeamDetail](c, rb)
}

func (c FfiConverterTeamDetail) Read(reader io.Reader) TeamDetail {
	return TeamDetail{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterSequenceTeamUserINSTANCE.Read(reader),
		FfiConverterSequenceTeamUserINSTANCE.Read(reader),
	}
}

func (c FfiConverterTeamDetail) Lower(value TeamDetail) C.RustBuffer {
	return LowerIntoRustBuffer[TeamDetail](c, value)
}

func (c FfiConverterTeamDetail) LowerExternal(value TeamDetail) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TeamDetail](c, value))
}

func (c FfiConverterTeamDetail) Write(writer io.Writer, value TeamDetail) {
	FfiConverterInt64INSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.DefaultRole)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MembersCount)
	FfiConverterSequenceTeamUserINSTANCE.Write(writer, value.Users)
	FfiConverterSequenceTeamUserINSTANCE.Write(writer, value.PendingInvites)
}

type FfiDestroyerTeamDetail struct{}

func (_ FfiDestroyerTeamDetail) Destroy(value TeamDetail) {
	value.Destroy()
}

// A team's endpoint association.
type TeamEndpoint struct {
	// Endpoint identifier.
	Id int64
	// Endpoint subdomain.
	Subdomain string
	// Blockchain the endpoint serves.
	Chain *string
	// Network within the chain.
	Network *string
}

func (r *TeamEndpoint) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Subdomain)
	FfiDestroyerOptionalString{}.Destroy(r.Chain)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
}

type FfiConverterTeamEndpoint struct{}

var FfiConverterTeamEndpointINSTANCE = FfiConverterTeamEndpoint{}

func (c FfiConverterTeamEndpoint) Lift(rb RustBufferI) TeamEndpoint {
	return LiftFromRustBuffer[TeamEndpoint](c, rb)
}

func (c FfiConverterTeamEndpoint) Read(reader io.Reader) TeamEndpoint {
	return TeamEndpoint{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterTeamEndpoint) Lower(value TeamEndpoint) C.RustBuffer {
	return LowerIntoRustBuffer[TeamEndpoint](c, value)
}

func (c FfiConverterTeamEndpoint) LowerExternal(value TeamEndpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TeamEndpoint](c, value))
}

func (c FfiConverterTeamEndpoint) Write(writer io.Writer, value TeamEndpoint) {
	FfiConverterInt64INSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Subdomain)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Chain)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
}

type FfiDestroyerTeamEndpoint struct{}

func (_ FfiDestroyerTeamEndpoint) Destroy(value TeamEndpoint) {
	value.Destroy()
}

// Shared message-shaped data wrapper for team operations.
type TeamMessageData struct {
	// Human-readable confirmation message.
	Message *string
}

func (r *TeamMessageData) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Message)
}

type FfiConverterTeamMessageData struct{}

var FfiConverterTeamMessageDataINSTANCE = FfiConverterTeamMessageData{}

func (c FfiConverterTeamMessageData) Lift(rb RustBufferI) TeamMessageData {
	return LiftFromRustBuffer[TeamMessageData](c, rb)
}

func (c FfiConverterTeamMessageData) Read(reader io.Reader) TeamMessageData {
	return TeamMessageData{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterTeamMessageData) Lower(value TeamMessageData) C.RustBuffer {
	return LowerIntoRustBuffer[TeamMessageData](c, value)
}

func (c FfiConverterTeamMessageData) LowerExternal(value TeamMessageData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TeamMessageData](c, value))
}

func (c FfiConverterTeamMessageData) Write(writer io.Writer, value TeamMessageData) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Message)
}

type FfiDestroyerTeamMessageData struct{}

func (_ FfiDestroyerTeamMessageData) Destroy(value TeamMessageData) {
	value.Destroy()
}

// Summary representation of a team in list responses.
type TeamSummary struct {
	// Team identifier.
	Id int64
	// Team name.
	Name string
	// Current member count.
	MembersCount *int64
	// Active team members.
	Users []TeamUser
}

func (r *TeamSummary) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerOptionalInt64{}.Destroy(r.MembersCount)
	FfiDestroyerSequenceTeamUser{}.Destroy(r.Users)
}

type FfiConverterTeamSummary struct{}

var FfiConverterTeamSummaryINSTANCE = FfiConverterTeamSummary{}

func (c FfiConverterTeamSummary) Lift(rb RustBufferI) TeamSummary {
	return LiftFromRustBuffer[TeamSummary](c, rb)
}

func (c FfiConverterTeamSummary) Read(reader io.Reader) TeamSummary {
	return TeamSummary{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterSequenceTeamUserINSTANCE.Read(reader),
	}
}

func (c FfiConverterTeamSummary) Lower(value TeamSummary) C.RustBuffer {
	return LowerIntoRustBuffer[TeamSummary](c, value)
}

func (c FfiConverterTeamSummary) LowerExternal(value TeamSummary) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TeamSummary](c, value))
}

func (c FfiConverterTeamSummary) Write(writer io.Writer, value TeamSummary) {
	FfiConverterInt64INSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MembersCount)
	FfiConverterSequenceTeamUserINSTANCE.Write(writer, value.Users)
}

type FfiDestroyerTeamSummary struct{}

func (_ FfiDestroyerTeamSummary) Destroy(value TeamSummary) {
	value.Destroy()
}

// A team member or pending invitee.
type TeamUser struct {
	// User identifier.
	Id int64
	// Display name.
	FullName *string
	// Email address.
	Email string
	// Team role (e.g. `admin`, `viewer`, `billing`).
	Role *string
	// Membership status (e.g. `active`, `pending`).
	Status *string
	// When the user was added.
	CreatedAt *string
	// Profile photo URL.
	PhotoUrl *string
	// Whether this user is the primary user on the account.
	AccountPrimaryUser *bool
}

func (r *TeamUser) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Id)
	FfiDestroyerOptionalString{}.Destroy(r.FullName)
	FfiDestroyerString{}.Destroy(r.Email)
	FfiDestroyerOptionalString{}.Destroy(r.Role)
	FfiDestroyerOptionalString{}.Destroy(r.Status)
	FfiDestroyerOptionalString{}.Destroy(r.CreatedAt)
	FfiDestroyerOptionalString{}.Destroy(r.PhotoUrl)
	FfiDestroyerOptionalBool{}.Destroy(r.AccountPrimaryUser)
}

type FfiConverterTeamUser struct{}

var FfiConverterTeamUserINSTANCE = FfiConverterTeamUser{}

func (c FfiConverterTeamUser) Lift(rb RustBufferI) TeamUser {
	return LiftFromRustBuffer[TeamUser](c, rb)
}

func (c FfiConverterTeamUser) Read(reader io.Reader) TeamUser {
	return TeamUser{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterTeamUser) Lower(value TeamUser) C.RustBuffer {
	return LowerIntoRustBuffer[TeamUser](c, value)
}

func (c FfiConverterTeamUser) LowerExternal(value TeamUser) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TeamUser](c, value))
}

func (c FfiConverterTeamUser) Write(writer io.Writer, value TeamUser) {
	FfiConverterInt64INSTANCE.Write(writer, value.Id)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FullName)
	FfiConverterStringINSTANCE.Write(writer, value.Email)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Role)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.CreatedAt)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.PhotoUrl)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.AccountPrimaryUser)
}

type FfiDestroyerTeamUser struct{}

func (_ FfiDestroyerTeamUser) Destroy(value TeamUser) {
	value.Destroy()
}

// Parameters for `test_filter`.
type TestFilterParams struct {
	// Blockchain network to run the test against (e.g. `ethereum-mainnet`).
	Network string
	// Dataset the filter operates on.
	Dataset StreamDataset
	// Specific block number to feed into the filter for the test.
	Block string
	// Base64-encoded filter function to evaluate. Required by the API. To inspect raw block data with no transformation, supply a base64-encoded identity function such as `function main(d){return d;}`.
	FilterFunction string
	// Language the filter function is written in.
	FilterLanguage *FilterLanguage
	// Address book linked to the filter, if any.
	AddressBookConfig *AddressBookConfig
}

func (r *TestFilterParams) Destroy() {
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerStreamDataset{}.Destroy(r.Dataset)
	FfiDestroyerString{}.Destroy(r.Block)
	FfiDestroyerString{}.Destroy(r.FilterFunction)
	FfiDestroyerOptionalFilterLanguage{}.Destroy(r.FilterLanguage)
	FfiDestroyerOptionalAddressBookConfig{}.Destroy(r.AddressBookConfig)
}

type FfiConverterTestFilterParams struct{}

var FfiConverterTestFilterParamsINSTANCE = FfiConverterTestFilterParams{}

func (c FfiConverterTestFilterParams) Lift(rb RustBufferI) TestFilterParams {
	return LiftFromRustBuffer[TestFilterParams](c, rb)
}

func (c FfiConverterTestFilterParams) Read(reader io.Reader) TestFilterParams {
	return TestFilterParams{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStreamDatasetINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalFilterLanguageINSTANCE.Read(reader),
		FfiConverterOptionalAddressBookConfigINSTANCE.Read(reader),
	}
}

func (c FfiConverterTestFilterParams) Lower(value TestFilterParams) C.RustBuffer {
	return LowerIntoRustBuffer[TestFilterParams](c, value)
}

func (c FfiConverterTestFilterParams) LowerExternal(value TestFilterParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TestFilterParams](c, value))
}

func (c FfiConverterTestFilterParams) Write(writer io.Writer, value TestFilterParams) {
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterStreamDatasetINSTANCE.Write(writer, value.Dataset)
	FfiConverterStringINSTANCE.Write(writer, value.Block)
	FfiConverterStringINSTANCE.Write(writer, value.FilterFunction)
	FfiConverterOptionalFilterLanguageINSTANCE.Write(writer, value.FilterLanguage)
	FfiConverterOptionalAddressBookConfigINSTANCE.Write(writer, value.AddressBookConfig)
}

type FfiDestroyerTestFilterParams struct{}

func (_ FfiDestroyerTestFilterParams) Destroy(value TestFilterParams) {
	value.Destroy()
}

// Result of a `test_filter` call.
type TestFilterResponse struct {
	// Filter output as a JSON string. Shape depends on the dataset and the user's filter function.
	Result string
	// Log lines emitted by the filter function during evaluation.
	Logs []string
}

func (r *TestFilterResponse) Destroy() {
	FfiDestroyerString{}.Destroy(r.Result)
	FfiDestroyerSequenceString{}.Destroy(r.Logs)
}

type FfiConverterTestFilterResponse struct{}

var FfiConverterTestFilterResponseINSTANCE = FfiConverterTestFilterResponse{}

func (c FfiConverterTestFilterResponse) Lift(rb RustBufferI) TestFilterResponse {
	return LiftFromRustBuffer[TestFilterResponse](c, rb)
}

func (c FfiConverterTestFilterResponse) Read(reader io.Reader) TestFilterResponse {
	return TestFilterResponse{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterTestFilterResponse) Lower(value TestFilterResponse) C.RustBuffer {
	return LowerIntoRustBuffer[TestFilterResponse](c, value)
}

func (c FfiConverterTestFilterResponse) LowerExternal(value TestFilterResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TestFilterResponse](c, value))
}

func (c FfiConverterTestFilterResponse) Write(writer io.Writer, value TestFilterResponse) {
	FfiConverterStringINSTANCE.Write(writer, value.Result)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Logs)
}

type FfiDestroyerTestFilterResponse struct{}

func (_ FfiDestroyerTestFilterResponse) Destroy(value TestFilterResponse) {
	value.Destroy()
}

// Parameters for `update_endpoint`.
type UpdateEndpointRequest struct {
	// New human-readable label.
	Label *string
}

func (r *UpdateEndpointRequest) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Label)
}

type FfiConverterUpdateEndpointRequest struct{}

var FfiConverterUpdateEndpointRequestINSTANCE = FfiConverterUpdateEndpointRequest{}

func (c FfiConverterUpdateEndpointRequest) Lift(rb RustBufferI) UpdateEndpointRequest {
	return LiftFromRustBuffer[UpdateEndpointRequest](c, rb)
}

func (c FfiConverterUpdateEndpointRequest) Read(reader io.Reader) UpdateEndpointRequest {
	return UpdateEndpointRequest{
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateEndpointRequest) Lower(value UpdateEndpointRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateEndpointRequest](c, value)
}

func (c FfiConverterUpdateEndpointRequest) LowerExternal(value UpdateEndpointRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateEndpointRequest](c, value))
}

func (c FfiConverterUpdateEndpointRequest) Write(writer io.Writer, value UpdateEndpointRequest) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Label)
}

type FfiDestroyerUpdateEndpointRequest struct{}

func (_ FfiDestroyerUpdateEndpointRequest) Destroy(value UpdateEndpointRequest) {
	value.Destroy()
}

// Parameters for `update_endpoint_status`.
type UpdateEndpointStatusRequest struct {
	// New status (`active` or `paused`).
	Status string
}

func (r *UpdateEndpointStatusRequest) Destroy() {
	FfiDestroyerString{}.Destroy(r.Status)
}

type FfiConverterUpdateEndpointStatusRequest struct{}

var FfiConverterUpdateEndpointStatusRequestINSTANCE = FfiConverterUpdateEndpointStatusRequest{}

func (c FfiConverterUpdateEndpointStatusRequest) Lift(rb RustBufferI) UpdateEndpointStatusRequest {
	return LiftFromRustBuffer[UpdateEndpointStatusRequest](c, rb)
}

func (c FfiConverterUpdateEndpointStatusRequest) Read(reader io.Reader) UpdateEndpointStatusRequest {
	return UpdateEndpointStatusRequest{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateEndpointStatusRequest) Lower(value UpdateEndpointStatusRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateEndpointStatusRequest](c, value)
}

func (c FfiConverterUpdateEndpointStatusRequest) LowerExternal(value UpdateEndpointStatusRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateEndpointStatusRequest](c, value))
}

func (c FfiConverterUpdateEndpointStatusRequest) Write(writer io.Writer, value UpdateEndpointStatusRequest) {
	FfiConverterStringINSTANCE.Write(writer, value.Status)
}

type FfiDestroyerUpdateEndpointStatusRequest struct{}

func (_ FfiDestroyerUpdateEndpointStatusRequest) Destroy(value UpdateEndpointStatusRequest) {
	value.Destroy()
}

// Response from `update_endpoint_status`.
type UpdateEndpointStatusResponse struct {
	// Confirmation string returned by the API.
	Data *string
	// Error message when the request did not succeed.
	Error *string
}

func (r *UpdateEndpointStatusResponse) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterUpdateEndpointStatusResponse struct{}

var FfiConverterUpdateEndpointStatusResponseINSTANCE = FfiConverterUpdateEndpointStatusResponse{}

func (c FfiConverterUpdateEndpointStatusResponse) Lift(rb RustBufferI) UpdateEndpointStatusResponse {
	return LiftFromRustBuffer[UpdateEndpointStatusResponse](c, rb)
}

func (c FfiConverterUpdateEndpointStatusResponse) Read(reader io.Reader) UpdateEndpointStatusResponse {
	return UpdateEndpointStatusResponse{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateEndpointStatusResponse) Lower(value UpdateEndpointStatusResponse) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateEndpointStatusResponse](c, value)
}

func (c FfiConverterUpdateEndpointStatusResponse) LowerExternal(value UpdateEndpointStatusResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateEndpointStatusResponse](c, value))
}

func (c FfiConverterUpdateEndpointStatusResponse) Write(writer io.Writer, value UpdateEndpointStatusResponse) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerUpdateEndpointStatusResponse struct{}

func (_ FfiDestroyerUpdateEndpointStatusResponse) Destroy(value UpdateEndpointStatusResponse) {
	value.Destroy()
}

// Parameters for `update_list`. Either or both fields may be supplied.
type UpdateListParams struct {
	// Items to add to the list.
	AddItems *[]string
	// Items to remove from the list.
	RemoveItems *[]string
}

func (r *UpdateListParams) Destroy() {
	FfiDestroyerOptionalSequenceString{}.Destroy(r.AddItems)
	FfiDestroyerOptionalSequenceString{}.Destroy(r.RemoveItems)
}

type FfiConverterUpdateListParams struct{}

var FfiConverterUpdateListParamsINSTANCE = FfiConverterUpdateListParams{}

func (c FfiConverterUpdateListParams) Lift(rb RustBufferI) UpdateListParams {
	return LiftFromRustBuffer[UpdateListParams](c, rb)
}

func (c FfiConverterUpdateListParams) Read(reader io.Reader) UpdateListParams {
	return UpdateListParams{
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateListParams) Lower(value UpdateListParams) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateListParams](c, value)
}

func (c FfiConverterUpdateListParams) LowerExternal(value UpdateListParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateListParams](c, value))
}

func (c FfiConverterUpdateListParams) Write(writer io.Writer, value UpdateListParams) {
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.AddItems)
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.RemoveItems)
}

type FfiDestroyerUpdateListParams struct{}

func (_ FfiDestroyerUpdateListParams) Destroy(value UpdateListParams) {
	value.Destroy()
}

// Parameters for `update_method_rate_limit`. Only provided fields are changed.
type UpdateMethodRateLimitRequest struct {
	// New set of RPC methods the limiter applies to.
	Methods *[]string
	// New status (`enabled` or `disabled`).
	Status *string
	// New rate value.
	Rate *int32
}

func (r *UpdateMethodRateLimitRequest) Destroy() {
	FfiDestroyerOptionalSequenceString{}.Destroy(r.Methods)
	FfiDestroyerOptionalString{}.Destroy(r.Status)
	FfiDestroyerOptionalInt32{}.Destroy(r.Rate)
}

type FfiConverterUpdateMethodRateLimitRequest struct{}

var FfiConverterUpdateMethodRateLimitRequestINSTANCE = FfiConverterUpdateMethodRateLimitRequest{}

func (c FfiConverterUpdateMethodRateLimitRequest) Lift(rb RustBufferI) UpdateMethodRateLimitRequest {
	return LiftFromRustBuffer[UpdateMethodRateLimitRequest](c, rb)
}

func (c FfiConverterUpdateMethodRateLimitRequest) Read(reader io.Reader) UpdateMethodRateLimitRequest {
	return UpdateMethodRateLimitRequest{
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateMethodRateLimitRequest) Lower(value UpdateMethodRateLimitRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateMethodRateLimitRequest](c, value)
}

func (c FfiConverterUpdateMethodRateLimitRequest) LowerExternal(value UpdateMethodRateLimitRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateMethodRateLimitRequest](c, value))
}

func (c FfiConverterUpdateMethodRateLimitRequest) Write(writer io.Writer, value UpdateMethodRateLimitRequest) {
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Methods)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.Rate)
}

type FfiDestroyerUpdateMethodRateLimitRequest struct{}

func (_ FfiDestroyerUpdateMethodRateLimitRequest) Destroy(value UpdateMethodRateLimitRequest) {
	value.Destroy()
}

// Response from `update_method_rate_limit`.
type UpdateMethodRateLimitResponse struct {
	// The updated rate limiter.
	Data *MethodRateLimiter
	// Error message when the request did not succeed.
	Error *string
}

func (r *UpdateMethodRateLimitResponse) Destroy() {
	FfiDestroyerOptionalMethodRateLimiter{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterUpdateMethodRateLimitResponse struct{}

var FfiConverterUpdateMethodRateLimitResponseINSTANCE = FfiConverterUpdateMethodRateLimitResponse{}

func (c FfiConverterUpdateMethodRateLimitResponse) Lift(rb RustBufferI) UpdateMethodRateLimitResponse {
	return LiftFromRustBuffer[UpdateMethodRateLimitResponse](c, rb)
}

func (c FfiConverterUpdateMethodRateLimitResponse) Read(reader io.Reader) UpdateMethodRateLimitResponse {
	return UpdateMethodRateLimitResponse{
		FfiConverterOptionalMethodRateLimiterINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateMethodRateLimitResponse) Lower(value UpdateMethodRateLimitResponse) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateMethodRateLimitResponse](c, value)
}

func (c FfiConverterUpdateMethodRateLimitResponse) LowerExternal(value UpdateMethodRateLimitResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateMethodRateLimitResponse](c, value))
}

func (c FfiConverterUpdateMethodRateLimitResponse) Write(writer io.Writer, value UpdateMethodRateLimitResponse) {
	FfiConverterOptionalMethodRateLimiterINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerUpdateMethodRateLimitResponse struct{}

func (_ FfiDestroyerUpdateMethodRateLimitResponse) Destroy(value UpdateMethodRateLimitResponse) {
	value.Destroy()
}

// Parameters for `update_rate_limits`.
type UpdateRateLimitsRequest struct {
	// Rate limit values to apply.
	RateLimits RateLimitSettings
}

func (r *UpdateRateLimitsRequest) Destroy() {
	FfiDestroyerRateLimitSettings{}.Destroy(r.RateLimits)
}

type FfiConverterUpdateRateLimitsRequest struct{}

var FfiConverterUpdateRateLimitsRequestINSTANCE = FfiConverterUpdateRateLimitsRequest{}

func (c FfiConverterUpdateRateLimitsRequest) Lift(rb RustBufferI) UpdateRateLimitsRequest {
	return LiftFromRustBuffer[UpdateRateLimitsRequest](c, rb)
}

func (c FfiConverterUpdateRateLimitsRequest) Read(reader io.Reader) UpdateRateLimitsRequest {
	return UpdateRateLimitsRequest{
		FfiConverterRateLimitSettingsINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateRateLimitsRequest) Lower(value UpdateRateLimitsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateRateLimitsRequest](c, value)
}

func (c FfiConverterUpdateRateLimitsRequest) LowerExternal(value UpdateRateLimitsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateRateLimitsRequest](c, value))
}

func (c FfiConverterUpdateRateLimitsRequest) Write(writer io.Writer, value UpdateRateLimitsRequest) {
	FfiConverterRateLimitSettingsINSTANCE.Write(writer, value.RateLimits)
}

type FfiDestroyerUpdateRateLimitsRequest struct{}

func (_ FfiDestroyerUpdateRateLimitsRequest) Destroy(value UpdateRateLimitsRequest) {
	value.Destroy()
}

// Parameters for `update_request_filter`.
type UpdateRequestFilterRequest struct {
	// New set of whitelisted RPC methods.
	Method *[]string
}

func (r *UpdateRequestFilterRequest) Destroy() {
	FfiDestroyerOptionalSequenceString{}.Destroy(r.Method)
}

type FfiConverterUpdateRequestFilterRequest struct{}

var FfiConverterUpdateRequestFilterRequestINSTANCE = FfiConverterUpdateRequestFilterRequest{}

func (c FfiConverterUpdateRequestFilterRequest) Lift(rb RustBufferI) UpdateRequestFilterRequest {
	return LiftFromRustBuffer[UpdateRequestFilterRequest](c, rb)
}

func (c FfiConverterUpdateRequestFilterRequest) Read(reader io.Reader) UpdateRequestFilterRequest {
	return UpdateRequestFilterRequest{
		FfiConverterOptionalSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateRequestFilterRequest) Lower(value UpdateRequestFilterRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateRequestFilterRequest](c, value)
}

func (c FfiConverterUpdateRequestFilterRequest) LowerExternal(value UpdateRequestFilterRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateRequestFilterRequest](c, value))
}

func (c FfiConverterUpdateRequestFilterRequest) Write(writer io.Writer, value UpdateRequestFilterRequest) {
	FfiConverterOptionalSequenceStringINSTANCE.Write(writer, value.Method)
}

type FfiDestroyerUpdateRequestFilterRequest struct{}

func (_ FfiDestroyerUpdateRequestFilterRequest) Destroy(value UpdateRequestFilterRequest) {
	value.Destroy()
}

// Parameters for `update_security_options`.
type UpdateSecurityOptionsRequest struct {
	// Security toggles to apply.
	Options SecurityOptionsUpdate
}

func (r *UpdateSecurityOptionsRequest) Destroy() {
	FfiDestroyerSecurityOptionsUpdate{}.Destroy(r.Options)
}

type FfiConverterUpdateSecurityOptionsRequest struct{}

var FfiConverterUpdateSecurityOptionsRequestINSTANCE = FfiConverterUpdateSecurityOptionsRequest{}

func (c FfiConverterUpdateSecurityOptionsRequest) Lift(rb RustBufferI) UpdateSecurityOptionsRequest {
	return LiftFromRustBuffer[UpdateSecurityOptionsRequest](c, rb)
}

func (c FfiConverterUpdateSecurityOptionsRequest) Read(reader io.Reader) UpdateSecurityOptionsRequest {
	return UpdateSecurityOptionsRequest{
		FfiConverterSecurityOptionsUpdateINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateSecurityOptionsRequest) Lower(value UpdateSecurityOptionsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateSecurityOptionsRequest](c, value)
}

func (c FfiConverterUpdateSecurityOptionsRequest) LowerExternal(value UpdateSecurityOptionsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateSecurityOptionsRequest](c, value))
}

func (c FfiConverterUpdateSecurityOptionsRequest) Write(writer io.Writer, value UpdateSecurityOptionsRequest) {
	FfiConverterSecurityOptionsUpdateINSTANCE.Write(writer, value.Options)
}

type FfiDestroyerUpdateSecurityOptionsRequest struct{}

func (_ FfiDestroyerUpdateSecurityOptionsRequest) Destroy(value UpdateSecurityOptionsRequest) {
	value.Destroy()
}

// Response from `update_security_options`.
type UpdateSecurityOptionsResponse struct {
	// Updated security options.
	Data []SecurityOption
	// Error message when the request did not succeed.
	Error *string
}

func (r *UpdateSecurityOptionsResponse) Destroy() {
	FfiDestroyerSequenceSecurityOption{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterUpdateSecurityOptionsResponse struct{}

var FfiConverterUpdateSecurityOptionsResponseINSTANCE = FfiConverterUpdateSecurityOptionsResponse{}

func (c FfiConverterUpdateSecurityOptionsResponse) Lift(rb RustBufferI) UpdateSecurityOptionsResponse {
	return LiftFromRustBuffer[UpdateSecurityOptionsResponse](c, rb)
}

func (c FfiConverterUpdateSecurityOptionsResponse) Read(reader io.Reader) UpdateSecurityOptionsResponse {
	return UpdateSecurityOptionsResponse{
		FfiConverterSequenceSecurityOptionINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateSecurityOptionsResponse) Lower(value UpdateSecurityOptionsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateSecurityOptionsResponse](c, value)
}

func (c FfiConverterUpdateSecurityOptionsResponse) LowerExternal(value UpdateSecurityOptionsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateSecurityOptionsResponse](c, value))
}

func (c FfiConverterUpdateSecurityOptionsResponse) Write(writer io.Writer, value UpdateSecurityOptionsResponse) {
	FfiConverterSequenceSecurityOptionINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerUpdateSecurityOptionsResponse struct{}

func (_ FfiDestroyerUpdateSecurityOptionsResponse) Destroy(value UpdateSecurityOptionsResponse) {
	value.Destroy()
}

// Parameters for `update_stream`. Only fields that are set are modified;
// omitted fields leave the current value unchanged.
type UpdateStreamParams struct {
	// New human-readable name.
	Name *string
	// New region.
	Region *StreamRegion
	// New blockchain network.
	Network *string
	// New dataset.
	Dataset *StreamDataset
	// New start block.
	StartRange *int64
	// New end block; `-1` for continuous operation.
	EndRange *int64
	// New primary destination configuration.
	DestinationAttributes *DestinationAttributes
	// New billing plan.
	Plan *string
	// New fetcher buffer threshold.
	ThresholdFetchBuffer *int64
	// New batch size.
	DatasetBatchSize *int64
	// New upper bound on elastic batch size.
	MaxBatchSize *int64
	// New maximum buffered block range.
	MaxBufferRangeSize *int64
	// New maximum number of buffer-processing workers.
	MaxBufferProcessingWorkers *int64
	// New distance from the chain tip.
	KeepDistanceFromTip *int64
	// New base64-encoded filter function.
	FilterFunction *string
	// New filter function language.
	FilterLanguage *FilterLanguage
	// New address book configuration.
	AddressBookConfig *AddressBookConfig
	// New stream-metadata location.
	IncludeStreamMetadata *StreamMetadataLocation
	// New notification email.
	NotificationEmail *string
	// New minimum charge cap.
	ChargeMinCap *int32
	// New reorg-handling flag (0 or 1).
	FixBlockReorgs *int32
	// Whether elastic batching is enabled.
	ElasticBatchEnabled *bool
	// New operational state.
	Status *StreamStatus
	// Free-text memo to attach to the stream.
	Memo *string
	// New set of extra destinations.
	ExtraDestinations *[]DestinationAttributes
}

func (r *UpdateStreamParams) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Name)
	FfiDestroyerOptionalStreamRegion{}.Destroy(r.Region)
	FfiDestroyerOptionalString{}.Destroy(r.Network)
	FfiDestroyerOptionalStreamDataset{}.Destroy(r.Dataset)
	FfiDestroyerOptionalInt64{}.Destroy(r.StartRange)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndRange)
	FfiDestroyerOptionalDestinationAttributes{}.Destroy(r.DestinationAttributes)
	FfiDestroyerOptionalString{}.Destroy(r.Plan)
	FfiDestroyerOptionalInt64{}.Destroy(r.ThresholdFetchBuffer)
	FfiDestroyerOptionalInt64{}.Destroy(r.DatasetBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBatchSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferRangeSize)
	FfiDestroyerOptionalInt64{}.Destroy(r.MaxBufferProcessingWorkers)
	FfiDestroyerOptionalInt64{}.Destroy(r.KeepDistanceFromTip)
	FfiDestroyerOptionalString{}.Destroy(r.FilterFunction)
	FfiDestroyerOptionalFilterLanguage{}.Destroy(r.FilterLanguage)
	FfiDestroyerOptionalAddressBookConfig{}.Destroy(r.AddressBookConfig)
	FfiDestroyerOptionalStreamMetadataLocation{}.Destroy(r.IncludeStreamMetadata)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalInt32{}.Destroy(r.ChargeMinCap)
	FfiDestroyerOptionalInt32{}.Destroy(r.FixBlockReorgs)
	FfiDestroyerOptionalBool{}.Destroy(r.ElasticBatchEnabled)
	FfiDestroyerOptionalStreamStatus{}.Destroy(r.Status)
	FfiDestroyerOptionalString{}.Destroy(r.Memo)
	FfiDestroyerOptionalSequenceDestinationAttributes{}.Destroy(r.ExtraDestinations)
}

type FfiConverterUpdateStreamParams struct{}

var FfiConverterUpdateStreamParamsINSTANCE = FfiConverterUpdateStreamParams{}

func (c FfiConverterUpdateStreamParams) Lift(rb RustBufferI) UpdateStreamParams {
	return LiftFromRustBuffer[UpdateStreamParams](c, rb)
}

func (c FfiConverterUpdateStreamParams) Read(reader io.Reader) UpdateStreamParams {
	return UpdateStreamParams{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStreamRegionINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStreamDatasetINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalDestinationAttributesINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalFilterLanguageINSTANCE.Read(reader),
		FfiConverterOptionalAddressBookConfigINSTANCE.Read(reader),
		FfiConverterOptionalStreamMetadataLocationINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalInt32INSTANCE.Read(reader),
		FfiConverterOptionalBoolINSTANCE.Read(reader),
		FfiConverterOptionalStreamStatusINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateStreamParams) Lower(value UpdateStreamParams) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateStreamParams](c, value)
}

func (c FfiConverterUpdateStreamParams) LowerExternal(value UpdateStreamParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateStreamParams](c, value))
}

func (c FfiConverterUpdateStreamParams) Write(writer io.Writer, value UpdateStreamParams) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStreamRegionINSTANCE.Write(writer, value.Region)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Network)
	FfiConverterOptionalStreamDatasetINSTANCE.Write(writer, value.Dataset)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartRange)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndRange)
	FfiConverterOptionalDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Plan)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.ThresholdFetchBuffer)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.DatasetBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBatchSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferRangeSize)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.MaxBufferProcessingWorkers)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.KeepDistanceFromTip)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.FilterFunction)
	FfiConverterOptionalFilterLanguageINSTANCE.Write(writer, value.FilterLanguage)
	FfiConverterOptionalAddressBookConfigINSTANCE.Write(writer, value.AddressBookConfig)
	FfiConverterOptionalStreamMetadataLocationINSTANCE.Write(writer, value.IncludeStreamMetadata)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.ChargeMinCap)
	FfiConverterOptionalInt32INSTANCE.Write(writer, value.FixBlockReorgs)
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.ElasticBatchEnabled)
	FfiConverterOptionalStreamStatusINSTANCE.Write(writer, value.Status)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Memo)
	FfiConverterOptionalSequenceDestinationAttributesINSTANCE.Write(writer, value.ExtraDestinations)
}

type FfiDestroyerUpdateStreamParams struct{}

func (_ FfiDestroyerUpdateStreamParams) Destroy(value UpdateStreamParams) {
	value.Destroy()
}

// Inner data for `update_team_endpoints` responses.
type UpdateTeamEndpointsData struct {
	// `true` when the association update succeeded.
	Success *bool
}

func (r *UpdateTeamEndpointsData) Destroy() {
	FfiDestroyerOptionalBool{}.Destroy(r.Success)
}

type FfiConverterUpdateTeamEndpointsData struct{}

var FfiConverterUpdateTeamEndpointsDataINSTANCE = FfiConverterUpdateTeamEndpointsData{}

func (c FfiConverterUpdateTeamEndpointsData) Lift(rb RustBufferI) UpdateTeamEndpointsData {
	return LiftFromRustBuffer[UpdateTeamEndpointsData](c, rb)
}

func (c FfiConverterUpdateTeamEndpointsData) Read(reader io.Reader) UpdateTeamEndpointsData {
	return UpdateTeamEndpointsData{
		FfiConverterOptionalBoolINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateTeamEndpointsData) Lower(value UpdateTeamEndpointsData) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateTeamEndpointsData](c, value)
}

func (c FfiConverterUpdateTeamEndpointsData) LowerExternal(value UpdateTeamEndpointsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateTeamEndpointsData](c, value))
}

func (c FfiConverterUpdateTeamEndpointsData) Write(writer io.Writer, value UpdateTeamEndpointsData) {
	FfiConverterOptionalBoolINSTANCE.Write(writer, value.Success)
}

type FfiDestroyerUpdateTeamEndpointsData struct{}

func (_ FfiDestroyerUpdateTeamEndpointsData) Destroy(value UpdateTeamEndpointsData) {
	value.Destroy()
}

// Parameters for `update_team_endpoints`.
type UpdateTeamEndpointsRequest struct {
	// Endpoint ids to associate with the team; pass an empty array to remove all.
	EndpointIds []string
}

func (r *UpdateTeamEndpointsRequest) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.EndpointIds)
}

type FfiConverterUpdateTeamEndpointsRequest struct{}

var FfiConverterUpdateTeamEndpointsRequestINSTANCE = FfiConverterUpdateTeamEndpointsRequest{}

func (c FfiConverterUpdateTeamEndpointsRequest) Lift(rb RustBufferI) UpdateTeamEndpointsRequest {
	return LiftFromRustBuffer[UpdateTeamEndpointsRequest](c, rb)
}

func (c FfiConverterUpdateTeamEndpointsRequest) Read(reader io.Reader) UpdateTeamEndpointsRequest {
	return UpdateTeamEndpointsRequest{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateTeamEndpointsRequest) Lower(value UpdateTeamEndpointsRequest) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateTeamEndpointsRequest](c, value)
}

func (c FfiConverterUpdateTeamEndpointsRequest) LowerExternal(value UpdateTeamEndpointsRequest) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateTeamEndpointsRequest](c, value))
}

func (c FfiConverterUpdateTeamEndpointsRequest) Write(writer io.Writer, value UpdateTeamEndpointsRequest) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.EndpointIds)
}

type FfiDestroyerUpdateTeamEndpointsRequest struct{}

func (_ FfiDestroyerUpdateTeamEndpointsRequest) Destroy(value UpdateTeamEndpointsRequest) {
	value.Destroy()
}

// Response from `update_team_endpoints`.
type UpdateTeamEndpointsResponse struct {
	// Update result.
	Data *UpdateTeamEndpointsData
	// Error message when the request did not succeed.
	Error *string
}

func (r *UpdateTeamEndpointsResponse) Destroy() {
	FfiDestroyerOptionalUpdateTeamEndpointsData{}.Destroy(r.Data)
	FfiDestroyerOptionalString{}.Destroy(r.Error)
}

type FfiConverterUpdateTeamEndpointsResponse struct{}

var FfiConverterUpdateTeamEndpointsResponseINSTANCE = FfiConverterUpdateTeamEndpointsResponse{}

func (c FfiConverterUpdateTeamEndpointsResponse) Lift(rb RustBufferI) UpdateTeamEndpointsResponse {
	return LiftFromRustBuffer[UpdateTeamEndpointsResponse](c, rb)
}

func (c FfiConverterUpdateTeamEndpointsResponse) Read(reader io.Reader) UpdateTeamEndpointsResponse {
	return UpdateTeamEndpointsResponse{
		FfiConverterOptionalUpdateTeamEndpointsDataINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateTeamEndpointsResponse) Lower(value UpdateTeamEndpointsResponse) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateTeamEndpointsResponse](c, value)
}

func (c FfiConverterUpdateTeamEndpointsResponse) LowerExternal(value UpdateTeamEndpointsResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateTeamEndpointsResponse](c, value))
}

func (c FfiConverterUpdateTeamEndpointsResponse) Write(writer io.Writer, value UpdateTeamEndpointsResponse) {
	FfiConverterOptionalUpdateTeamEndpointsDataINSTANCE.Write(writer, value.Data)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Error)
}

type FfiDestroyerUpdateTeamEndpointsResponse struct{}

func (_ FfiDestroyerUpdateTeamEndpointsResponse) Destroy(value UpdateTeamEndpointsResponse) {
	value.Destroy()
}

// Parameters for `update_webhook`. All fields are optional; only set fields
// are modified.
type UpdateWebhookParams struct {
	// New human-readable name.
	Name *string
	// New notification email.
	NotificationEmail *string
	// New destination configuration.
	DestinationAttributes *WebhookDestinationAttributes
}

func (r *UpdateWebhookParams) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalWebhookDestinationAttributes{}.Destroy(r.DestinationAttributes)
}

type FfiConverterUpdateWebhookParams struct{}

var FfiConverterUpdateWebhookParamsINSTANCE = FfiConverterUpdateWebhookParams{}

func (c FfiConverterUpdateWebhookParams) Lift(rb RustBufferI) UpdateWebhookParams {
	return LiftFromRustBuffer[UpdateWebhookParams](c, rb)
}

func (c FfiConverterUpdateWebhookParams) Read(reader io.Reader) UpdateWebhookParams {
	return UpdateWebhookParams{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalWebhookDestinationAttributesINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateWebhookParams) Lower(value UpdateWebhookParams) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateWebhookParams](c, value)
}

func (c FfiConverterUpdateWebhookParams) LowerExternal(value UpdateWebhookParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateWebhookParams](c, value))
}

func (c FfiConverterUpdateWebhookParams) Write(writer io.Writer, value UpdateWebhookParams) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalWebhookDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
}

type FfiDestroyerUpdateWebhookParams struct{}

func (_ FfiDestroyerUpdateWebhookParams) Destroy(value UpdateWebhookParams) {
	value.Destroy()
}

// Parameters for `update_webhook_template`.
type UpdateWebhookTemplateParams struct {
	// New human-readable name.
	Name *string
	// New notification email.
	NotificationEmail *string
	// New destination configuration.
	DestinationAttributes *WebhookDestinationAttributes
	// New template identifier and arguments.
	TemplateArgs TemplateArgs
}

func (r *UpdateWebhookTemplateParams) Destroy() {
	FfiDestroyerOptionalString{}.Destroy(r.Name)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalWebhookDestinationAttributes{}.Destroy(r.DestinationAttributes)
	FfiDestroyerTemplateArgs{}.Destroy(r.TemplateArgs)
}

type FfiConverterUpdateWebhookTemplateParams struct{}

var FfiConverterUpdateWebhookTemplateParamsINSTANCE = FfiConverterUpdateWebhookTemplateParams{}

func (c FfiConverterUpdateWebhookTemplateParams) Lift(rb RustBufferI) UpdateWebhookTemplateParams {
	return LiftFromRustBuffer[UpdateWebhookTemplateParams](c, rb)
}

func (c FfiConverterUpdateWebhookTemplateParams) Read(reader io.Reader) UpdateWebhookTemplateParams {
	return UpdateWebhookTemplateParams{
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalWebhookDestinationAttributesINSTANCE.Read(reader),
		FfiConverterTemplateArgsINSTANCE.Read(reader),
	}
}

func (c FfiConverterUpdateWebhookTemplateParams) Lower(value UpdateWebhookTemplateParams) C.RustBuffer {
	return LowerIntoRustBuffer[UpdateWebhookTemplateParams](c, value)
}

func (c FfiConverterUpdateWebhookTemplateParams) LowerExternal(value UpdateWebhookTemplateParams) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UpdateWebhookTemplateParams](c, value))
}

func (c FfiConverterUpdateWebhookTemplateParams) Write(writer io.Writer, value UpdateWebhookTemplateParams) {
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Name)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalWebhookDestinationAttributesINSTANCE.Write(writer, value.DestinationAttributes)
	FfiConverterTemplateArgsINSTANCE.Write(writer, value.TemplateArgs)
}

type FfiDestroyerUpdateWebhookTemplateParams struct{}

func (_ FfiDestroyerUpdateWebhookTemplateParams) Destroy(value UpdateWebhookTemplateParams) {
	value.Destroy()
}

// Inner data for `get_usage_by_chain`.
type UsageByChainData struct {
	// Per-chain rows.
	Chains []ChainUsage
	// Start of the queried window.
	StartTime *int64
	// End of the queried window.
	EndTime *int64
}

func (r *UsageByChainData) Destroy() {
	FfiDestroyerSequenceChainUsage{}.Destroy(r.Chains)
	FfiDestroyerOptionalInt64{}.Destroy(r.StartTime)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndTime)
}

type FfiConverterUsageByChainData struct{}

var FfiConverterUsageByChainDataINSTANCE = FfiConverterUsageByChainData{}

func (c FfiConverterUsageByChainData) Lift(rb RustBufferI) UsageByChainData {
	return LiftFromRustBuffer[UsageByChainData](c, rb)
}

func (c FfiConverterUsageByChainData) Read(reader io.Reader) UsageByChainData {
	return UsageByChainData{
		FfiConverterSequenceChainUsageINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUsageByChainData) Lower(value UsageByChainData) C.RustBuffer {
	return LowerIntoRustBuffer[UsageByChainData](c, value)
}

func (c FfiConverterUsageByChainData) LowerExternal(value UsageByChainData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UsageByChainData](c, value))
}

func (c FfiConverterUsageByChainData) Write(writer io.Writer, value UsageByChainData) {
	FfiConverterSequenceChainUsageINSTANCE.Write(writer, value.Chains)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerUsageByChainData struct{}

func (_ FfiDestroyerUsageByChainData) Destroy(value UsageByChainData) {
	value.Destroy()
}

// Inner data for `get_usage_by_endpoint`.
type UsageByEndpointData struct {
	// Per-endpoint rows.
	Endpoints []EndpointUsage
	// Start of the queried window.
	StartTime *int64
	// End of the queried window.
	EndTime *int64
}

func (r *UsageByEndpointData) Destroy() {
	FfiDestroyerSequenceEndpointUsage{}.Destroy(r.Endpoints)
	FfiDestroyerOptionalInt64{}.Destroy(r.StartTime)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndTime)
}

type FfiConverterUsageByEndpointData struct{}

var FfiConverterUsageByEndpointDataINSTANCE = FfiConverterUsageByEndpointData{}

func (c FfiConverterUsageByEndpointData) Lift(rb RustBufferI) UsageByEndpointData {
	return LiftFromRustBuffer[UsageByEndpointData](c, rb)
}

func (c FfiConverterUsageByEndpointData) Read(reader io.Reader) UsageByEndpointData {
	return UsageByEndpointData{
		FfiConverterSequenceEndpointUsageINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUsageByEndpointData) Lower(value UsageByEndpointData) C.RustBuffer {
	return LowerIntoRustBuffer[UsageByEndpointData](c, value)
}

func (c FfiConverterUsageByEndpointData) LowerExternal(value UsageByEndpointData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UsageByEndpointData](c, value))
}

func (c FfiConverterUsageByEndpointData) Write(writer io.Writer, value UsageByEndpointData) {
	FfiConverterSequenceEndpointUsageINSTANCE.Write(writer, value.Endpoints)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerUsageByEndpointData struct{}

func (_ FfiDestroyerUsageByEndpointData) Destroy(value UsageByEndpointData) {
	value.Destroy()
}

// Inner data for `get_usage_by_method`.
type UsageByMethodData struct {
	// Per-method rows.
	Methods []MethodUsage
	// Start of the queried window.
	StartTime *int64
	// End of the queried window.
	EndTime *int64
}

func (r *UsageByMethodData) Destroy() {
	FfiDestroyerSequenceMethodUsage{}.Destroy(r.Methods)
	FfiDestroyerOptionalInt64{}.Destroy(r.StartTime)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndTime)
}

type FfiConverterUsageByMethodData struct{}

var FfiConverterUsageByMethodDataINSTANCE = FfiConverterUsageByMethodData{}

func (c FfiConverterUsageByMethodData) Lift(rb RustBufferI) UsageByMethodData {
	return LiftFromRustBuffer[UsageByMethodData](c, rb)
}

func (c FfiConverterUsageByMethodData) Read(reader io.Reader) UsageByMethodData {
	return UsageByMethodData{
		FfiConverterSequenceMethodUsageINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUsageByMethodData) Lower(value UsageByMethodData) C.RustBuffer {
	return LowerIntoRustBuffer[UsageByMethodData](c, value)
}

func (c FfiConverterUsageByMethodData) LowerExternal(value UsageByMethodData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UsageByMethodData](c, value))
}

func (c FfiConverterUsageByMethodData) Write(writer io.Writer, value UsageByMethodData) {
	FfiConverterSequenceMethodUsageINSTANCE.Write(writer, value.Methods)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerUsageByMethodData struct{}

func (_ FfiDestroyerUsageByMethodData) Destroy(value UsageByMethodData) {
	value.Destroy()
}

// Inner data for `get_usage_by_tag`.
type UsageByTagData struct {
	// Per-tag rows.
	Tags []TagUsage
	// Start of the queried window.
	StartTime *int64
	// End of the queried window.
	EndTime *int64
}

func (r *UsageByTagData) Destroy() {
	FfiDestroyerSequenceTagUsage{}.Destroy(r.Tags)
	FfiDestroyerOptionalInt64{}.Destroy(r.StartTime)
	FfiDestroyerOptionalInt64{}.Destroy(r.EndTime)
}

type FfiConverterUsageByTagData struct{}

var FfiConverterUsageByTagDataINSTANCE = FfiConverterUsageByTagData{}

func (c FfiConverterUsageByTagData) Lift(rb RustBufferI) UsageByTagData {
	return LiftFromRustBuffer[UsageByTagData](c, rb)
}

func (c FfiConverterUsageByTagData) Read(reader io.Reader) UsageByTagData {
	return UsageByTagData{
		FfiConverterSequenceTagUsageINSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUsageByTagData) Lower(value UsageByTagData) C.RustBuffer {
	return LowerIntoRustBuffer[UsageByTagData](c, value)
}

func (c FfiConverterUsageByTagData) LowerExternal(value UsageByTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UsageByTagData](c, value))
}

func (c FfiConverterUsageByTagData) Write(writer io.Writer, value UsageByTagData) {
	FfiConverterSequenceTagUsageINSTANCE.Write(writer, value.Tags)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerUsageByTagData struct{}

func (_ FfiDestroyerUsageByTagData) Destroy(value UsageByTagData) {
	value.Destroy()
}

// Aggregate account usage for a time window.
type UsageData struct {
	// Credits consumed during the window.
	CreditsUsed int64
	// Credits still available, when the plan has a finite limit.
	CreditsRemaining *int64
	// Plan's credit limit, when applicable.
	Limit *int64
	// Credits consumed beyond the plan limit.
	Overages *int64
	// Start of the queried window.
	StartTime int64
	// End of the queried window.
	EndTime int64
}

func (r *UsageData) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.CreditsUsed)
	FfiDestroyerOptionalInt64{}.Destroy(r.CreditsRemaining)
	FfiDestroyerOptionalInt64{}.Destroy(r.Limit)
	FfiDestroyerOptionalInt64{}.Destroy(r.Overages)
	FfiDestroyerInt64{}.Destroy(r.StartTime)
	FfiDestroyerInt64{}.Destroy(r.EndTime)
}

type FfiConverterUsageData struct{}

var FfiConverterUsageDataINSTANCE = FfiConverterUsageData{}

func (c FfiConverterUsageData) Lift(rb RustBufferI) UsageData {
	return LiftFromRustBuffer[UsageData](c, rb)
}

func (c FfiConverterUsageData) Read(reader io.Reader) UsageData {
	return UsageData{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterOptionalInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterUsageData) Lower(value UsageData) C.RustBuffer {
	return LowerIntoRustBuffer[UsageData](c, value)
}

func (c FfiConverterUsageData) LowerExternal(value UsageData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[UsageData](c, value))
}

func (c FfiConverterUsageData) Write(writer io.Writer, value UsageData) {
	FfiConverterInt64INSTANCE.Write(writer, value.CreditsUsed)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.CreditsRemaining)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterOptionalInt64INSTANCE.Write(writer, value.Overages)
	FfiConverterInt64INSTANCE.Write(writer, value.StartTime)
	FfiConverterInt64INSTANCE.Write(writer, value.EndTime)
}

type FfiDestroyerUsageData struct{}

func (_ FfiDestroyerUsageData) Destroy(value UsageData) {
	value.Destroy()
}

// A webhook's full configuration and current state.
type Webhook struct {
	// Unique webhook identifier.
	Id string
	// Human-readable webhook name.
	Name string
	// Current operational state (e.g. `active`, `paused`).
	Status string
	// Blockchain network the webhook is watching.
	Network string
	// Timestamp when the webhook was created.
	CreatedAt string
	// Timestamp of the most recent modification.
	UpdatedAt *string
	// Template identifier used to create the webhook, if any.
	TemplateId *string
	// Email address notified of webhook terminations or failures.
	NotificationEmail *string
	// Destination-specific configuration as a JSON string.
	DestinationAttributes *string
}

func (r *Webhook) Destroy() {
	FfiDestroyerString{}.Destroy(r.Id)
	FfiDestroyerString{}.Destroy(r.Name)
	FfiDestroyerString{}.Destroy(r.Status)
	FfiDestroyerString{}.Destroy(r.Network)
	FfiDestroyerString{}.Destroy(r.CreatedAt)
	FfiDestroyerOptionalString{}.Destroy(r.UpdatedAt)
	FfiDestroyerOptionalString{}.Destroy(r.TemplateId)
	FfiDestroyerOptionalString{}.Destroy(r.NotificationEmail)
	FfiDestroyerOptionalString{}.Destroy(r.DestinationAttributes)
}

type FfiConverterWebhook struct{}

var FfiConverterWebhookINSTANCE = FfiConverterWebhook{}

func (c FfiConverterWebhook) Lift(rb RustBufferI) Webhook {
	return LiftFromRustBuffer[Webhook](c, rb)
}

func (c FfiConverterWebhook) Read(reader io.Reader) Webhook {
	return Webhook{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterWebhook) Lower(value Webhook) C.RustBuffer {
	return LowerIntoRustBuffer[Webhook](c, value)
}

func (c FfiConverterWebhook) LowerExternal(value Webhook) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Webhook](c, value))
}

func (c FfiConverterWebhook) Write(writer io.Writer, value Webhook) {
	FfiConverterStringINSTANCE.Write(writer, value.Id)
	FfiConverterStringINSTANCE.Write(writer, value.Name)
	FfiConverterStringINSTANCE.Write(writer, value.Status)
	FfiConverterStringINSTANCE.Write(writer, value.Network)
	FfiConverterStringINSTANCE.Write(writer, value.CreatedAt)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.UpdatedAt)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.TemplateId)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.NotificationEmail)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.DestinationAttributes)
}

type FfiDestroyerWebhook struct{}

func (_ FfiDestroyerWebhook) Destroy(value Webhook) {
	value.Destroy()
}

// Configuration for delivering stream batches to an HTTP webhook endpoint.
type WebhookAttributes struct {
	// Destination URL that receives batched stream payloads.
	Url string
	// Maximum number of retry attempts for a failed delivery. Must be in the range 1–10.
	MaxRetry int32
	// Seconds to wait between retry attempts.
	RetryIntervalSec int32
	// Timeout in seconds for each POST request.
	PostTimeoutSec int32
	// Optional token included with each request so the receiver can verify authenticity. When supplied, must be at least 32 bytes (256 bits).
	SecurityToken *string
	// Compression applied to the payload (e.g. `none`, `gzip`). When omitted the server defaults to no compression.
	Compression *string
}

func (r *WebhookAttributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.Url)
	FfiDestroyerInt32{}.Destroy(r.MaxRetry)
	FfiDestroyerInt32{}.Destroy(r.RetryIntervalSec)
	FfiDestroyerInt32{}.Destroy(r.PostTimeoutSec)
	FfiDestroyerOptionalString{}.Destroy(r.SecurityToken)
	FfiDestroyerOptionalString{}.Destroy(r.Compression)
}

type FfiConverterWebhookAttributes struct{}

var FfiConverterWebhookAttributesINSTANCE = FfiConverterWebhookAttributes{}

func (c FfiConverterWebhookAttributes) Lift(rb RustBufferI) WebhookAttributes {
	return LiftFromRustBuffer[WebhookAttributes](c, rb)
}

func (c FfiConverterWebhookAttributes) Read(reader io.Reader) WebhookAttributes {
	return WebhookAttributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterInt32INSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterWebhookAttributes) Lower(value WebhookAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookAttributes](c, value)
}

func (c FfiConverterWebhookAttributes) LowerExternal(value WebhookAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookAttributes](c, value))
}

func (c FfiConverterWebhookAttributes) Write(writer io.Writer, value WebhookAttributes) {
	FfiConverterStringINSTANCE.Write(writer, value.Url)
	FfiConverterInt32INSTANCE.Write(writer, value.MaxRetry)
	FfiConverterInt32INSTANCE.Write(writer, value.RetryIntervalSec)
	FfiConverterInt32INSTANCE.Write(writer, value.PostTimeoutSec)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.SecurityToken)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Compression)
}

type FfiDestroyerWebhookAttributes struct{}

func (_ FfiDestroyerWebhookAttributes) Destroy(value WebhookAttributes) {
	value.Destroy()
}

// Destination configuration for a webhook.
type WebhookDestinationAttributes struct {
	// Target URL that receives webhook payloads.
	Url string
	// Optional token sent with each payload so the receiver can verify authenticity; generated automatically when omitted.
	SecurityToken *string
	// Payload compression (`gzip` or `none`).
	Compression string
}

func (r *WebhookDestinationAttributes) Destroy() {
	FfiDestroyerString{}.Destroy(r.Url)
	FfiDestroyerOptionalString{}.Destroy(r.SecurityToken)
	FfiDestroyerString{}.Destroy(r.Compression)
}

type FfiConverterWebhookDestinationAttributes struct{}

var FfiConverterWebhookDestinationAttributesINSTANCE = FfiConverterWebhookDestinationAttributes{}

func (c FfiConverterWebhookDestinationAttributes) Lift(rb RustBufferI) WebhookDestinationAttributes {
	return LiftFromRustBuffer[WebhookDestinationAttributes](c, rb)
}

func (c FfiConverterWebhookDestinationAttributes) Read(reader io.Reader) WebhookDestinationAttributes {
	return WebhookDestinationAttributes{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterWebhookDestinationAttributes) Lower(value WebhookDestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookDestinationAttributes](c, value)
}

func (c FfiConverterWebhookDestinationAttributes) LowerExternal(value WebhookDestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookDestinationAttributes](c, value))
}

func (c FfiConverterWebhookDestinationAttributes) Write(writer io.Writer, value WebhookDestinationAttributes) {
	FfiConverterStringINSTANCE.Write(writer, value.Url)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.SecurityToken)
	FfiConverterStringINSTANCE.Write(writer, value.Compression)
}

type FfiDestroyerWebhookDestinationAttributes struct{}

func (_ FfiDestroyerWebhookDestinationAttributes) Destroy(value WebhookDestinationAttributes) {
	value.Destroy()
}

// Response from `get_enabled_count` for webhooks.
type WebhookEnabledCountResponse struct {
	// Total count of enabled webhooks on the account.
	Total int64
}

func (r *WebhookEnabledCountResponse) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Total)
}

type FfiConverterWebhookEnabledCountResponse struct{}

var FfiConverterWebhookEnabledCountResponseINSTANCE = FfiConverterWebhookEnabledCountResponse{}

func (c FfiConverterWebhookEnabledCountResponse) Lift(rb RustBufferI) WebhookEnabledCountResponse {
	return LiftFromRustBuffer[WebhookEnabledCountResponse](c, rb)
}

func (c FfiConverterWebhookEnabledCountResponse) Read(reader io.Reader) WebhookEnabledCountResponse {
	return WebhookEnabledCountResponse{
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterWebhookEnabledCountResponse) Lower(value WebhookEnabledCountResponse) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookEnabledCountResponse](c, value)
}

func (c FfiConverterWebhookEnabledCountResponse) LowerExternal(value WebhookEnabledCountResponse) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookEnabledCountResponse](c, value))
}

func (c FfiConverterWebhookEnabledCountResponse) Write(writer io.Writer, value WebhookEnabledCountResponse) {
	FfiConverterInt64INSTANCE.Write(writer, value.Total)
}

type FfiDestroyerWebhookEnabledCountResponse struct{}

func (_ FfiDestroyerWebhookEnabledCountResponse) Destroy(value WebhookEnabledCountResponse) {
	value.Destroy()
}

// Pagination metadata returned alongside a paginated webhooks list.
type WebhookPageInfo struct {
	// Page size used for this response.
	Limit int64
	// Starting index of this page within the full result set.
	Offset int64
	// Total number of webhooks matching the query across all pages.
	Total int64
}

func (r *WebhookPageInfo) Destroy() {
	FfiDestroyerInt64{}.Destroy(r.Limit)
	FfiDestroyerInt64{}.Destroy(r.Offset)
	FfiDestroyerInt64{}.Destroy(r.Total)
}

type FfiConverterWebhookPageInfo struct{}

var FfiConverterWebhookPageInfoINSTANCE = FfiConverterWebhookPageInfo{}

func (c FfiConverterWebhookPageInfo) Lift(rb RustBufferI) WebhookPageInfo {
	return LiftFromRustBuffer[WebhookPageInfo](c, rb)
}

func (c FfiConverterWebhookPageInfo) Read(reader io.Reader) WebhookPageInfo {
	return WebhookPageInfo{
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
		FfiConverterInt64INSTANCE.Read(reader),
	}
}

func (c FfiConverterWebhookPageInfo) Lower(value WebhookPageInfo) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookPageInfo](c, value)
}

func (c FfiConverterWebhookPageInfo) LowerExternal(value WebhookPageInfo) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookPageInfo](c, value))
}

func (c FfiConverterWebhookPageInfo) Write(writer io.Writer, value WebhookPageInfo) {
	FfiConverterInt64INSTANCE.Write(writer, value.Limit)
	FfiConverterInt64INSTANCE.Write(writer, value.Offset)
	FfiConverterInt64INSTANCE.Write(writer, value.Total)
}

type FfiDestroyerWebhookPageInfo struct{}

func (_ FfiDestroyerWebhookPageInfo) Destroy(value WebhookPageInfo) {
	value.Destroy()
}

// ByList form of `XrplWalletFilterTemplate`.
type XrplWalletFilterByListTemplate struct {
	// Name of the pre-created wallets list.
	WalletsListName string
}

func (r *XrplWalletFilterByListTemplate) Destroy() {
	FfiDestroyerString{}.Destroy(r.WalletsListName)
}

type FfiConverterXrplWalletFilterByListTemplate struct{}

var FfiConverterXrplWalletFilterByListTemplateINSTANCE = FfiConverterXrplWalletFilterByListTemplate{}

func (c FfiConverterXrplWalletFilterByListTemplate) Lift(rb RustBufferI) XrplWalletFilterByListTemplate {
	return LiftFromRustBuffer[XrplWalletFilterByListTemplate](c, rb)
}

func (c FfiConverterXrplWalletFilterByListTemplate) Read(reader io.Reader) XrplWalletFilterByListTemplate {
	return XrplWalletFilterByListTemplate{
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterXrplWalletFilterByListTemplate) Lower(value XrplWalletFilterByListTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[XrplWalletFilterByListTemplate](c, value)
}

func (c FfiConverterXrplWalletFilterByListTemplate) LowerExternal(value XrplWalletFilterByListTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[XrplWalletFilterByListTemplate](c, value))
}

func (c FfiConverterXrplWalletFilterByListTemplate) Write(writer io.Writer, value XrplWalletFilterByListTemplate) {
	FfiConverterStringINSTANCE.Write(writer, value.WalletsListName)
}

type FfiDestroyerXrplWalletFilterByListTemplate struct{}

func (_ FfiDestroyerXrplWalletFilterByListTemplate) Destroy(value XrplWalletFilterByListTemplate) {
	value.Destroy()
}

// Template arguments for an XRPL wallet filter.
type XrplWalletFilterTemplate struct {
	// XRPL wallet addresses to match against.
	Wallets []string
}

func (r *XrplWalletFilterTemplate) Destroy() {
	FfiDestroyerSequenceString{}.Destroy(r.Wallets)
}

type FfiConverterXrplWalletFilterTemplate struct{}

var FfiConverterXrplWalletFilterTemplateINSTANCE = FfiConverterXrplWalletFilterTemplate{}

func (c FfiConverterXrplWalletFilterTemplate) Lift(rb RustBufferI) XrplWalletFilterTemplate {
	return LiftFromRustBuffer[XrplWalletFilterTemplate](c, rb)
}

func (c FfiConverterXrplWalletFilterTemplate) Read(reader io.Reader) XrplWalletFilterTemplate {
	return XrplWalletFilterTemplate{
		FfiConverterSequenceStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterXrplWalletFilterTemplate) Lower(value XrplWalletFilterTemplate) C.RustBuffer {
	return LowerIntoRustBuffer[XrplWalletFilterTemplate](c, value)
}

func (c FfiConverterXrplWalletFilterTemplate) LowerExternal(value XrplWalletFilterTemplate) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[XrplWalletFilterTemplate](c, value))
}

func (c FfiConverterXrplWalletFilterTemplate) Write(writer io.Writer, value XrplWalletFilterTemplate) {
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Wallets)
}

type FfiDestroyerXrplWalletFilterTemplate struct{}

func (_ FfiDestroyerXrplWalletFilterTemplate) Destroy(value XrplWalletFilterTemplate) {
	value.Destroy()
}

// `BitcoinWalletFilter` template arguments in either inline or by-list form.
type BitcoinWalletFilterInput interface {
	Destroy()
}
type BitcoinWalletFilterInputInline struct {
	Field0 BitcoinWalletFilterTemplate
}

func (e BitcoinWalletFilterInputInline) Destroy() {
	FfiDestroyerBitcoinWalletFilterTemplate{}.Destroy(e.Field0)
}

type BitcoinWalletFilterInputByList struct {
	Field0 BitcoinWalletFilterByListTemplate
}

func (e BitcoinWalletFilterInputByList) Destroy() {
	FfiDestroyerBitcoinWalletFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterBitcoinWalletFilterInput struct{}

var FfiConverterBitcoinWalletFilterInputINSTANCE = FfiConverterBitcoinWalletFilterInput{}

func (c FfiConverterBitcoinWalletFilterInput) Lift(rb RustBufferI) BitcoinWalletFilterInput {
	return LiftFromRustBuffer[BitcoinWalletFilterInput](c, rb)
}

func (c FfiConverterBitcoinWalletFilterInput) Lower(value BitcoinWalletFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[BitcoinWalletFilterInput](c, value)
}

func (c FfiConverterBitcoinWalletFilterInput) LowerExternal(value BitcoinWalletFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BitcoinWalletFilterInput](c, value))
}
func (FfiConverterBitcoinWalletFilterInput) Read(reader io.Reader) BitcoinWalletFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return BitcoinWalletFilterInputInline{
			FfiConverterBitcoinWalletFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return BitcoinWalletFilterInputByList{
			FfiConverterBitcoinWalletFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterBitcoinWalletFilterInput.Read()", id))
	}
}

func (FfiConverterBitcoinWalletFilterInput) Write(writer io.Writer, value BitcoinWalletFilterInput) {
	switch variant_value := value.(type) {
	case BitcoinWalletFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterBitcoinWalletFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case BitcoinWalletFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterBitcoinWalletFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterBitcoinWalletFilterInput.Write", value))
	}
}

type FfiDestroyerBitcoinWalletFilterInput struct{}

func (_ FfiDestroyerBitcoinWalletFilterInput) Destroy(value BitcoinWalletFilterInput) {
	value.Destroy()
}

// Destination-specific configuration for a stream. Exactly one variant
// selects where and how batches are delivered.
type DestinationAttributes interface {
	Destroy()
}

// HTTP webhook endpoint that receives batches in real time.
type DestinationAttributesWebhook struct {
	Field0 WebhookAttributes
}

func (e DestinationAttributesWebhook) Destroy() {
	FfiDestroyerWebhookAttributes{}.Destroy(e.Field0)
}

// S3-compatible object storage for archival or batch processing.
type DestinationAttributesS3 struct {
	Field0 S3Attributes
}

func (e DestinationAttributesS3) Destroy() {
	FfiDestroyerS3Attributes{}.Destroy(e.Field0)
}

// Azure Blob Storage destination.
type DestinationAttributesAzure struct {
	Field0 AzureAttributes
}

func (e DestinationAttributesAzure) Destroy() {
	FfiDestroyerAzureAttributes{}.Destroy(e.Field0)
}

// PostgreSQL database destination.
type DestinationAttributesPostgres struct {
	Field0 PostgresAttributes
}

func (e DestinationAttributesPostgres) Destroy() {
	FfiDestroyerPostgresAttributes{}.Destroy(e.Field0)
}

// Kafka topic destination.
type DestinationAttributesKafka struct {
	Field0 KafkaAttributes
}

func (e DestinationAttributesKafka) Destroy() {
	FfiDestroyerKafkaAttributes{}.Destroy(e.Field0)
}

type FfiConverterDestinationAttributes struct{}

var FfiConverterDestinationAttributesINSTANCE = FfiConverterDestinationAttributes{}

func (c FfiConverterDestinationAttributes) Lift(rb RustBufferI) DestinationAttributes {
	return LiftFromRustBuffer[DestinationAttributes](c, rb)
}

func (c FfiConverterDestinationAttributes) Lower(value DestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[DestinationAttributes](c, value)
}

func (c FfiConverterDestinationAttributes) LowerExternal(value DestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DestinationAttributes](c, value))
}
func (FfiConverterDestinationAttributes) Read(reader io.Reader) DestinationAttributes {
	id := readInt32(reader)
	switch id {
	case 1:
		return DestinationAttributesWebhook{
			FfiConverterWebhookAttributesINSTANCE.Read(reader),
		}
	case 2:
		return DestinationAttributesS3{
			FfiConverterS3AttributesINSTANCE.Read(reader),
		}
	case 3:
		return DestinationAttributesAzure{
			FfiConverterAzureAttributesINSTANCE.Read(reader),
		}
	case 4:
		return DestinationAttributesPostgres{
			FfiConverterPostgresAttributesINSTANCE.Read(reader),
		}
	case 5:
		return DestinationAttributesKafka{
			FfiConverterKafkaAttributesINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterDestinationAttributes.Read()", id))
	}
}

func (FfiConverterDestinationAttributes) Write(writer io.Writer, value DestinationAttributes) {
	switch variant_value := value.(type) {
	case DestinationAttributesWebhook:
		writeInt32(writer, 1)
		FfiConverterWebhookAttributesINSTANCE.Write(writer, variant_value.Field0)
	case DestinationAttributesS3:
		writeInt32(writer, 2)
		FfiConverterS3AttributesINSTANCE.Write(writer, variant_value.Field0)
	case DestinationAttributesAzure:
		writeInt32(writer, 3)
		FfiConverterAzureAttributesINSTANCE.Write(writer, variant_value.Field0)
	case DestinationAttributesPostgres:
		writeInt32(writer, 4)
		FfiConverterPostgresAttributesINSTANCE.Write(writer, variant_value.Field0)
	case DestinationAttributesKafka:
		writeInt32(writer, 5)
		FfiConverterKafkaAttributesINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterDestinationAttributes.Write", value))
	}
}

type FfiDestroyerDestinationAttributes struct{}

func (_ FfiDestroyerDestinationAttributes) Destroy(value DestinationAttributes) {
	value.Destroy()
}

// `EvmAbiFilter` template arguments in either inline or by-list form.
type EvmAbiFilterInput interface {
	Destroy()
}
type EvmAbiFilterInputInline struct {
	Field0 EvmAbiFilterTemplate
}

func (e EvmAbiFilterInputInline) Destroy() {
	FfiDestroyerEvmAbiFilterTemplate{}.Destroy(e.Field0)
}

type EvmAbiFilterInputByList struct {
	Field0 EvmAbiFilterByListTemplate
}

func (e EvmAbiFilterInputByList) Destroy() {
	FfiDestroyerEvmAbiFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterEvmAbiFilterInput struct{}

var FfiConverterEvmAbiFilterInputINSTANCE = FfiConverterEvmAbiFilterInput{}

func (c FfiConverterEvmAbiFilterInput) Lift(rb RustBufferI) EvmAbiFilterInput {
	return LiftFromRustBuffer[EvmAbiFilterInput](c, rb)
}

func (c FfiConverterEvmAbiFilterInput) Lower(value EvmAbiFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[EvmAbiFilterInput](c, value)
}

func (c FfiConverterEvmAbiFilterInput) LowerExternal(value EvmAbiFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmAbiFilterInput](c, value))
}
func (FfiConverterEvmAbiFilterInput) Read(reader io.Reader) EvmAbiFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return EvmAbiFilterInputInline{
			FfiConverterEvmAbiFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return EvmAbiFilterInputByList{
			FfiConverterEvmAbiFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterEvmAbiFilterInput.Read()", id))
	}
}

func (FfiConverterEvmAbiFilterInput) Write(writer io.Writer, value EvmAbiFilterInput) {
	switch variant_value := value.(type) {
	case EvmAbiFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterEvmAbiFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case EvmAbiFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterEvmAbiFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterEvmAbiFilterInput.Write", value))
	}
}

type FfiDestroyerEvmAbiFilterInput struct{}

func (_ FfiDestroyerEvmAbiFilterInput) Destroy(value EvmAbiFilterInput) {
	value.Destroy()
}

// `EvmContractEvents` template arguments in either inline or by-list form.
type EvmContractEventsInput interface {
	Destroy()
}
type EvmContractEventsInputInline struct {
	Field0 EvmContractEventsTemplate
}

func (e EvmContractEventsInputInline) Destroy() {
	FfiDestroyerEvmContractEventsTemplate{}.Destroy(e.Field0)
}

type EvmContractEventsInputByList struct {
	Field0 EvmContractEventsByListTemplate
}

func (e EvmContractEventsInputByList) Destroy() {
	FfiDestroyerEvmContractEventsByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterEvmContractEventsInput struct{}

var FfiConverterEvmContractEventsInputINSTANCE = FfiConverterEvmContractEventsInput{}

func (c FfiConverterEvmContractEventsInput) Lift(rb RustBufferI) EvmContractEventsInput {
	return LiftFromRustBuffer[EvmContractEventsInput](c, rb)
}

func (c FfiConverterEvmContractEventsInput) Lower(value EvmContractEventsInput) C.RustBuffer {
	return LowerIntoRustBuffer[EvmContractEventsInput](c, value)
}

func (c FfiConverterEvmContractEventsInput) LowerExternal(value EvmContractEventsInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmContractEventsInput](c, value))
}
func (FfiConverterEvmContractEventsInput) Read(reader io.Reader) EvmContractEventsInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return EvmContractEventsInputInline{
			FfiConverterEvmContractEventsTemplateINSTANCE.Read(reader),
		}
	case 2:
		return EvmContractEventsInputByList{
			FfiConverterEvmContractEventsByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterEvmContractEventsInput.Read()", id))
	}
}

func (FfiConverterEvmContractEventsInput) Write(writer io.Writer, value EvmContractEventsInput) {
	switch variant_value := value.(type) {
	case EvmContractEventsInputInline:
		writeInt32(writer, 1)
		FfiConverterEvmContractEventsTemplateINSTANCE.Write(writer, variant_value.Field0)
	case EvmContractEventsInputByList:
		writeInt32(writer, 2)
		FfiConverterEvmContractEventsByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterEvmContractEventsInput.Write", value))
	}
}

type FfiDestroyerEvmContractEventsInput struct{}

func (_ FfiDestroyerEvmContractEventsInput) Destroy(value EvmContractEventsInput) {
	value.Destroy()
}

// `EvmWalletFilter` template arguments in either inline or by-list form.
type EvmWalletFilterInput interface {
	Destroy()
}
type EvmWalletFilterInputInline struct {
	Field0 EvmWalletFilterTemplate
}

func (e EvmWalletFilterInputInline) Destroy() {
	FfiDestroyerEvmWalletFilterTemplate{}.Destroy(e.Field0)
}

type EvmWalletFilterInputByList struct {
	Field0 EvmWalletFilterByListTemplate
}

func (e EvmWalletFilterInputByList) Destroy() {
	FfiDestroyerEvmWalletFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterEvmWalletFilterInput struct{}

var FfiConverterEvmWalletFilterInputINSTANCE = FfiConverterEvmWalletFilterInput{}

func (c FfiConverterEvmWalletFilterInput) Lift(rb RustBufferI) EvmWalletFilterInput {
	return LiftFromRustBuffer[EvmWalletFilterInput](c, rb)
}

func (c FfiConverterEvmWalletFilterInput) Lower(value EvmWalletFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[EvmWalletFilterInput](c, value)
}

func (c FfiConverterEvmWalletFilterInput) LowerExternal(value EvmWalletFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[EvmWalletFilterInput](c, value))
}
func (FfiConverterEvmWalletFilterInput) Read(reader io.Reader) EvmWalletFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return EvmWalletFilterInputInline{
			FfiConverterEvmWalletFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return EvmWalletFilterInputByList{
			FfiConverterEvmWalletFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterEvmWalletFilterInput.Read()", id))
	}
}

func (FfiConverterEvmWalletFilterInput) Write(writer io.Writer, value EvmWalletFilterInput) {
	switch variant_value := value.(type) {
	case EvmWalletFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterEvmWalletFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case EvmWalletFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterEvmWalletFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterEvmWalletFilterInput.Write", value))
	}
}

type FfiDestroyerEvmWalletFilterInput struct{}

func (_ FfiDestroyerEvmWalletFilterInput) Destroy(value EvmWalletFilterInput) {
	value.Destroy()
}

// Language a stream's filter function is written in.
type FilterLanguage uint

const (
	FilterLanguageJavascript FilterLanguage = 1
	FilterLanguageGo         FilterLanguage = 2
	FilterLanguageWasm       FilterLanguage = 3
)

type FfiConverterFilterLanguage struct{}

var FfiConverterFilterLanguageINSTANCE = FfiConverterFilterLanguage{}

func (c FfiConverterFilterLanguage) Lift(rb RustBufferI) FilterLanguage {
	return LiftFromRustBuffer[FilterLanguage](c, rb)
}

func (c FfiConverterFilterLanguage) Lower(value FilterLanguage) C.RustBuffer {
	return LowerIntoRustBuffer[FilterLanguage](c, value)
}

func (c FfiConverterFilterLanguage) LowerExternal(value FilterLanguage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[FilterLanguage](c, value))
}
func (FfiConverterFilterLanguage) Read(reader io.Reader) FilterLanguage {
	id := readInt32(reader)
	return FilterLanguage(id)
}

func (FfiConverterFilterLanguage) Write(writer io.Writer, value FilterLanguage) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerFilterLanguage struct{}

func (_ FfiDestroyerFilterLanguage) Destroy(value FilterLanguage) {
}

// `HyperliquidWalletEventsFilter` template arguments in either inline or by-list form.
type HyperliquidWalletEventsFilterInput interface {
	Destroy()
}
type HyperliquidWalletEventsFilterInputInline struct {
	Field0 HyperliquidWalletEventsFilterTemplate
}

func (e HyperliquidWalletEventsFilterInputInline) Destroy() {
	FfiDestroyerHyperliquidWalletEventsFilterTemplate{}.Destroy(e.Field0)
}

type HyperliquidWalletEventsFilterInputByList struct {
	Field0 HyperliquidWalletEventsFilterByListTemplate
}

func (e HyperliquidWalletEventsFilterInputByList) Destroy() {
	FfiDestroyerHyperliquidWalletEventsFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterHyperliquidWalletEventsFilterInput struct{}

var FfiConverterHyperliquidWalletEventsFilterInputINSTANCE = FfiConverterHyperliquidWalletEventsFilterInput{}

func (c FfiConverterHyperliquidWalletEventsFilterInput) Lift(rb RustBufferI) HyperliquidWalletEventsFilterInput {
	return LiftFromRustBuffer[HyperliquidWalletEventsFilterInput](c, rb)
}

func (c FfiConverterHyperliquidWalletEventsFilterInput) Lower(value HyperliquidWalletEventsFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[HyperliquidWalletEventsFilterInput](c, value)
}

func (c FfiConverterHyperliquidWalletEventsFilterInput) LowerExternal(value HyperliquidWalletEventsFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[HyperliquidWalletEventsFilterInput](c, value))
}
func (FfiConverterHyperliquidWalletEventsFilterInput) Read(reader io.Reader) HyperliquidWalletEventsFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return HyperliquidWalletEventsFilterInputInline{
			FfiConverterHyperliquidWalletEventsFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return HyperliquidWalletEventsFilterInputByList{
			FfiConverterHyperliquidWalletEventsFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterHyperliquidWalletEventsFilterInput.Read()", id))
	}
}

func (FfiConverterHyperliquidWalletEventsFilterInput) Write(writer io.Writer, value HyperliquidWalletEventsFilterInput) {
	switch variant_value := value.(type) {
	case HyperliquidWalletEventsFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterHyperliquidWalletEventsFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case HyperliquidWalletEventsFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterHyperliquidWalletEventsFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterHyperliquidWalletEventsFilterInput.Write", value))
	}
}

type FfiDestroyerHyperliquidWalletEventsFilterInput struct{}

func (_ FfiDestroyerHyperliquidWalletEventsFilterInput) Destroy(value HyperliquidWalletEventsFilterInput) {
	value.Destroy()
}

// Billing product type the stream is associated with.
type ProductType uint

const (
	ProductTypeStream  ProductType = 1
	ProductTypeWebhook ProductType = 2
)

type FfiConverterProductType struct{}

var FfiConverterProductTypeINSTANCE = FfiConverterProductType{}

func (c FfiConverterProductType) Lift(rb RustBufferI) ProductType {
	return LiftFromRustBuffer[ProductType](c, rb)
}

func (c FfiConverterProductType) Lower(value ProductType) C.RustBuffer {
	return LowerIntoRustBuffer[ProductType](c, value)
}

func (c FfiConverterProductType) LowerExternal(value ProductType) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ProductType](c, value))
}
func (FfiConverterProductType) Read(reader io.Reader) ProductType {
	id := readInt32(reader)
	return ProductType(id)
}

func (FfiConverterProductType) Write(writer io.Writer, value ProductType) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerProductType struct{}

func (_ FfiDestroyerProductType) Destroy(value ProductType) {
}

// Errors surfaced across the Go FFI boundary. Mirrors the typed hierarchy used
// by the other bindings (see CLAUDE.md §Error Handling): `Config` covers
// configuration/URL-parse failures; `Http`/`Timeout`/`Connection` cover
// transport-level failures; `Api` carries the HTTP status and raw body;
// `Decode` carries the raw body that failed to parse.
type QuicknodeError struct {
	err error
}

// Convenience method to turn *QuicknodeError into error
// Avoiding treating nil pointer as non nil error interface
func (err *QuicknodeError) AsError() error {
	if err == nil {
		return nil
	} else {
		return err
	}
}

func (err QuicknodeError) Error() string {
	return fmt.Sprintf("QuicknodeError: %s", err.err.Error())
}

func (err QuicknodeError) Unwrap() error {
	return err.err
}

// Err* are used for checking error type with `errors.Is`
var ErrQuicknodeErrorConfig = fmt.Errorf("QuicknodeErrorConfig")
var ErrQuicknodeErrorHttp = fmt.Errorf("QuicknodeErrorHttp")
var ErrQuicknodeErrorTimeout = fmt.Errorf("QuicknodeErrorTimeout")
var ErrQuicknodeErrorConnection = fmt.Errorf("QuicknodeErrorConnection")
var ErrQuicknodeErrorApi = fmt.Errorf("QuicknodeErrorApi")
var ErrQuicknodeErrorDecode = fmt.Errorf("QuicknodeErrorDecode")

// Variant structs
type QuicknodeErrorConfig struct {
	Message string
}

func NewQuicknodeErrorConfig(
	message string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorConfig{
		Message: message}}
}

func (e QuicknodeErrorConfig) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
}

func (err QuicknodeErrorConfig) Error() string {
	return fmt.Sprint("Config",
		": ",

		"Message=",
		err.Message,
	)
}

func (self QuicknodeErrorConfig) Is(target error) bool {
	return target == ErrQuicknodeErrorConfig
}

type QuicknodeErrorHttp struct {
	Message string
}

func NewQuicknodeErrorHttp(
	message string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorHttp{
		Message: message}}
}

func (e QuicknodeErrorHttp) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
}

func (err QuicknodeErrorHttp) Error() string {
	return fmt.Sprint("Http",
		": ",

		"Message=",
		err.Message,
	)
}

func (self QuicknodeErrorHttp) Is(target error) bool {
	return target == ErrQuicknodeErrorHttp
}

type QuicknodeErrorTimeout struct {
	Message string
}

func NewQuicknodeErrorTimeout(
	message string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorTimeout{
		Message: message}}
}

func (e QuicknodeErrorTimeout) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
}

func (err QuicknodeErrorTimeout) Error() string {
	return fmt.Sprint("Timeout",
		": ",

		"Message=",
		err.Message,
	)
}

func (self QuicknodeErrorTimeout) Is(target error) bool {
	return target == ErrQuicknodeErrorTimeout
}

type QuicknodeErrorConnection struct {
	Message string
}

func NewQuicknodeErrorConnection(
	message string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorConnection{
		Message: message}}
}

func (e QuicknodeErrorConnection) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
}

func (err QuicknodeErrorConnection) Error() string {
	return fmt.Sprint("Connection",
		": ",

		"Message=",
		err.Message,
	)
}

func (self QuicknodeErrorConnection) Is(target error) bool {
	return target == ErrQuicknodeErrorConnection
}

type QuicknodeErrorApi struct {
	Message string
	Status  uint16
	Body    string
}

func NewQuicknodeErrorApi(
	message string,
	status uint16,
	body string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorApi{
		Message: message,
		Status:  status,
		Body:    body}}
}

func (e QuicknodeErrorApi) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
	FfiDestroyerUint16{}.Destroy(e.Status)
	FfiDestroyerString{}.Destroy(e.Body)
}

func (err QuicknodeErrorApi) Error() string {
	return fmt.Sprint("Api",
		": ",

		"Message=",
		err.Message,
		", ",
		"Status=",
		err.Status,
		", ",
		"Body=",
		err.Body,
	)
}

func (self QuicknodeErrorApi) Is(target error) bool {
	return target == ErrQuicknodeErrorApi
}

type QuicknodeErrorDecode struct {
	Message string
	Body    string
}

func NewQuicknodeErrorDecode(
	message string,
	body string,
) *QuicknodeError {
	return &QuicknodeError{err: &QuicknodeErrorDecode{
		Message: message,
		Body:    body}}
}

func (e QuicknodeErrorDecode) destroy() {
	FfiDestroyerString{}.Destroy(e.Message)
	FfiDestroyerString{}.Destroy(e.Body)
}

func (err QuicknodeErrorDecode) Error() string {
	return fmt.Sprint("Decode",
		": ",

		"Message=",
		err.Message,
		", ",
		"Body=",
		err.Body,
	)
}

func (self QuicknodeErrorDecode) Is(target error) bool {
	return target == ErrQuicknodeErrorDecode
}

type FfiConverterQuicknodeError struct{}

var FfiConverterQuicknodeErrorINSTANCE = FfiConverterQuicknodeError{}

func (c FfiConverterQuicknodeError) Lift(eb RustBufferI) *QuicknodeError {
	return LiftFromRustBuffer[*QuicknodeError](c, eb)
}

func (c FfiConverterQuicknodeError) Lower(value *QuicknodeError) C.RustBuffer {
	return LowerIntoRustBuffer[*QuicknodeError](c, value)
}

func (c FfiConverterQuicknodeError) LowerExternal(value *QuicknodeError) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*QuicknodeError](c, value))
}

func (c FfiConverterQuicknodeError) Read(reader io.Reader) *QuicknodeError {
	errorID := readUint32(reader)

	switch errorID {
	case 1:
		return &QuicknodeError{&QuicknodeErrorConfig{
			Message: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 2:
		return &QuicknodeError{&QuicknodeErrorHttp{
			Message: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 3:
		return &QuicknodeError{&QuicknodeErrorTimeout{
			Message: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 4:
		return &QuicknodeError{&QuicknodeErrorConnection{
			Message: FfiConverterStringINSTANCE.Read(reader),
		}}
	case 5:
		return &QuicknodeError{&QuicknodeErrorApi{
			Message: FfiConverterStringINSTANCE.Read(reader),
			Status:  FfiConverterUint16INSTANCE.Read(reader),
			Body:    FfiConverterStringINSTANCE.Read(reader),
		}}
	case 6:
		return &QuicknodeError{&QuicknodeErrorDecode{
			Message: FfiConverterStringINSTANCE.Read(reader),
			Body:    FfiConverterStringINSTANCE.Read(reader),
		}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterQuicknodeError.Read()", errorID))
	}
}

func (c FfiConverterQuicknodeError) Write(writer io.Writer, value *QuicknodeError) {
	switch variantValue := value.err.(type) {
	case *QuicknodeErrorConfig:
		writeInt32(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
	case *QuicknodeErrorHttp:
		writeInt32(writer, 2)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
	case *QuicknodeErrorTimeout:
		writeInt32(writer, 3)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
	case *QuicknodeErrorConnection:
		writeInt32(writer, 4)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
	case *QuicknodeErrorApi:
		writeInt32(writer, 5)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
		FfiConverterUint16INSTANCE.Write(writer, variantValue.Status)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Body)
	case *QuicknodeErrorDecode:
		writeInt32(writer, 6)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Message)
		FfiConverterStringINSTANCE.Write(writer, variantValue.Body)
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiConverterQuicknodeError.Write", value))
	}
}

type FfiDestroyerQuicknodeError struct{}

func (_ FfiDestroyerQuicknodeError) Destroy(value *QuicknodeError) {
	switch variantValue := value.err.(type) {
	case QuicknodeErrorConfig:
		variantValue.destroy()
	case QuicknodeErrorHttp:
		variantValue.destroy()
	case QuicknodeErrorTimeout:
		variantValue.destroy()
	case QuicknodeErrorConnection:
		variantValue.destroy()
	case QuicknodeErrorApi:
		variantValue.destroy()
	case QuicknodeErrorDecode:
		variantValue.destroy()
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiDestroyerQuicknodeError.Destroy", value))
	}
}

// `SolanaWalletFilter` template arguments in either inline or by-list form.
type SolanaWalletFilterInput interface {
	Destroy()
}
type SolanaWalletFilterInputInline struct {
	Field0 SolanaWalletFilterTemplate
}

func (e SolanaWalletFilterInputInline) Destroy() {
	FfiDestroyerSolanaWalletFilterTemplate{}.Destroy(e.Field0)
}

type SolanaWalletFilterInputByList struct {
	Field0 SolanaWalletFilterByListTemplate
}

func (e SolanaWalletFilterInputByList) Destroy() {
	FfiDestroyerSolanaWalletFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterSolanaWalletFilterInput struct{}

var FfiConverterSolanaWalletFilterInputINSTANCE = FfiConverterSolanaWalletFilterInput{}

func (c FfiConverterSolanaWalletFilterInput) Lift(rb RustBufferI) SolanaWalletFilterInput {
	return LiftFromRustBuffer[SolanaWalletFilterInput](c, rb)
}

func (c FfiConverterSolanaWalletFilterInput) Lower(value SolanaWalletFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[SolanaWalletFilterInput](c, value)
}

func (c FfiConverterSolanaWalletFilterInput) LowerExternal(value SolanaWalletFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SolanaWalletFilterInput](c, value))
}
func (FfiConverterSolanaWalletFilterInput) Read(reader io.Reader) SolanaWalletFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return SolanaWalletFilterInputInline{
			FfiConverterSolanaWalletFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return SolanaWalletFilterInputByList{
			FfiConverterSolanaWalletFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterSolanaWalletFilterInput.Read()", id))
	}
}

func (FfiConverterSolanaWalletFilterInput) Write(writer io.Writer, value SolanaWalletFilterInput) {
	switch variant_value := value.(type) {
	case SolanaWalletFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterSolanaWalletFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case SolanaWalletFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterSolanaWalletFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterSolanaWalletFilterInput.Write", value))
	}
}

type FfiDestroyerSolanaWalletFilterInput struct{}

func (_ FfiDestroyerSolanaWalletFilterInput) Destroy(value SolanaWalletFilterInput) {
	value.Destroy()
}

// `StellarWalletTransactionsSourceAccountFilter` template arguments in
// either inline or by-list form.
type StellarWalletTransactionsFilterInput interface {
	Destroy()
}
type StellarWalletTransactionsFilterInputInline struct {
	Field0 StellarWalletTransactionsFilterTemplate
}

func (e StellarWalletTransactionsFilterInputInline) Destroy() {
	FfiDestroyerStellarWalletTransactionsFilterTemplate{}.Destroy(e.Field0)
}

type StellarWalletTransactionsFilterInputByList struct {
	Field0 StellarWalletTransactionsFilterByListTemplate
}

func (e StellarWalletTransactionsFilterInputByList) Destroy() {
	FfiDestroyerStellarWalletTransactionsFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterStellarWalletTransactionsFilterInput struct{}

var FfiConverterStellarWalletTransactionsFilterInputINSTANCE = FfiConverterStellarWalletTransactionsFilterInput{}

func (c FfiConverterStellarWalletTransactionsFilterInput) Lift(rb RustBufferI) StellarWalletTransactionsFilterInput {
	return LiftFromRustBuffer[StellarWalletTransactionsFilterInput](c, rb)
}

func (c FfiConverterStellarWalletTransactionsFilterInput) Lower(value StellarWalletTransactionsFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[StellarWalletTransactionsFilterInput](c, value)
}

func (c FfiConverterStellarWalletTransactionsFilterInput) LowerExternal(value StellarWalletTransactionsFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StellarWalletTransactionsFilterInput](c, value))
}
func (FfiConverterStellarWalletTransactionsFilterInput) Read(reader io.Reader) StellarWalletTransactionsFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return StellarWalletTransactionsFilterInputInline{
			FfiConverterStellarWalletTransactionsFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return StellarWalletTransactionsFilterInputByList{
			FfiConverterStellarWalletTransactionsFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterStellarWalletTransactionsFilterInput.Read()", id))
	}
}

func (FfiConverterStellarWalletTransactionsFilterInput) Write(writer io.Writer, value StellarWalletTransactionsFilterInput) {
	switch variant_value := value.(type) {
	case StellarWalletTransactionsFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterStellarWalletTransactionsFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case StellarWalletTransactionsFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterStellarWalletTransactionsFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterStellarWalletTransactionsFilterInput.Write", value))
	}
}

type FfiDestroyerStellarWalletTransactionsFilterInput struct{}

func (_ FfiDestroyerStellarWalletTransactionsFilterInput) Destroy(value StellarWalletTransactionsFilterInput) {
	value.Destroy()
}

// Type of on-chain data a stream delivers (blocks, transactions, logs, etc.).
type StreamDataset uint

const (
	StreamDatasetBlock                       StreamDataset = 1
	StreamDatasetBlockWithReceipts           StreamDataset = 2
	StreamDatasetTransactions                StreamDataset = 3
	StreamDatasetLogs                        StreamDataset = 4
	StreamDatasetReceipts                    StreamDataset = 5
	StreamDatasetTraceBlocks                 StreamDataset = 6
	StreamDatasetDebugTraces                 StreamDataset = 7
	StreamDatasetBlockWithReceiptsDebugTrace StreamDataset = 8
	StreamDatasetBlockWithReceiptsTraceBlock StreamDataset = 9
	StreamDatasetBlobSidecars                StreamDataset = 10
	StreamDatasetProgramsWithLogs            StreamDataset = 11
	StreamDatasetLedger                      StreamDataset = 12
	StreamDatasetEvents                      StreamDataset = 13
	StreamDatasetOrders                      StreamDataset = 14
	StreamDatasetTrades                      StreamDataset = 15
	StreamDatasetBookUpdates                 StreamDataset = 16
	StreamDatasetTwap                        StreamDataset = 17
	StreamDatasetWriterActions               StreamDataset = 18
)

type FfiConverterStreamDataset struct{}

var FfiConverterStreamDatasetINSTANCE = FfiConverterStreamDataset{}

func (c FfiConverterStreamDataset) Lift(rb RustBufferI) StreamDataset {
	return LiftFromRustBuffer[StreamDataset](c, rb)
}

func (c FfiConverterStreamDataset) Lower(value StreamDataset) C.RustBuffer {
	return LowerIntoRustBuffer[StreamDataset](c, value)
}

func (c FfiConverterStreamDataset) LowerExternal(value StreamDataset) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StreamDataset](c, value))
}
func (FfiConverterStreamDataset) Read(reader io.Reader) StreamDataset {
	id := readInt32(reader)
	return StreamDataset(id)
}

func (FfiConverterStreamDataset) Write(writer io.Writer, value StreamDataset) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerStreamDataset struct{}

func (_ FfiDestroyerStreamDataset) Destroy(value StreamDataset) {
}

// Destination kind a stream delivers to (webhook, S3, Postgres, etc.).
type StreamDestination uint

const (
	StreamDestinationWebhook  StreamDestination = 1
	StreamDestinationS3       StreamDestination = 2
	StreamDestinationAzure    StreamDestination = 3
	StreamDestinationPostgres StreamDestination = 4
	StreamDestinationKafka    StreamDestination = 5
)

type FfiConverterStreamDestination struct{}

var FfiConverterStreamDestinationINSTANCE = FfiConverterStreamDestination{}

func (c FfiConverterStreamDestination) Lift(rb RustBufferI) StreamDestination {
	return LiftFromRustBuffer[StreamDestination](c, rb)
}

func (c FfiConverterStreamDestination) Lower(value StreamDestination) C.RustBuffer {
	return LowerIntoRustBuffer[StreamDestination](c, value)
}

func (c FfiConverterStreamDestination) LowerExternal(value StreamDestination) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StreamDestination](c, value))
}
func (FfiConverterStreamDestination) Read(reader io.Reader) StreamDestination {
	id := readInt32(reader)
	return StreamDestination(id)
}

func (FfiConverterStreamDestination) Write(writer io.Writer, value StreamDestination) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerStreamDestination struct{}

func (_ FfiDestroyerStreamDestination) Destroy(value StreamDestination) {
}

// Where stream metadata is included in delivered payloads.
type StreamMetadataLocation uint

const (
	StreamMetadataLocationBody   StreamMetadataLocation = 1
	StreamMetadataLocationHeader StreamMetadataLocation = 2
	StreamMetadataLocationNone   StreamMetadataLocation = 3
)

type FfiConverterStreamMetadataLocation struct{}

var FfiConverterStreamMetadataLocationINSTANCE = FfiConverterStreamMetadataLocation{}

func (c FfiConverterStreamMetadataLocation) Lift(rb RustBufferI) StreamMetadataLocation {
	return LiftFromRustBuffer[StreamMetadataLocation](c, rb)
}

func (c FfiConverterStreamMetadataLocation) Lower(value StreamMetadataLocation) C.RustBuffer {
	return LowerIntoRustBuffer[StreamMetadataLocation](c, value)
}

func (c FfiConverterStreamMetadataLocation) LowerExternal(value StreamMetadataLocation) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StreamMetadataLocation](c, value))
}
func (FfiConverterStreamMetadataLocation) Read(reader io.Reader) StreamMetadataLocation {
	id := readInt32(reader)
	return StreamMetadataLocation(id)
}

func (FfiConverterStreamMetadataLocation) Write(writer io.Writer, value StreamMetadataLocation) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerStreamMetadataLocation struct{}

func (_ FfiDestroyerStreamMetadataLocation) Destroy(value StreamMetadataLocation) {
}

// Geographic region where a stream runs.
type StreamRegion uint

const (
	StreamRegionUsaEast       StreamRegion = 1
	StreamRegionEuropeCentral StreamRegion = 2
	StreamRegionAsiaEast      StreamRegion = 3
)

type FfiConverterStreamRegion struct{}

var FfiConverterStreamRegionINSTANCE = FfiConverterStreamRegion{}

func (c FfiConverterStreamRegion) Lift(rb RustBufferI) StreamRegion {
	return LiftFromRustBuffer[StreamRegion](c, rb)
}

func (c FfiConverterStreamRegion) Lower(value StreamRegion) C.RustBuffer {
	return LowerIntoRustBuffer[StreamRegion](c, value)
}

func (c FfiConverterStreamRegion) LowerExternal(value StreamRegion) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StreamRegion](c, value))
}
func (FfiConverterStreamRegion) Read(reader io.Reader) StreamRegion {
	id := readInt32(reader)
	return StreamRegion(id)
}

func (FfiConverterStreamRegion) Write(writer io.Writer, value StreamRegion) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerStreamRegion struct{}

func (_ FfiDestroyerStreamRegion) Destroy(value StreamRegion) {
}

// Operational state of a stream.
type StreamStatus uint

const (
	StreamStatusActive     StreamStatus = 1
	StreamStatusPaused     StreamStatus = 2
	StreamStatusTerminated StreamStatus = 3
	StreamStatusCompleted  StreamStatus = 4
	StreamStatusBlocked    StreamStatus = 5
)

type FfiConverterStreamStatus struct{}

var FfiConverterStreamStatusINSTANCE = FfiConverterStreamStatus{}

func (c FfiConverterStreamStatus) Lift(rb RustBufferI) StreamStatus {
	return LiftFromRustBuffer[StreamStatus](c, rb)
}

func (c FfiConverterStreamStatus) Lower(value StreamStatus) C.RustBuffer {
	return LowerIntoRustBuffer[StreamStatus](c, value)
}

func (c FfiConverterStreamStatus) LowerExternal(value StreamStatus) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[StreamStatus](c, value))
}
func (FfiConverterStreamStatus) Read(reader io.Reader) StreamStatus {
	id := readInt32(reader)
	return StreamStatus(id)
}

func (FfiConverterStreamStatus) Write(writer io.Writer, value StreamStatus) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerStreamStatus struct{}

func (_ FfiDestroyerStreamStatus) Destroy(value StreamStatus) {
}

// Template identifier paired with its arguments. Exactly one variant selects
// which filter is applied; each variant's inner enum picks between inline
// values and a list reference. Consumed by `create_webhook_from_template`
// and `update_webhook_template`.
type TemplateArgs interface {
	Destroy()
}

// EVM wallet filter.
type TemplateArgsEvmWalletFilter struct {
	Field0 EvmWalletFilterInput
}

func (e TemplateArgsEvmWalletFilter) Destroy() {
	FfiDestroyerEvmWalletFilterInput{}.Destroy(e.Field0)
}

// EVM contract events filter.
type TemplateArgsEvmContractEvents struct {
	Field0 EvmContractEventsInput
}

func (e TemplateArgsEvmContractEvents) Destroy() {
	FfiDestroyerEvmContractEventsInput{}.Destroy(e.Field0)
}

// EVM ABI filter.
type TemplateArgsEvmAbiFilter struct {
	Field0 EvmAbiFilterInput
}

func (e TemplateArgsEvmAbiFilter) Destroy() {
	FfiDestroyerEvmAbiFilterInput{}.Destroy(e.Field0)
}

// Solana wallet filter.
type TemplateArgsSolanaWalletFilter struct {
	Field0 SolanaWalletFilterInput
}

func (e TemplateArgsSolanaWalletFilter) Destroy() {
	FfiDestroyerSolanaWalletFilterInput{}.Destroy(e.Field0)
}

// Bitcoin wallet filter.
type TemplateArgsBitcoinWalletFilter struct {
	Field0 BitcoinWalletFilterInput
}

func (e TemplateArgsBitcoinWalletFilter) Destroy() {
	FfiDestroyerBitcoinWalletFilterInput{}.Destroy(e.Field0)
}

// XRPL wallet filter.
type TemplateArgsXrplWalletFilter struct {
	Field0 XrplWalletFilterInput
}

func (e TemplateArgsXrplWalletFilter) Destroy() {
	FfiDestroyerXrplWalletFilterInput{}.Destroy(e.Field0)
}

// Hyperliquid wallet-events filter.
type TemplateArgsHyperliquidWalletEventsFilter struct {
	Field0 HyperliquidWalletEventsFilterInput
}

func (e TemplateArgsHyperliquidWalletEventsFilter) Destroy() {
	FfiDestroyerHyperliquidWalletEventsFilterInput{}.Destroy(e.Field0)
}

// Stellar wallet-transactions filter (source-account match).
type TemplateArgsStellarWalletTransactionsSourceAccountFilter struct {
	Field0 StellarWalletTransactionsFilterInput
}

func (e TemplateArgsStellarWalletTransactionsSourceAccountFilter) Destroy() {
	FfiDestroyerStellarWalletTransactionsFilterInput{}.Destroy(e.Field0)
}

type FfiConverterTemplateArgs struct{}

var FfiConverterTemplateArgsINSTANCE = FfiConverterTemplateArgs{}

func (c FfiConverterTemplateArgs) Lift(rb RustBufferI) TemplateArgs {
	return LiftFromRustBuffer[TemplateArgs](c, rb)
}

func (c FfiConverterTemplateArgs) Lower(value TemplateArgs) C.RustBuffer {
	return LowerIntoRustBuffer[TemplateArgs](c, value)
}

func (c FfiConverterTemplateArgs) LowerExternal(value TemplateArgs) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TemplateArgs](c, value))
}
func (FfiConverterTemplateArgs) Read(reader io.Reader) TemplateArgs {
	id := readInt32(reader)
	switch id {
	case 1:
		return TemplateArgsEvmWalletFilter{
			FfiConverterEvmWalletFilterInputINSTANCE.Read(reader),
		}
	case 2:
		return TemplateArgsEvmContractEvents{
			FfiConverterEvmContractEventsInputINSTANCE.Read(reader),
		}
	case 3:
		return TemplateArgsEvmAbiFilter{
			FfiConverterEvmAbiFilterInputINSTANCE.Read(reader),
		}
	case 4:
		return TemplateArgsSolanaWalletFilter{
			FfiConverterSolanaWalletFilterInputINSTANCE.Read(reader),
		}
	case 5:
		return TemplateArgsBitcoinWalletFilter{
			FfiConverterBitcoinWalletFilterInputINSTANCE.Read(reader),
		}
	case 6:
		return TemplateArgsXrplWalletFilter{
			FfiConverterXrplWalletFilterInputINSTANCE.Read(reader),
		}
	case 7:
		return TemplateArgsHyperliquidWalletEventsFilter{
			FfiConverterHyperliquidWalletEventsFilterInputINSTANCE.Read(reader),
		}
	case 8:
		return TemplateArgsStellarWalletTransactionsSourceAccountFilter{
			FfiConverterStellarWalletTransactionsFilterInputINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterTemplateArgs.Read()", id))
	}
}

func (FfiConverterTemplateArgs) Write(writer io.Writer, value TemplateArgs) {
	switch variant_value := value.(type) {
	case TemplateArgsEvmWalletFilter:
		writeInt32(writer, 1)
		FfiConverterEvmWalletFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsEvmContractEvents:
		writeInt32(writer, 2)
		FfiConverterEvmContractEventsInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsEvmAbiFilter:
		writeInt32(writer, 3)
		FfiConverterEvmAbiFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsSolanaWalletFilter:
		writeInt32(writer, 4)
		FfiConverterSolanaWalletFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsBitcoinWalletFilter:
		writeInt32(writer, 5)
		FfiConverterBitcoinWalletFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsXrplWalletFilter:
		writeInt32(writer, 6)
		FfiConverterXrplWalletFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsHyperliquidWalletEventsFilter:
		writeInt32(writer, 7)
		FfiConverterHyperliquidWalletEventsFilterInputINSTANCE.Write(writer, variant_value.Field0)
	case TemplateArgsStellarWalletTransactionsSourceAccountFilter:
		writeInt32(writer, 8)
		FfiConverterStellarWalletTransactionsFilterInputINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterTemplateArgs.Write", value))
	}
}

type FfiDestroyerTemplateArgs struct{}

func (_ FfiDestroyerTemplateArgs) Destroy(value TemplateArgs) {
	value.Destroy()
}

// Position a webhook begins (or resumes) delivering from when activated.
type WebhookStartFrom uint

const (
	// Resume from the last-delivered block.
	WebhookStartFromLast WebhookStartFrom = 1
	// Start from the newest available block.
	WebhookStartFromLatest WebhookStartFrom = 2
)

type FfiConverterWebhookStartFrom struct{}

var FfiConverterWebhookStartFromINSTANCE = FfiConverterWebhookStartFrom{}

func (c FfiConverterWebhookStartFrom) Lift(rb RustBufferI) WebhookStartFrom {
	return LiftFromRustBuffer[WebhookStartFrom](c, rb)
}

func (c FfiConverterWebhookStartFrom) Lower(value WebhookStartFrom) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookStartFrom](c, value)
}

func (c FfiConverterWebhookStartFrom) LowerExternal(value WebhookStartFrom) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookStartFrom](c, value))
}
func (FfiConverterWebhookStartFrom) Read(reader io.Reader) WebhookStartFrom {
	id := readInt32(reader)
	return WebhookStartFrom(id)
}

func (FfiConverterWebhookStartFrom) Write(writer io.Writer, value WebhookStartFrom) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerWebhookStartFrom struct{}

func (_ FfiDestroyerWebhookStartFrom) Destroy(value WebhookStartFrom) {
}

// Identifier of a predefined webhook filter template.
type WebhookTemplateId uint

const (
	WebhookTemplateIdEvmWalletFilter                              WebhookTemplateId = 1
	WebhookTemplateIdEvmContractEvents                            WebhookTemplateId = 2
	WebhookTemplateIdEvmAbiFilter                                 WebhookTemplateId = 3
	WebhookTemplateIdSolanaWalletFilter                           WebhookTemplateId = 4
	WebhookTemplateIdBitcoinWalletFilter                          WebhookTemplateId = 5
	WebhookTemplateIdXrplWalletFilter                             WebhookTemplateId = 6
	WebhookTemplateIdHyperliquidWalletEventsFilter                WebhookTemplateId = 7
	WebhookTemplateIdStellarWalletTransactionsSourceAccountFilter WebhookTemplateId = 8
)

type FfiConverterWebhookTemplateId struct{}

var FfiConverterWebhookTemplateIdINSTANCE = FfiConverterWebhookTemplateId{}

func (c FfiConverterWebhookTemplateId) Lift(rb RustBufferI) WebhookTemplateId {
	return LiftFromRustBuffer[WebhookTemplateId](c, rb)
}

func (c FfiConverterWebhookTemplateId) Lower(value WebhookTemplateId) C.RustBuffer {
	return LowerIntoRustBuffer[WebhookTemplateId](c, value)
}

func (c FfiConverterWebhookTemplateId) LowerExternal(value WebhookTemplateId) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[WebhookTemplateId](c, value))
}
func (FfiConverterWebhookTemplateId) Read(reader io.Reader) WebhookTemplateId {
	id := readInt32(reader)
	return WebhookTemplateId(id)
}

func (FfiConverterWebhookTemplateId) Write(writer io.Writer, value WebhookTemplateId) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerWebhookTemplateId struct{}

func (_ FfiDestroyerWebhookTemplateId) Destroy(value WebhookTemplateId) {
}

// `XrplWalletFilter` template arguments in either inline or by-list form.
type XrplWalletFilterInput interface {
	Destroy()
}
type XrplWalletFilterInputInline struct {
	Field0 XrplWalletFilterTemplate
}

func (e XrplWalletFilterInputInline) Destroy() {
	FfiDestroyerXrplWalletFilterTemplate{}.Destroy(e.Field0)
}

type XrplWalletFilterInputByList struct {
	Field0 XrplWalletFilterByListTemplate
}

func (e XrplWalletFilterInputByList) Destroy() {
	FfiDestroyerXrplWalletFilterByListTemplate{}.Destroy(e.Field0)
}

type FfiConverterXrplWalletFilterInput struct{}

var FfiConverterXrplWalletFilterInputINSTANCE = FfiConverterXrplWalletFilterInput{}

func (c FfiConverterXrplWalletFilterInput) Lift(rb RustBufferI) XrplWalletFilterInput {
	return LiftFromRustBuffer[XrplWalletFilterInput](c, rb)
}

func (c FfiConverterXrplWalletFilterInput) Lower(value XrplWalletFilterInput) C.RustBuffer {
	return LowerIntoRustBuffer[XrplWalletFilterInput](c, value)
}

func (c FfiConverterXrplWalletFilterInput) LowerExternal(value XrplWalletFilterInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[XrplWalletFilterInput](c, value))
}
func (FfiConverterXrplWalletFilterInput) Read(reader io.Reader) XrplWalletFilterInput {
	id := readInt32(reader)
	switch id {
	case 1:
		return XrplWalletFilterInputInline{
			FfiConverterXrplWalletFilterTemplateINSTANCE.Read(reader),
		}
	case 2:
		return XrplWalletFilterInputByList{
			FfiConverterXrplWalletFilterByListTemplateINSTANCE.Read(reader),
		}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterXrplWalletFilterInput.Read()", id))
	}
}

func (FfiConverterXrplWalletFilterInput) Write(writer io.Writer, value XrplWalletFilterInput) {
	switch variant_value := value.(type) {
	case XrplWalletFilterInputInline:
		writeInt32(writer, 1)
		FfiConverterXrplWalletFilterTemplateINSTANCE.Write(writer, variant_value.Field0)
	case XrplWalletFilterInputByList:
		writeInt32(writer, 2)
		FfiConverterXrplWalletFilterByListTemplateINSTANCE.Write(writer, variant_value.Field0)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterXrplWalletFilterInput.Write", value))
	}
}

type FfiDestroyerXrplWalletFilterInput struct{}

func (_ FfiDestroyerXrplWalletFilterInput) Destroy(value XrplWalletFilterInput) {
	value.Destroy()
}

type FfiConverterOptionalInt32 struct{}

var FfiConverterOptionalInt32INSTANCE = FfiConverterOptionalInt32{}

func (c FfiConverterOptionalInt32) Lift(rb RustBufferI) *int32 {
	return LiftFromRustBuffer[*int32](c, rb)
}

func (_ FfiConverterOptionalInt32) Read(reader io.Reader) *int32 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterInt32INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalInt32) Lower(value *int32) C.RustBuffer {
	return LowerIntoRustBuffer[*int32](c, value)
}

func (c FfiConverterOptionalInt32) LowerExternal(value *int32) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*int32](c, value))
}

func (_ FfiConverterOptionalInt32) Write(writer io.Writer, value *int32) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterInt32INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalInt32 struct{}

func (_ FfiDestroyerOptionalInt32) Destroy(value *int32) {
	if value != nil {
		FfiDestroyerInt32{}.Destroy(*value)
	}
}

type FfiConverterOptionalInt64 struct{}

var FfiConverterOptionalInt64INSTANCE = FfiConverterOptionalInt64{}

func (c FfiConverterOptionalInt64) Lift(rb RustBufferI) *int64 {
	return LiftFromRustBuffer[*int64](c, rb)
}

func (_ FfiConverterOptionalInt64) Read(reader io.Reader) *int64 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterInt64INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalInt64) Lower(value *int64) C.RustBuffer {
	return LowerIntoRustBuffer[*int64](c, value)
}

func (c FfiConverterOptionalInt64) LowerExternal(value *int64) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*int64](c, value))
}

func (_ FfiConverterOptionalInt64) Write(writer io.Writer, value *int64) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterInt64INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalInt64 struct{}

func (_ FfiDestroyerOptionalInt64) Destroy(value *int64) {
	if value != nil {
		FfiDestroyerInt64{}.Destroy(*value)
	}
}

type FfiConverterOptionalBool struct{}

var FfiConverterOptionalBoolINSTANCE = FfiConverterOptionalBool{}

func (c FfiConverterOptionalBool) Lift(rb RustBufferI) *bool {
	return LiftFromRustBuffer[*bool](c, rb)
}

func (_ FfiConverterOptionalBool) Read(reader io.Reader) *bool {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterBoolINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalBool) Lower(value *bool) C.RustBuffer {
	return LowerIntoRustBuffer[*bool](c, value)
}

func (c FfiConverterOptionalBool) LowerExternal(value *bool) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*bool](c, value))
}

func (_ FfiConverterOptionalBool) Write(writer io.Writer, value *bool) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterBoolINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalBool struct{}

func (_ FfiDestroyerOptionalBool) Destroy(value *bool) {
	if value != nil {
		FfiDestroyerBool{}.Destroy(*value)
	}
}

type FfiConverterOptionalString struct{}

var FfiConverterOptionalStringINSTANCE = FfiConverterOptionalString{}

func (c FfiConverterOptionalString) Lift(rb RustBufferI) *string {
	return LiftFromRustBuffer[*string](c, rb)
}

func (_ FfiConverterOptionalString) Read(reader io.Reader) *string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalString) Lower(value *string) C.RustBuffer {
	return LowerIntoRustBuffer[*string](c, value)
}

func (c FfiConverterOptionalString) LowerExternal(value *string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*string](c, value))
}

func (_ FfiConverterOptionalString) Write(writer io.Writer, value *string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalString struct{}

func (_ FfiDestroyerOptionalString) Destroy(value *string) {
	if value != nil {
		FfiDestroyerString{}.Destroy(*value)
	}
}

type FfiConverterOptionalAccountTag struct{}

var FfiConverterOptionalAccountTagINSTANCE = FfiConverterOptionalAccountTag{}

func (c FfiConverterOptionalAccountTag) Lift(rb RustBufferI) *AccountTag {
	return LiftFromRustBuffer[*AccountTag](c, rb)
}

func (_ FfiConverterOptionalAccountTag) Read(reader io.Reader) *AccountTag {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterAccountTagINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalAccountTag) Lower(value *AccountTag) C.RustBuffer {
	return LowerIntoRustBuffer[*AccountTag](c, value)
}

func (c FfiConverterOptionalAccountTag) LowerExternal(value *AccountTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*AccountTag](c, value))
}

func (_ FfiConverterOptionalAccountTag) Write(writer io.Writer, value *AccountTag) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterAccountTagINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalAccountTag struct{}

func (_ FfiDestroyerOptionalAccountTag) Destroy(value *AccountTag) {
	if value != nil {
		FfiDestroyerAccountTag{}.Destroy(*value)
	}
}

type FfiConverterOptionalAddressBookConfig struct{}

var FfiConverterOptionalAddressBookConfigINSTANCE = FfiConverterOptionalAddressBookConfig{}

func (c FfiConverterOptionalAddressBookConfig) Lift(rb RustBufferI) *AddressBookConfig {
	return LiftFromRustBuffer[*AddressBookConfig](c, rb)
}

func (_ FfiConverterOptionalAddressBookConfig) Read(reader io.Reader) *AddressBookConfig {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterAddressBookConfigINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalAddressBookConfig) Lower(value *AddressBookConfig) C.RustBuffer {
	return LowerIntoRustBuffer[*AddressBookConfig](c, value)
}

func (c FfiConverterOptionalAddressBookConfig) LowerExternal(value *AddressBookConfig) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*AddressBookConfig](c, value))
}

func (_ FfiConverterOptionalAddressBookConfig) Write(writer io.Writer, value *AddressBookConfig) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterAddressBookConfigINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalAddressBookConfig struct{}

func (_ FfiDestroyerOptionalAddressBookConfig) Destroy(value *AddressBookConfig) {
	if value != nil {
		FfiDestroyerAddressBookConfig{}.Destroy(*value)
	}
}

type FfiConverterOptionalBulkAddTagData struct{}

var FfiConverterOptionalBulkAddTagDataINSTANCE = FfiConverterOptionalBulkAddTagData{}

func (c FfiConverterOptionalBulkAddTagData) Lift(rb RustBufferI) *BulkAddTagData {
	return LiftFromRustBuffer[*BulkAddTagData](c, rb)
}

func (_ FfiConverterOptionalBulkAddTagData) Read(reader io.Reader) *BulkAddTagData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterBulkAddTagDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalBulkAddTagData) Lower(value *BulkAddTagData) C.RustBuffer {
	return LowerIntoRustBuffer[*BulkAddTagData](c, value)
}

func (c FfiConverterOptionalBulkAddTagData) LowerExternal(value *BulkAddTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*BulkAddTagData](c, value))
}

func (_ FfiConverterOptionalBulkAddTagData) Write(writer io.Writer, value *BulkAddTagData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterBulkAddTagDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalBulkAddTagData struct{}

func (_ FfiDestroyerOptionalBulkAddTagData) Destroy(value *BulkAddTagData) {
	if value != nil {
		FfiDestroyerBulkAddTagData{}.Destroy(*value)
	}
}

type FfiConverterOptionalBulkRemoveTagData struct{}

var FfiConverterOptionalBulkRemoveTagDataINSTANCE = FfiConverterOptionalBulkRemoveTagData{}

func (c FfiConverterOptionalBulkRemoveTagData) Lift(rb RustBufferI) *BulkRemoveTagData {
	return LiftFromRustBuffer[*BulkRemoveTagData](c, rb)
}

func (_ FfiConverterOptionalBulkRemoveTagData) Read(reader io.Reader) *BulkRemoveTagData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterBulkRemoveTagDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalBulkRemoveTagData) Lower(value *BulkRemoveTagData) C.RustBuffer {
	return LowerIntoRustBuffer[*BulkRemoveTagData](c, value)
}

func (c FfiConverterOptionalBulkRemoveTagData) LowerExternal(value *BulkRemoveTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*BulkRemoveTagData](c, value))
}

func (_ FfiConverterOptionalBulkRemoveTagData) Write(writer io.Writer, value *BulkRemoveTagData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterBulkRemoveTagDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalBulkRemoveTagData struct{}

func (_ FfiDestroyerOptionalBulkRemoveTagData) Destroy(value *BulkRemoveTagData) {
	if value != nil {
		FfiDestroyerBulkRemoveTagData{}.Destroy(*value)
	}
}

type FfiConverterOptionalBulkUpdateEndpointStatusData struct{}

var FfiConverterOptionalBulkUpdateEndpointStatusDataINSTANCE = FfiConverterOptionalBulkUpdateEndpointStatusData{}

func (c FfiConverterOptionalBulkUpdateEndpointStatusData) Lift(rb RustBufferI) *BulkUpdateEndpointStatusData {
	return LiftFromRustBuffer[*BulkUpdateEndpointStatusData](c, rb)
}

func (_ FfiConverterOptionalBulkUpdateEndpointStatusData) Read(reader io.Reader) *BulkUpdateEndpointStatusData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterBulkUpdateEndpointStatusDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalBulkUpdateEndpointStatusData) Lower(value *BulkUpdateEndpointStatusData) C.RustBuffer {
	return LowerIntoRustBuffer[*BulkUpdateEndpointStatusData](c, value)
}

func (c FfiConverterOptionalBulkUpdateEndpointStatusData) LowerExternal(value *BulkUpdateEndpointStatusData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*BulkUpdateEndpointStatusData](c, value))
}

func (_ FfiConverterOptionalBulkUpdateEndpointStatusData) Write(writer io.Writer, value *BulkUpdateEndpointStatusData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterBulkUpdateEndpointStatusDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalBulkUpdateEndpointStatusData struct{}

func (_ FfiDestroyerOptionalBulkUpdateEndpointStatusData) Destroy(value *BulkUpdateEndpointStatusData) {
	if value != nil {
		FfiDestroyerBulkUpdateEndpointStatusData{}.Destroy(*value)
	}
}

type FfiConverterOptionalCreateRequestFilterData struct{}

var FfiConverterOptionalCreateRequestFilterDataINSTANCE = FfiConverterOptionalCreateRequestFilterData{}

func (c FfiConverterOptionalCreateRequestFilterData) Lift(rb RustBufferI) *CreateRequestFilterData {
	return LiftFromRustBuffer[*CreateRequestFilterData](c, rb)
}

func (_ FfiConverterOptionalCreateRequestFilterData) Read(reader io.Reader) *CreateRequestFilterData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterCreateRequestFilterDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalCreateRequestFilterData) Lower(value *CreateRequestFilterData) C.RustBuffer {
	return LowerIntoRustBuffer[*CreateRequestFilterData](c, value)
}

func (c FfiConverterOptionalCreateRequestFilterData) LowerExternal(value *CreateRequestFilterData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*CreateRequestFilterData](c, value))
}

func (_ FfiConverterOptionalCreateRequestFilterData) Write(writer io.Writer, value *CreateRequestFilterData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterCreateRequestFilterDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalCreateRequestFilterData struct{}

func (_ FfiDestroyerOptionalCreateRequestFilterData) Destroy(value *CreateRequestFilterData) {
	if value != nil {
		FfiDestroyerCreateRequestFilterData{}.Destroy(*value)
	}
}

type FfiConverterOptionalCreateTeamData struct{}

var FfiConverterOptionalCreateTeamDataINSTANCE = FfiConverterOptionalCreateTeamData{}

func (c FfiConverterOptionalCreateTeamData) Lift(rb RustBufferI) *CreateTeamData {
	return LiftFromRustBuffer[*CreateTeamData](c, rb)
}

func (_ FfiConverterOptionalCreateTeamData) Read(reader io.Reader) *CreateTeamData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterCreateTeamDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalCreateTeamData) Lower(value *CreateTeamData) C.RustBuffer {
	return LowerIntoRustBuffer[*CreateTeamData](c, value)
}

func (c FfiConverterOptionalCreateTeamData) LowerExternal(value *CreateTeamData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*CreateTeamData](c, value))
}

func (_ FfiConverterOptionalCreateTeamData) Write(writer io.Writer, value *CreateTeamData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterCreateTeamDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalCreateTeamData struct{}

func (_ FfiDestroyerOptionalCreateTeamData) Destroy(value *CreateTeamData) {
	if value != nil {
		FfiDestroyerCreateTeamData{}.Destroy(*value)
	}
}

type FfiConverterOptionalDeleteAccountTagData struct{}

var FfiConverterOptionalDeleteAccountTagDataINSTANCE = FfiConverterOptionalDeleteAccountTagData{}

func (c FfiConverterOptionalDeleteAccountTagData) Lift(rb RustBufferI) *DeleteAccountTagData {
	return LiftFromRustBuffer[*DeleteAccountTagData](c, rb)
}

func (_ FfiConverterOptionalDeleteAccountTagData) Read(reader io.Reader) *DeleteAccountTagData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterDeleteAccountTagDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalDeleteAccountTagData) Lower(value *DeleteAccountTagData) C.RustBuffer {
	return LowerIntoRustBuffer[*DeleteAccountTagData](c, value)
}

func (c FfiConverterOptionalDeleteAccountTagData) LowerExternal(value *DeleteAccountTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*DeleteAccountTagData](c, value))
}

func (_ FfiConverterOptionalDeleteAccountTagData) Write(writer io.Writer, value *DeleteAccountTagData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterDeleteAccountTagDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalDeleteAccountTagData struct{}

func (_ FfiDestroyerOptionalDeleteAccountTagData) Destroy(value *DeleteAccountTagData) {
	if value != nil {
		FfiDestroyerDeleteAccountTagData{}.Destroy(*value)
	}
}

type FfiConverterOptionalDeleteTeamData struct{}

var FfiConverterOptionalDeleteTeamDataINSTANCE = FfiConverterOptionalDeleteTeamData{}

func (c FfiConverterOptionalDeleteTeamData) Lift(rb RustBufferI) *DeleteTeamData {
	return LiftFromRustBuffer[*DeleteTeamData](c, rb)
}

func (_ FfiConverterOptionalDeleteTeamData) Read(reader io.Reader) *DeleteTeamData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterDeleteTeamDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalDeleteTeamData) Lower(value *DeleteTeamData) C.RustBuffer {
	return LowerIntoRustBuffer[*DeleteTeamData](c, value)
}

func (c FfiConverterOptionalDeleteTeamData) LowerExternal(value *DeleteTeamData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*DeleteTeamData](c, value))
}

func (_ FfiConverterOptionalDeleteTeamData) Write(writer io.Writer, value *DeleteTeamData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterDeleteTeamDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalDeleteTeamData struct{}

func (_ FfiDestroyerOptionalDeleteTeamData) Destroy(value *DeleteTeamData) {
	if value != nil {
		FfiDestroyerDeleteTeamData{}.Destroy(*value)
	}
}

type FfiConverterOptionalEndpointIpCustomHeaderOption struct{}

var FfiConverterOptionalEndpointIpCustomHeaderOptionINSTANCE = FfiConverterOptionalEndpointIpCustomHeaderOption{}

func (c FfiConverterOptionalEndpointIpCustomHeaderOption) Lift(rb RustBufferI) *EndpointIpCustomHeaderOption {
	return LiftFromRustBuffer[*EndpointIpCustomHeaderOption](c, rb)
}

func (_ FfiConverterOptionalEndpointIpCustomHeaderOption) Read(reader io.Reader) *EndpointIpCustomHeaderOption {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterEndpointIpCustomHeaderOptionINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalEndpointIpCustomHeaderOption) Lower(value *EndpointIpCustomHeaderOption) C.RustBuffer {
	return LowerIntoRustBuffer[*EndpointIpCustomHeaderOption](c, value)
}

func (c FfiConverterOptionalEndpointIpCustomHeaderOption) LowerExternal(value *EndpointIpCustomHeaderOption) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*EndpointIpCustomHeaderOption](c, value))
}

func (_ FfiConverterOptionalEndpointIpCustomHeaderOption) Write(writer io.Writer, value *EndpointIpCustomHeaderOption) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterEndpointIpCustomHeaderOptionINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalEndpointIpCustomHeaderOption struct{}

func (_ FfiDestroyerOptionalEndpointIpCustomHeaderOption) Destroy(value *EndpointIpCustomHeaderOption) {
	if value != nil {
		FfiDestroyerEndpointIpCustomHeaderOption{}.Destroy(*value)
	}
}

type FfiConverterOptionalEndpointRateLimits struct{}

var FfiConverterOptionalEndpointRateLimitsINSTANCE = FfiConverterOptionalEndpointRateLimits{}

func (c FfiConverterOptionalEndpointRateLimits) Lift(rb RustBufferI) *EndpointRateLimits {
	return LiftFromRustBuffer[*EndpointRateLimits](c, rb)
}

func (_ FfiConverterOptionalEndpointRateLimits) Read(reader io.Reader) *EndpointRateLimits {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterEndpointRateLimitsINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalEndpointRateLimits) Lower(value *EndpointRateLimits) C.RustBuffer {
	return LowerIntoRustBuffer[*EndpointRateLimits](c, value)
}

func (c FfiConverterOptionalEndpointRateLimits) LowerExternal(value *EndpointRateLimits) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*EndpointRateLimits](c, value))
}

func (_ FfiConverterOptionalEndpointRateLimits) Write(writer io.Writer, value *EndpointRateLimits) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterEndpointRateLimitsINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalEndpointRateLimits struct{}

func (_ FfiDestroyerOptionalEndpointRateLimits) Destroy(value *EndpointRateLimits) {
	if value != nil {
		FfiDestroyerEndpointRateLimits{}.Destroy(*value)
	}
}

type FfiConverterOptionalEndpointSecurity struct{}

var FfiConverterOptionalEndpointSecurityINSTANCE = FfiConverterOptionalEndpointSecurity{}

func (c FfiConverterOptionalEndpointSecurity) Lift(rb RustBufferI) *EndpointSecurity {
	return LiftFromRustBuffer[*EndpointSecurity](c, rb)
}

func (_ FfiConverterOptionalEndpointSecurity) Read(reader io.Reader) *EndpointSecurity {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterEndpointSecurityINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalEndpointSecurity) Lower(value *EndpointSecurity) C.RustBuffer {
	return LowerIntoRustBuffer[*EndpointSecurity](c, value)
}

func (c FfiConverterOptionalEndpointSecurity) LowerExternal(value *EndpointSecurity) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*EndpointSecurity](c, value))
}

func (_ FfiConverterOptionalEndpointSecurity) Write(writer io.Writer, value *EndpointSecurity) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterEndpointSecurityINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalEndpointSecurity struct{}

func (_ FfiDestroyerOptionalEndpointSecurity) Destroy(value *EndpointSecurity) {
	if value != nil {
		FfiDestroyerEndpointSecurity{}.Destroy(*value)
	}
}

type FfiConverterOptionalEndpointSecurityOptions struct{}

var FfiConverterOptionalEndpointSecurityOptionsINSTANCE = FfiConverterOptionalEndpointSecurityOptions{}

func (c FfiConverterOptionalEndpointSecurityOptions) Lift(rb RustBufferI) *EndpointSecurityOptions {
	return LiftFromRustBuffer[*EndpointSecurityOptions](c, rb)
}

func (_ FfiConverterOptionalEndpointSecurityOptions) Read(reader io.Reader) *EndpointSecurityOptions {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterEndpointSecurityOptionsINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalEndpointSecurityOptions) Lower(value *EndpointSecurityOptions) C.RustBuffer {
	return LowerIntoRustBuffer[*EndpointSecurityOptions](c, value)
}

func (c FfiConverterOptionalEndpointSecurityOptions) LowerExternal(value *EndpointSecurityOptions) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*EndpointSecurityOptions](c, value))
}

func (_ FfiConverterOptionalEndpointSecurityOptions) Write(writer io.Writer, value *EndpointSecurityOptions) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterEndpointSecurityOptionsINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalEndpointSecurityOptions struct{}

func (_ FfiDestroyerOptionalEndpointSecurityOptions) Destroy(value *EndpointSecurityOptions) {
	if value != nil {
		FfiDestroyerEndpointSecurityOptions{}.Destroy(*value)
	}
}

type FfiConverterOptionalGetEndpointUrlsData struct{}

var FfiConverterOptionalGetEndpointUrlsDataINSTANCE = FfiConverterOptionalGetEndpointUrlsData{}

func (c FfiConverterOptionalGetEndpointUrlsData) Lift(rb RustBufferI) *GetEndpointUrlsData {
	return LiftFromRustBuffer[*GetEndpointUrlsData](c, rb)
}

func (_ FfiConverterOptionalGetEndpointUrlsData) Read(reader io.Reader) *GetEndpointUrlsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterGetEndpointUrlsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalGetEndpointUrlsData) Lower(value *GetEndpointUrlsData) C.RustBuffer {
	return LowerIntoRustBuffer[*GetEndpointUrlsData](c, value)
}

func (c FfiConverterOptionalGetEndpointUrlsData) LowerExternal(value *GetEndpointUrlsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*GetEndpointUrlsData](c, value))
}

func (_ FfiConverterOptionalGetEndpointUrlsData) Write(writer io.Writer, value *GetEndpointUrlsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterGetEndpointUrlsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalGetEndpointUrlsData struct{}

func (_ FfiDestroyerOptionalGetEndpointUrlsData) Destroy(value *GetEndpointUrlsData) {
	if value != nil {
		FfiDestroyerGetEndpointUrlsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalGetMethodRateLimitsData struct{}

var FfiConverterOptionalGetMethodRateLimitsDataINSTANCE = FfiConverterOptionalGetMethodRateLimitsData{}

func (c FfiConverterOptionalGetMethodRateLimitsData) Lift(rb RustBufferI) *GetMethodRateLimitsData {
	return LiftFromRustBuffer[*GetMethodRateLimitsData](c, rb)
}

func (_ FfiConverterOptionalGetMethodRateLimitsData) Read(reader io.Reader) *GetMethodRateLimitsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterGetMethodRateLimitsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalGetMethodRateLimitsData) Lower(value *GetMethodRateLimitsData) C.RustBuffer {
	return LowerIntoRustBuffer[*GetMethodRateLimitsData](c, value)
}

func (c FfiConverterOptionalGetMethodRateLimitsData) LowerExternal(value *GetMethodRateLimitsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*GetMethodRateLimitsData](c, value))
}

func (_ FfiConverterOptionalGetMethodRateLimitsData) Write(writer io.Writer, value *GetMethodRateLimitsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterGetMethodRateLimitsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalGetMethodRateLimitsData struct{}

func (_ FfiDestroyerOptionalGetMethodRateLimitsData) Destroy(value *GetMethodRateLimitsData) {
	if value != nil {
		FfiDestroyerGetMethodRateLimitsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalGetRateLimitsData struct{}

var FfiConverterOptionalGetRateLimitsDataINSTANCE = FfiConverterOptionalGetRateLimitsData{}

func (c FfiConverterOptionalGetRateLimitsData) Lift(rb RustBufferI) *GetRateLimitsData {
	return LiftFromRustBuffer[*GetRateLimitsData](c, rb)
}

func (_ FfiConverterOptionalGetRateLimitsData) Read(reader io.Reader) *GetRateLimitsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterGetRateLimitsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalGetRateLimitsData) Lower(value *GetRateLimitsData) C.RustBuffer {
	return LowerIntoRustBuffer[*GetRateLimitsData](c, value)
}

func (c FfiConverterOptionalGetRateLimitsData) LowerExternal(value *GetRateLimitsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*GetRateLimitsData](c, value))
}

func (_ FfiConverterOptionalGetRateLimitsData) Write(writer io.Writer, value *GetRateLimitsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterGetRateLimitsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalGetRateLimitsData struct{}

func (_ FfiDestroyerOptionalGetRateLimitsData) Destroy(value *GetRateLimitsData) {
	if value != nil {
		FfiDestroyerGetRateLimitsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalIpCustomHeaderData struct{}

var FfiConverterOptionalIpCustomHeaderDataINSTANCE = FfiConverterOptionalIpCustomHeaderData{}

func (c FfiConverterOptionalIpCustomHeaderData) Lift(rb RustBufferI) *IpCustomHeaderData {
	return LiftFromRustBuffer[*IpCustomHeaderData](c, rb)
}

func (_ FfiConverterOptionalIpCustomHeaderData) Read(reader io.Reader) *IpCustomHeaderData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterIpCustomHeaderDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalIpCustomHeaderData) Lower(value *IpCustomHeaderData) C.RustBuffer {
	return LowerIntoRustBuffer[*IpCustomHeaderData](c, value)
}

func (c FfiConverterOptionalIpCustomHeaderData) LowerExternal(value *IpCustomHeaderData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*IpCustomHeaderData](c, value))
}

func (_ FfiConverterOptionalIpCustomHeaderData) Write(writer io.Writer, value *IpCustomHeaderData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterIpCustomHeaderDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalIpCustomHeaderData struct{}

func (_ FfiDestroyerOptionalIpCustomHeaderData) Destroy(value *IpCustomHeaderData) {
	if value != nil {
		FfiDestroyerIpCustomHeaderData{}.Destroy(*value)
	}
}

type FfiConverterOptionalListInvoicesData struct{}

var FfiConverterOptionalListInvoicesDataINSTANCE = FfiConverterOptionalListInvoicesData{}

func (c FfiConverterOptionalListInvoicesData) Lift(rb RustBufferI) *ListInvoicesData {
	return LiftFromRustBuffer[*ListInvoicesData](c, rb)
}

func (_ FfiConverterOptionalListInvoicesData) Read(reader io.Reader) *ListInvoicesData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterListInvoicesDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalListInvoicesData) Lower(value *ListInvoicesData) C.RustBuffer {
	return LowerIntoRustBuffer[*ListInvoicesData](c, value)
}

func (c FfiConverterOptionalListInvoicesData) LowerExternal(value *ListInvoicesData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*ListInvoicesData](c, value))
}

func (_ FfiConverterOptionalListInvoicesData) Write(writer io.Writer, value *ListInvoicesData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterListInvoicesDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalListInvoicesData struct{}

func (_ FfiDestroyerOptionalListInvoicesData) Destroy(value *ListInvoicesData) {
	if value != nil {
		FfiDestroyerListInvoicesData{}.Destroy(*value)
	}
}

type FfiConverterOptionalListPaymentsData struct{}

var FfiConverterOptionalListPaymentsDataINSTANCE = FfiConverterOptionalListPaymentsData{}

func (c FfiConverterOptionalListPaymentsData) Lift(rb RustBufferI) *ListPaymentsData {
	return LiftFromRustBuffer[*ListPaymentsData](c, rb)
}

func (_ FfiConverterOptionalListPaymentsData) Read(reader io.Reader) *ListPaymentsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterListPaymentsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalListPaymentsData) Lower(value *ListPaymentsData) C.RustBuffer {
	return LowerIntoRustBuffer[*ListPaymentsData](c, value)
}

func (c FfiConverterOptionalListPaymentsData) LowerExternal(value *ListPaymentsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*ListPaymentsData](c, value))
}

func (_ FfiConverterOptionalListPaymentsData) Write(writer io.Writer, value *ListPaymentsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterListPaymentsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalListPaymentsData struct{}

func (_ FfiDestroyerOptionalListPaymentsData) Destroy(value *ListPaymentsData) {
	if value != nil {
		FfiDestroyerListPaymentsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalListTagsData struct{}

var FfiConverterOptionalListTagsDataINSTANCE = FfiConverterOptionalListTagsData{}

func (c FfiConverterOptionalListTagsData) Lift(rb RustBufferI) *ListTagsData {
	return LiftFromRustBuffer[*ListTagsData](c, rb)
}

func (_ FfiConverterOptionalListTagsData) Read(reader io.Reader) *ListTagsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterListTagsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalListTagsData) Lower(value *ListTagsData) C.RustBuffer {
	return LowerIntoRustBuffer[*ListTagsData](c, value)
}

func (c FfiConverterOptionalListTagsData) LowerExternal(value *ListTagsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*ListTagsData](c, value))
}

func (_ FfiConverterOptionalListTagsData) Write(writer io.Writer, value *ListTagsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterListTagsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalListTagsData struct{}

func (_ FfiDestroyerOptionalListTagsData) Destroy(value *ListTagsData) {
	if value != nil {
		FfiDestroyerListTagsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalLogDetails struct{}

var FfiConverterOptionalLogDetailsINSTANCE = FfiConverterOptionalLogDetails{}

func (c FfiConverterOptionalLogDetails) Lift(rb RustBufferI) *LogDetails {
	return LiftFromRustBuffer[*LogDetails](c, rb)
}

func (_ FfiConverterOptionalLogDetails) Read(reader io.Reader) *LogDetails {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterLogDetailsINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalLogDetails) Lower(value *LogDetails) C.RustBuffer {
	return LowerIntoRustBuffer[*LogDetails](c, value)
}

func (c FfiConverterOptionalLogDetails) LowerExternal(value *LogDetails) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*LogDetails](c, value))
}

func (_ FfiConverterOptionalLogDetails) Write(writer io.Writer, value *LogDetails) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterLogDetailsINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalLogDetails struct{}

func (_ FfiDestroyerOptionalLogDetails) Destroy(value *LogDetails) {
	if value != nil {
		FfiDestroyerLogDetails{}.Destroy(*value)
	}
}

type FfiConverterOptionalMethodRateLimiter struct{}

var FfiConverterOptionalMethodRateLimiterINSTANCE = FfiConverterOptionalMethodRateLimiter{}

func (c FfiConverterOptionalMethodRateLimiter) Lift(rb RustBufferI) *MethodRateLimiter {
	return LiftFromRustBuffer[*MethodRateLimiter](c, rb)
}

func (_ FfiConverterOptionalMethodRateLimiter) Read(reader io.Reader) *MethodRateLimiter {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterMethodRateLimiterINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalMethodRateLimiter) Lower(value *MethodRateLimiter) C.RustBuffer {
	return LowerIntoRustBuffer[*MethodRateLimiter](c, value)
}

func (c FfiConverterOptionalMethodRateLimiter) LowerExternal(value *MethodRateLimiter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*MethodRateLimiter](c, value))
}

func (_ FfiConverterOptionalMethodRateLimiter) Write(writer io.Writer, value *MethodRateLimiter) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterMethodRateLimiterINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalMethodRateLimiter struct{}

func (_ FfiDestroyerOptionalMethodRateLimiter) Destroy(value *MethodRateLimiter) {
	if value != nil {
		FfiDestroyerMethodRateLimiter{}.Destroy(*value)
	}
}

type FfiConverterOptionalPagination struct{}

var FfiConverterOptionalPaginationINSTANCE = FfiConverterOptionalPagination{}

func (c FfiConverterOptionalPagination) Lift(rb RustBufferI) *Pagination {
	return LiftFromRustBuffer[*Pagination](c, rb)
}

func (_ FfiConverterOptionalPagination) Read(reader io.Reader) *Pagination {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterPaginationINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalPagination) Lower(value *Pagination) C.RustBuffer {
	return LowerIntoRustBuffer[*Pagination](c, value)
}

func (c FfiConverterOptionalPagination) LowerExternal(value *Pagination) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*Pagination](c, value))
}

func (_ FfiConverterOptionalPagination) Write(writer io.Writer, value *Pagination) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterPaginationINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalPagination struct{}

func (_ FfiDestroyerOptionalPagination) Destroy(value *Pagination) {
	if value != nil {
		FfiDestroyerPagination{}.Destroy(*value)
	}
}

type FfiConverterOptionalSingleEndpoint struct{}

var FfiConverterOptionalSingleEndpointINSTANCE = FfiConverterOptionalSingleEndpoint{}

func (c FfiConverterOptionalSingleEndpoint) Lift(rb RustBufferI) *SingleEndpoint {
	return LiftFromRustBuffer[*SingleEndpoint](c, rb)
}

func (_ FfiConverterOptionalSingleEndpoint) Read(reader io.Reader) *SingleEndpoint {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSingleEndpointINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSingleEndpoint) Lower(value *SingleEndpoint) C.RustBuffer {
	return LowerIntoRustBuffer[*SingleEndpoint](c, value)
}

func (c FfiConverterOptionalSingleEndpoint) LowerExternal(value *SingleEndpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*SingleEndpoint](c, value))
}

func (_ FfiConverterOptionalSingleEndpoint) Write(writer io.Writer, value *SingleEndpoint) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSingleEndpointINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSingleEndpoint struct{}

func (_ FfiDestroyerOptionalSingleEndpoint) Destroy(value *SingleEndpoint) {
	if value != nil {
		FfiDestroyerSingleEndpoint{}.Destroy(*value)
	}
}

type FfiConverterOptionalTeamDetail struct{}

var FfiConverterOptionalTeamDetailINSTANCE = FfiConverterOptionalTeamDetail{}

func (c FfiConverterOptionalTeamDetail) Lift(rb RustBufferI) *TeamDetail {
	return LiftFromRustBuffer[*TeamDetail](c, rb)
}

func (_ FfiConverterOptionalTeamDetail) Read(reader io.Reader) *TeamDetail {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterTeamDetailINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalTeamDetail) Lower(value *TeamDetail) C.RustBuffer {
	return LowerIntoRustBuffer[*TeamDetail](c, value)
}

func (c FfiConverterOptionalTeamDetail) LowerExternal(value *TeamDetail) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*TeamDetail](c, value))
}

func (_ FfiConverterOptionalTeamDetail) Write(writer io.Writer, value *TeamDetail) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterTeamDetailINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalTeamDetail struct{}

func (_ FfiDestroyerOptionalTeamDetail) Destroy(value *TeamDetail) {
	if value != nil {
		FfiDestroyerTeamDetail{}.Destroy(*value)
	}
}

type FfiConverterOptionalTeamMessageData struct{}

var FfiConverterOptionalTeamMessageDataINSTANCE = FfiConverterOptionalTeamMessageData{}

func (c FfiConverterOptionalTeamMessageData) Lift(rb RustBufferI) *TeamMessageData {
	return LiftFromRustBuffer[*TeamMessageData](c, rb)
}

func (_ FfiConverterOptionalTeamMessageData) Read(reader io.Reader) *TeamMessageData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterTeamMessageDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalTeamMessageData) Lower(value *TeamMessageData) C.RustBuffer {
	return LowerIntoRustBuffer[*TeamMessageData](c, value)
}

func (c FfiConverterOptionalTeamMessageData) LowerExternal(value *TeamMessageData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*TeamMessageData](c, value))
}

func (_ FfiConverterOptionalTeamMessageData) Write(writer io.Writer, value *TeamMessageData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterTeamMessageDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalTeamMessageData struct{}

func (_ FfiDestroyerOptionalTeamMessageData) Destroy(value *TeamMessageData) {
	if value != nil {
		FfiDestroyerTeamMessageData{}.Destroy(*value)
	}
}

type FfiConverterOptionalTeamUser struct{}

var FfiConverterOptionalTeamUserINSTANCE = FfiConverterOptionalTeamUser{}

func (c FfiConverterOptionalTeamUser) Lift(rb RustBufferI) *TeamUser {
	return LiftFromRustBuffer[*TeamUser](c, rb)
}

func (_ FfiConverterOptionalTeamUser) Read(reader io.Reader) *TeamUser {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterTeamUserINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalTeamUser) Lower(value *TeamUser) C.RustBuffer {
	return LowerIntoRustBuffer[*TeamUser](c, value)
}

func (c FfiConverterOptionalTeamUser) LowerExternal(value *TeamUser) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*TeamUser](c, value))
}

func (_ FfiConverterOptionalTeamUser) Write(writer io.Writer, value *TeamUser) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterTeamUserINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalTeamUser struct{}

func (_ FfiDestroyerOptionalTeamUser) Destroy(value *TeamUser) {
	if value != nil {
		FfiDestroyerTeamUser{}.Destroy(*value)
	}
}

type FfiConverterOptionalUpdateTeamEndpointsData struct{}

var FfiConverterOptionalUpdateTeamEndpointsDataINSTANCE = FfiConverterOptionalUpdateTeamEndpointsData{}

func (c FfiConverterOptionalUpdateTeamEndpointsData) Lift(rb RustBufferI) *UpdateTeamEndpointsData {
	return LiftFromRustBuffer[*UpdateTeamEndpointsData](c, rb)
}

func (_ FfiConverterOptionalUpdateTeamEndpointsData) Read(reader io.Reader) *UpdateTeamEndpointsData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUpdateTeamEndpointsDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUpdateTeamEndpointsData) Lower(value *UpdateTeamEndpointsData) C.RustBuffer {
	return LowerIntoRustBuffer[*UpdateTeamEndpointsData](c, value)
}

func (c FfiConverterOptionalUpdateTeamEndpointsData) LowerExternal(value *UpdateTeamEndpointsData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UpdateTeamEndpointsData](c, value))
}

func (_ FfiConverterOptionalUpdateTeamEndpointsData) Write(writer io.Writer, value *UpdateTeamEndpointsData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUpdateTeamEndpointsDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUpdateTeamEndpointsData struct{}

func (_ FfiDestroyerOptionalUpdateTeamEndpointsData) Destroy(value *UpdateTeamEndpointsData) {
	if value != nil {
		FfiDestroyerUpdateTeamEndpointsData{}.Destroy(*value)
	}
}

type FfiConverterOptionalUsageByChainData struct{}

var FfiConverterOptionalUsageByChainDataINSTANCE = FfiConverterOptionalUsageByChainData{}

func (c FfiConverterOptionalUsageByChainData) Lift(rb RustBufferI) *UsageByChainData {
	return LiftFromRustBuffer[*UsageByChainData](c, rb)
}

func (_ FfiConverterOptionalUsageByChainData) Read(reader io.Reader) *UsageByChainData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUsageByChainDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUsageByChainData) Lower(value *UsageByChainData) C.RustBuffer {
	return LowerIntoRustBuffer[*UsageByChainData](c, value)
}

func (c FfiConverterOptionalUsageByChainData) LowerExternal(value *UsageByChainData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UsageByChainData](c, value))
}

func (_ FfiConverterOptionalUsageByChainData) Write(writer io.Writer, value *UsageByChainData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUsageByChainDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUsageByChainData struct{}

func (_ FfiDestroyerOptionalUsageByChainData) Destroy(value *UsageByChainData) {
	if value != nil {
		FfiDestroyerUsageByChainData{}.Destroy(*value)
	}
}

type FfiConverterOptionalUsageByEndpointData struct{}

var FfiConverterOptionalUsageByEndpointDataINSTANCE = FfiConverterOptionalUsageByEndpointData{}

func (c FfiConverterOptionalUsageByEndpointData) Lift(rb RustBufferI) *UsageByEndpointData {
	return LiftFromRustBuffer[*UsageByEndpointData](c, rb)
}

func (_ FfiConverterOptionalUsageByEndpointData) Read(reader io.Reader) *UsageByEndpointData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUsageByEndpointDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUsageByEndpointData) Lower(value *UsageByEndpointData) C.RustBuffer {
	return LowerIntoRustBuffer[*UsageByEndpointData](c, value)
}

func (c FfiConverterOptionalUsageByEndpointData) LowerExternal(value *UsageByEndpointData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UsageByEndpointData](c, value))
}

func (_ FfiConverterOptionalUsageByEndpointData) Write(writer io.Writer, value *UsageByEndpointData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUsageByEndpointDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUsageByEndpointData struct{}

func (_ FfiDestroyerOptionalUsageByEndpointData) Destroy(value *UsageByEndpointData) {
	if value != nil {
		FfiDestroyerUsageByEndpointData{}.Destroy(*value)
	}
}

type FfiConverterOptionalUsageByMethodData struct{}

var FfiConverterOptionalUsageByMethodDataINSTANCE = FfiConverterOptionalUsageByMethodData{}

func (c FfiConverterOptionalUsageByMethodData) Lift(rb RustBufferI) *UsageByMethodData {
	return LiftFromRustBuffer[*UsageByMethodData](c, rb)
}

func (_ FfiConverterOptionalUsageByMethodData) Read(reader io.Reader) *UsageByMethodData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUsageByMethodDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUsageByMethodData) Lower(value *UsageByMethodData) C.RustBuffer {
	return LowerIntoRustBuffer[*UsageByMethodData](c, value)
}

func (c FfiConverterOptionalUsageByMethodData) LowerExternal(value *UsageByMethodData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UsageByMethodData](c, value))
}

func (_ FfiConverterOptionalUsageByMethodData) Write(writer io.Writer, value *UsageByMethodData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUsageByMethodDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUsageByMethodData struct{}

func (_ FfiDestroyerOptionalUsageByMethodData) Destroy(value *UsageByMethodData) {
	if value != nil {
		FfiDestroyerUsageByMethodData{}.Destroy(*value)
	}
}

type FfiConverterOptionalUsageByTagData struct{}

var FfiConverterOptionalUsageByTagDataINSTANCE = FfiConverterOptionalUsageByTagData{}

func (c FfiConverterOptionalUsageByTagData) Lift(rb RustBufferI) *UsageByTagData {
	return LiftFromRustBuffer[*UsageByTagData](c, rb)
}

func (_ FfiConverterOptionalUsageByTagData) Read(reader io.Reader) *UsageByTagData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUsageByTagDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUsageByTagData) Lower(value *UsageByTagData) C.RustBuffer {
	return LowerIntoRustBuffer[*UsageByTagData](c, value)
}

func (c FfiConverterOptionalUsageByTagData) LowerExternal(value *UsageByTagData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UsageByTagData](c, value))
}

func (_ FfiConverterOptionalUsageByTagData) Write(writer io.Writer, value *UsageByTagData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUsageByTagDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUsageByTagData struct{}

func (_ FfiDestroyerOptionalUsageByTagData) Destroy(value *UsageByTagData) {
	if value != nil {
		FfiDestroyerUsageByTagData{}.Destroy(*value)
	}
}

type FfiConverterOptionalUsageData struct{}

var FfiConverterOptionalUsageDataINSTANCE = FfiConverterOptionalUsageData{}

func (c FfiConverterOptionalUsageData) Lift(rb RustBufferI) *UsageData {
	return LiftFromRustBuffer[*UsageData](c, rb)
}

func (_ FfiConverterOptionalUsageData) Read(reader io.Reader) *UsageData {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUsageDataINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUsageData) Lower(value *UsageData) C.RustBuffer {
	return LowerIntoRustBuffer[*UsageData](c, value)
}

func (c FfiConverterOptionalUsageData) LowerExternal(value *UsageData) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*UsageData](c, value))
}

func (_ FfiConverterOptionalUsageData) Write(writer io.Writer, value *UsageData) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUsageDataINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUsageData struct{}

func (_ FfiDestroyerOptionalUsageData) Destroy(value *UsageData) {
	if value != nil {
		FfiDestroyerUsageData{}.Destroy(*value)
	}
}

type FfiConverterOptionalWebhookDestinationAttributes struct{}

var FfiConverterOptionalWebhookDestinationAttributesINSTANCE = FfiConverterOptionalWebhookDestinationAttributes{}

func (c FfiConverterOptionalWebhookDestinationAttributes) Lift(rb RustBufferI) *WebhookDestinationAttributes {
	return LiftFromRustBuffer[*WebhookDestinationAttributes](c, rb)
}

func (_ FfiConverterOptionalWebhookDestinationAttributes) Read(reader io.Reader) *WebhookDestinationAttributes {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterWebhookDestinationAttributesINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalWebhookDestinationAttributes) Lower(value *WebhookDestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[*WebhookDestinationAttributes](c, value)
}

func (c FfiConverterOptionalWebhookDestinationAttributes) LowerExternal(value *WebhookDestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*WebhookDestinationAttributes](c, value))
}

func (_ FfiConverterOptionalWebhookDestinationAttributes) Write(writer io.Writer, value *WebhookDestinationAttributes) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterWebhookDestinationAttributesINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalWebhookDestinationAttributes struct{}

func (_ FfiDestroyerOptionalWebhookDestinationAttributes) Destroy(value *WebhookDestinationAttributes) {
	if value != nil {
		FfiDestroyerWebhookDestinationAttributes{}.Destroy(*value)
	}
}

type FfiConverterOptionalDestinationAttributes struct{}

var FfiConverterOptionalDestinationAttributesINSTANCE = FfiConverterOptionalDestinationAttributes{}

func (c FfiConverterOptionalDestinationAttributes) Lift(rb RustBufferI) *DestinationAttributes {
	return LiftFromRustBuffer[*DestinationAttributes](c, rb)
}

func (_ FfiConverterOptionalDestinationAttributes) Read(reader io.Reader) *DestinationAttributes {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterDestinationAttributesINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalDestinationAttributes) Lower(value *DestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[*DestinationAttributes](c, value)
}

func (c FfiConverterOptionalDestinationAttributes) LowerExternal(value *DestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*DestinationAttributes](c, value))
}

func (_ FfiConverterOptionalDestinationAttributes) Write(writer io.Writer, value *DestinationAttributes) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterDestinationAttributesINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalDestinationAttributes struct{}

func (_ FfiDestroyerOptionalDestinationAttributes) Destroy(value *DestinationAttributes) {
	if value != nil {
		FfiDestroyerDestinationAttributes{}.Destroy(*value)
	}
}

type FfiConverterOptionalFilterLanguage struct{}

var FfiConverterOptionalFilterLanguageINSTANCE = FfiConverterOptionalFilterLanguage{}

func (c FfiConverterOptionalFilterLanguage) Lift(rb RustBufferI) *FilterLanguage {
	return LiftFromRustBuffer[*FilterLanguage](c, rb)
}

func (_ FfiConverterOptionalFilterLanguage) Read(reader io.Reader) *FilterLanguage {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterFilterLanguageINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalFilterLanguage) Lower(value *FilterLanguage) C.RustBuffer {
	return LowerIntoRustBuffer[*FilterLanguage](c, value)
}

func (c FfiConverterOptionalFilterLanguage) LowerExternal(value *FilterLanguage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*FilterLanguage](c, value))
}

func (_ FfiConverterOptionalFilterLanguage) Write(writer io.Writer, value *FilterLanguage) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterFilterLanguageINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalFilterLanguage struct{}

func (_ FfiDestroyerOptionalFilterLanguage) Destroy(value *FilterLanguage) {
	if value != nil {
		FfiDestroyerFilterLanguage{}.Destroy(*value)
	}
}

type FfiConverterOptionalProductType struct{}

var FfiConverterOptionalProductTypeINSTANCE = FfiConverterOptionalProductType{}

func (c FfiConverterOptionalProductType) Lift(rb RustBufferI) *ProductType {
	return LiftFromRustBuffer[*ProductType](c, rb)
}

func (_ FfiConverterOptionalProductType) Read(reader io.Reader) *ProductType {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterProductTypeINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalProductType) Lower(value *ProductType) C.RustBuffer {
	return LowerIntoRustBuffer[*ProductType](c, value)
}

func (c FfiConverterOptionalProductType) LowerExternal(value *ProductType) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*ProductType](c, value))
}

func (_ FfiConverterOptionalProductType) Write(writer io.Writer, value *ProductType) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterProductTypeINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalProductType struct{}

func (_ FfiDestroyerOptionalProductType) Destroy(value *ProductType) {
	if value != nil {
		FfiDestroyerProductType{}.Destroy(*value)
	}
}

type FfiConverterOptionalStreamDataset struct{}

var FfiConverterOptionalStreamDatasetINSTANCE = FfiConverterOptionalStreamDataset{}

func (c FfiConverterOptionalStreamDataset) Lift(rb RustBufferI) *StreamDataset {
	return LiftFromRustBuffer[*StreamDataset](c, rb)
}

func (_ FfiConverterOptionalStreamDataset) Read(reader io.Reader) *StreamDataset {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStreamDatasetINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalStreamDataset) Lower(value *StreamDataset) C.RustBuffer {
	return LowerIntoRustBuffer[*StreamDataset](c, value)
}

func (c FfiConverterOptionalStreamDataset) LowerExternal(value *StreamDataset) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*StreamDataset](c, value))
}

func (_ FfiConverterOptionalStreamDataset) Write(writer io.Writer, value *StreamDataset) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStreamDatasetINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalStreamDataset struct{}

func (_ FfiDestroyerOptionalStreamDataset) Destroy(value *StreamDataset) {
	if value != nil {
		FfiDestroyerStreamDataset{}.Destroy(*value)
	}
}

type FfiConverterOptionalStreamMetadataLocation struct{}

var FfiConverterOptionalStreamMetadataLocationINSTANCE = FfiConverterOptionalStreamMetadataLocation{}

func (c FfiConverterOptionalStreamMetadataLocation) Lift(rb RustBufferI) *StreamMetadataLocation {
	return LiftFromRustBuffer[*StreamMetadataLocation](c, rb)
}

func (_ FfiConverterOptionalStreamMetadataLocation) Read(reader io.Reader) *StreamMetadataLocation {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStreamMetadataLocationINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalStreamMetadataLocation) Lower(value *StreamMetadataLocation) C.RustBuffer {
	return LowerIntoRustBuffer[*StreamMetadataLocation](c, value)
}

func (c FfiConverterOptionalStreamMetadataLocation) LowerExternal(value *StreamMetadataLocation) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*StreamMetadataLocation](c, value))
}

func (_ FfiConverterOptionalStreamMetadataLocation) Write(writer io.Writer, value *StreamMetadataLocation) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStreamMetadataLocationINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalStreamMetadataLocation struct{}

func (_ FfiDestroyerOptionalStreamMetadataLocation) Destroy(value *StreamMetadataLocation) {
	if value != nil {
		FfiDestroyerStreamMetadataLocation{}.Destroy(*value)
	}
}

type FfiConverterOptionalStreamRegion struct{}

var FfiConverterOptionalStreamRegionINSTANCE = FfiConverterOptionalStreamRegion{}

func (c FfiConverterOptionalStreamRegion) Lift(rb RustBufferI) *StreamRegion {
	return LiftFromRustBuffer[*StreamRegion](c, rb)
}

func (_ FfiConverterOptionalStreamRegion) Read(reader io.Reader) *StreamRegion {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStreamRegionINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalStreamRegion) Lower(value *StreamRegion) C.RustBuffer {
	return LowerIntoRustBuffer[*StreamRegion](c, value)
}

func (c FfiConverterOptionalStreamRegion) LowerExternal(value *StreamRegion) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*StreamRegion](c, value))
}

func (_ FfiConverterOptionalStreamRegion) Write(writer io.Writer, value *StreamRegion) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStreamRegionINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalStreamRegion struct{}

func (_ FfiDestroyerOptionalStreamRegion) Destroy(value *StreamRegion) {
	if value != nil {
		FfiDestroyerStreamRegion{}.Destroy(*value)
	}
}

type FfiConverterOptionalStreamStatus struct{}

var FfiConverterOptionalStreamStatusINSTANCE = FfiConverterOptionalStreamStatus{}

func (c FfiConverterOptionalStreamStatus) Lift(rb RustBufferI) *StreamStatus {
	return LiftFromRustBuffer[*StreamStatus](c, rb)
}

func (_ FfiConverterOptionalStreamStatus) Read(reader io.Reader) *StreamStatus {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStreamStatusINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalStreamStatus) Lower(value *StreamStatus) C.RustBuffer {
	return LowerIntoRustBuffer[*StreamStatus](c, value)
}

func (c FfiConverterOptionalStreamStatus) LowerExternal(value *StreamStatus) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*StreamStatus](c, value))
}

func (_ FfiConverterOptionalStreamStatus) Write(writer io.Writer, value *StreamStatus) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStreamStatusINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalStreamStatus struct{}

func (_ FfiDestroyerOptionalStreamStatus) Destroy(value *StreamStatus) {
	if value != nil {
		FfiDestroyerStreamStatus{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceInt32 struct{}

var FfiConverterOptionalSequenceInt32INSTANCE = FfiConverterOptionalSequenceInt32{}

func (c FfiConverterOptionalSequenceInt32) Lift(rb RustBufferI) *[]int32 {
	return LiftFromRustBuffer[*[]int32](c, rb)
}

func (_ FfiConverterOptionalSequenceInt32) Read(reader io.Reader) *[]int32 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceInt32INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceInt32) Lower(value *[]int32) C.RustBuffer {
	return LowerIntoRustBuffer[*[]int32](c, value)
}

func (c FfiConverterOptionalSequenceInt32) LowerExternal(value *[]int32) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]int32](c, value))
}

func (_ FfiConverterOptionalSequenceInt32) Write(writer io.Writer, value *[]int32) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceInt32INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceInt32 struct{}

func (_ FfiDestroyerOptionalSequenceInt32) Destroy(value *[]int32) {
	if value != nil {
		FfiDestroyerSequenceInt32{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceString struct{}

var FfiConverterOptionalSequenceStringINSTANCE = FfiConverterOptionalSequenceString{}

func (c FfiConverterOptionalSequenceString) Lift(rb RustBufferI) *[]string {
	return LiftFromRustBuffer[*[]string](c, rb)
}

func (_ FfiConverterOptionalSequenceString) Read(reader io.Reader) *[]string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceString) Lower(value *[]string) C.RustBuffer {
	return LowerIntoRustBuffer[*[]string](c, value)
}

func (c FfiConverterOptionalSequenceString) LowerExternal(value *[]string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]string](c, value))
}

func (_ FfiConverterOptionalSequenceString) Write(writer io.Writer, value *[]string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceString struct{}

func (_ FfiDestroyerOptionalSequenceString) Destroy(value *[]string) {
	if value != nil {
		FfiDestroyerSequenceString{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointDomainMask struct{}

var FfiConverterOptionalSequenceEndpointDomainMaskINSTANCE = FfiConverterOptionalSequenceEndpointDomainMask{}

func (c FfiConverterOptionalSequenceEndpointDomainMask) Lift(rb RustBufferI) *[]EndpointDomainMask {
	return LiftFromRustBuffer[*[]EndpointDomainMask](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointDomainMask) Read(reader io.Reader) *[]EndpointDomainMask {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointDomainMaskINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointDomainMask) Lower(value *[]EndpointDomainMask) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointDomainMask](c, value)
}

func (c FfiConverterOptionalSequenceEndpointDomainMask) LowerExternal(value *[]EndpointDomainMask) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointDomainMask](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointDomainMask) Write(writer io.Writer, value *[]EndpointDomainMask) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointDomainMaskINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointDomainMask struct{}

func (_ FfiDestroyerOptionalSequenceEndpointDomainMask) Destroy(value *[]EndpointDomainMask) {
	if value != nil {
		FfiDestroyerSequenceEndpointDomainMask{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointIp struct{}

var FfiConverterOptionalSequenceEndpointIpINSTANCE = FfiConverterOptionalSequenceEndpointIp{}

func (c FfiConverterOptionalSequenceEndpointIp) Lift(rb RustBufferI) *[]EndpointIp {
	return LiftFromRustBuffer[*[]EndpointIp](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointIp) Read(reader io.Reader) *[]EndpointIp {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointIpINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointIp) Lower(value *[]EndpointIp) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointIp](c, value)
}

func (c FfiConverterOptionalSequenceEndpointIp) LowerExternal(value *[]EndpointIp) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointIp](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointIp) Write(writer io.Writer, value *[]EndpointIp) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointIpINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointIp struct{}

func (_ FfiDestroyerOptionalSequenceEndpointIp) Destroy(value *[]EndpointIp) {
	if value != nil {
		FfiDestroyerSequenceEndpointIp{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointJwt struct{}

var FfiConverterOptionalSequenceEndpointJwtINSTANCE = FfiConverterOptionalSequenceEndpointJwt{}

func (c FfiConverterOptionalSequenceEndpointJwt) Lift(rb RustBufferI) *[]EndpointJwt {
	return LiftFromRustBuffer[*[]EndpointJwt](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointJwt) Read(reader io.Reader) *[]EndpointJwt {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointJwtINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointJwt) Lower(value *[]EndpointJwt) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointJwt](c, value)
}

func (c FfiConverterOptionalSequenceEndpointJwt) LowerExternal(value *[]EndpointJwt) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointJwt](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointJwt) Write(writer io.Writer, value *[]EndpointJwt) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointJwtINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointJwt struct{}

func (_ FfiDestroyerOptionalSequenceEndpointJwt) Destroy(value *[]EndpointJwt) {
	if value != nil {
		FfiDestroyerSequenceEndpointJwt{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointReferrer struct{}

var FfiConverterOptionalSequenceEndpointReferrerINSTANCE = FfiConverterOptionalSequenceEndpointReferrer{}

func (c FfiConverterOptionalSequenceEndpointReferrer) Lift(rb RustBufferI) *[]EndpointReferrer {
	return LiftFromRustBuffer[*[]EndpointReferrer](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointReferrer) Read(reader io.Reader) *[]EndpointReferrer {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointReferrerINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointReferrer) Lower(value *[]EndpointReferrer) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointReferrer](c, value)
}

func (c FfiConverterOptionalSequenceEndpointReferrer) LowerExternal(value *[]EndpointReferrer) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointReferrer](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointReferrer) Write(writer io.Writer, value *[]EndpointReferrer) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointReferrerINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointReferrer struct{}

func (_ FfiDestroyerOptionalSequenceEndpointReferrer) Destroy(value *[]EndpointReferrer) {
	if value != nil {
		FfiDestroyerSequenceEndpointReferrer{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointRequestFilter struct{}

var FfiConverterOptionalSequenceEndpointRequestFilterINSTANCE = FfiConverterOptionalSequenceEndpointRequestFilter{}

func (c FfiConverterOptionalSequenceEndpointRequestFilter) Lift(rb RustBufferI) *[]EndpointRequestFilter {
	return LiftFromRustBuffer[*[]EndpointRequestFilter](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointRequestFilter) Read(reader io.Reader) *[]EndpointRequestFilter {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointRequestFilterINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointRequestFilter) Lower(value *[]EndpointRequestFilter) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointRequestFilter](c, value)
}

func (c FfiConverterOptionalSequenceEndpointRequestFilter) LowerExternal(value *[]EndpointRequestFilter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointRequestFilter](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointRequestFilter) Write(writer io.Writer, value *[]EndpointRequestFilter) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointRequestFilterINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointRequestFilter struct{}

func (_ FfiDestroyerOptionalSequenceEndpointRequestFilter) Destroy(value *[]EndpointRequestFilter) {
	if value != nil {
		FfiDestroyerSequenceEndpointRequestFilter{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceEndpointToken struct{}

var FfiConverterOptionalSequenceEndpointTokenINSTANCE = FfiConverterOptionalSequenceEndpointToken{}

func (c FfiConverterOptionalSequenceEndpointToken) Lift(rb RustBufferI) *[]EndpointToken {
	return LiftFromRustBuffer[*[]EndpointToken](c, rb)
}

func (_ FfiConverterOptionalSequenceEndpointToken) Read(reader io.Reader) *[]EndpointToken {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceEndpointTokenINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceEndpointToken) Lower(value *[]EndpointToken) C.RustBuffer {
	return LowerIntoRustBuffer[*[]EndpointToken](c, value)
}

func (c FfiConverterOptionalSequenceEndpointToken) LowerExternal(value *[]EndpointToken) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]EndpointToken](c, value))
}

func (_ FfiConverterOptionalSequenceEndpointToken) Write(writer io.Writer, value *[]EndpointToken) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceEndpointTokenINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceEndpointToken struct{}

func (_ FfiDestroyerOptionalSequenceEndpointToken) Destroy(value *[]EndpointToken) {
	if value != nil {
		FfiDestroyerSequenceEndpointToken{}.Destroy(*value)
	}
}

type FfiConverterOptionalSequenceDestinationAttributes struct{}

var FfiConverterOptionalSequenceDestinationAttributesINSTANCE = FfiConverterOptionalSequenceDestinationAttributes{}

func (c FfiConverterOptionalSequenceDestinationAttributes) Lift(rb RustBufferI) *[]DestinationAttributes {
	return LiftFromRustBuffer[*[]DestinationAttributes](c, rb)
}

func (_ FfiConverterOptionalSequenceDestinationAttributes) Read(reader io.Reader) *[]DestinationAttributes {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSequenceDestinationAttributesINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSequenceDestinationAttributes) Lower(value *[]DestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[*[]DestinationAttributes](c, value)
}

func (c FfiConverterOptionalSequenceDestinationAttributes) LowerExternal(value *[]DestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*[]DestinationAttributes](c, value))
}

func (_ FfiConverterOptionalSequenceDestinationAttributes) Write(writer io.Writer, value *[]DestinationAttributes) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSequenceDestinationAttributesINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSequenceDestinationAttributes struct{}

func (_ FfiDestroyerOptionalSequenceDestinationAttributes) Destroy(value *[]DestinationAttributes) {
	if value != nil {
		FfiDestroyerSequenceDestinationAttributes{}.Destroy(*value)
	}
}

type FfiConverterOptionalMapStringString struct{}

var FfiConverterOptionalMapStringStringINSTANCE = FfiConverterOptionalMapStringString{}

func (c FfiConverterOptionalMapStringString) Lift(rb RustBufferI) *map[string]string {
	return LiftFromRustBuffer[*map[string]string](c, rb)
}

func (_ FfiConverterOptionalMapStringString) Read(reader io.Reader) *map[string]string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterMapStringStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalMapStringString) Lower(value *map[string]string) C.RustBuffer {
	return LowerIntoRustBuffer[*map[string]string](c, value)
}

func (c FfiConverterOptionalMapStringString) LowerExternal(value *map[string]string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*map[string]string](c, value))
}

func (_ FfiConverterOptionalMapStringString) Write(writer io.Writer, value *map[string]string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterMapStringStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalMapStringString struct{}

func (_ FfiDestroyerOptionalMapStringString) Destroy(value *map[string]string) {
	if value != nil {
		FfiDestroyerMapStringString{}.Destroy(*value)
	}
}

type FfiConverterOptionalMapStringEndpointUrl struct{}

var FfiConverterOptionalMapStringEndpointUrlINSTANCE = FfiConverterOptionalMapStringEndpointUrl{}

func (c FfiConverterOptionalMapStringEndpointUrl) Lift(rb RustBufferI) *map[string]EndpointUrl {
	return LiftFromRustBuffer[*map[string]EndpointUrl](c, rb)
}

func (_ FfiConverterOptionalMapStringEndpointUrl) Read(reader io.Reader) *map[string]EndpointUrl {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterMapStringEndpointUrlINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalMapStringEndpointUrl) Lower(value *map[string]EndpointUrl) C.RustBuffer {
	return LowerIntoRustBuffer[*map[string]EndpointUrl](c, value)
}

func (c FfiConverterOptionalMapStringEndpointUrl) LowerExternal(value *map[string]EndpointUrl) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*map[string]EndpointUrl](c, value))
}

func (_ FfiConverterOptionalMapStringEndpointUrl) Write(writer io.Writer, value *map[string]EndpointUrl) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterMapStringEndpointUrlINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalMapStringEndpointUrl struct{}

func (_ FfiDestroyerOptionalMapStringEndpointUrl) Destroy(value *map[string]EndpointUrl) {
	if value != nil {
		FfiDestroyerMapStringEndpointUrl{}.Destroy(*value)
	}
}

type FfiConverterSequenceInt32 struct{}

var FfiConverterSequenceInt32INSTANCE = FfiConverterSequenceInt32{}

func (c FfiConverterSequenceInt32) Lift(rb RustBufferI) []int32 {
	return LiftFromRustBuffer[[]int32](c, rb)
}

func (c FfiConverterSequenceInt32) Read(reader io.Reader) []int32 {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]int32, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterInt32INSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceInt32) Lower(value []int32) C.RustBuffer {
	return LowerIntoRustBuffer[[]int32](c, value)
}

func (c FfiConverterSequenceInt32) LowerExternal(value []int32) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]int32](c, value))
}

func (c FfiConverterSequenceInt32) Write(writer io.Writer, value []int32) {
	if len(value) > math.MaxInt32 {
		panic("[]int32 is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterInt32INSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceInt32 struct{}

func (FfiDestroyerSequenceInt32) Destroy(sequence []int32) {
	for _, value := range sequence {
		FfiDestroyerInt32{}.Destroy(value)
	}
}

type FfiConverterSequenceInt64 struct{}

var FfiConverterSequenceInt64INSTANCE = FfiConverterSequenceInt64{}

func (c FfiConverterSequenceInt64) Lift(rb RustBufferI) []int64 {
	return LiftFromRustBuffer[[]int64](c, rb)
}

func (c FfiConverterSequenceInt64) Read(reader io.Reader) []int64 {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]int64, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterInt64INSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceInt64) Lower(value []int64) C.RustBuffer {
	return LowerIntoRustBuffer[[]int64](c, value)
}

func (c FfiConverterSequenceInt64) LowerExternal(value []int64) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]int64](c, value))
}

func (c FfiConverterSequenceInt64) Write(writer io.Writer, value []int64) {
	if len(value) > math.MaxInt32 {
		panic("[]int64 is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterInt64INSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceInt64 struct{}

func (FfiDestroyerSequenceInt64) Destroy(sequence []int64) {
	for _, value := range sequence {
		FfiDestroyerInt64{}.Destroy(value)
	}
}

type FfiConverterSequenceString struct{}

var FfiConverterSequenceStringINSTANCE = FfiConverterSequenceString{}

func (c FfiConverterSequenceString) Lift(rb RustBufferI) []string {
	return LiftFromRustBuffer[[]string](c, rb)
}

func (c FfiConverterSequenceString) Read(reader io.Reader) []string {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]string, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterStringINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceString) Lower(value []string) C.RustBuffer {
	return LowerIntoRustBuffer[[]string](c, value)
}

func (c FfiConverterSequenceString) LowerExternal(value []string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]string](c, value))
}

func (c FfiConverterSequenceString) Write(writer io.Writer, value []string) {
	if len(value) > math.MaxInt32 {
		panic("[]string is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterStringINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceString struct{}

func (FfiDestroyerSequenceString) Destroy(sequence []string) {
	for _, value := range sequence {
		FfiDestroyerString{}.Destroy(value)
	}
}

type FfiConverterSequenceAccountTag struct{}

var FfiConverterSequenceAccountTagINSTANCE = FfiConverterSequenceAccountTag{}

func (c FfiConverterSequenceAccountTag) Lift(rb RustBufferI) []AccountTag {
	return LiftFromRustBuffer[[]AccountTag](c, rb)
}

func (c FfiConverterSequenceAccountTag) Read(reader io.Reader) []AccountTag {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]AccountTag, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterAccountTagINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceAccountTag) Lower(value []AccountTag) C.RustBuffer {
	return LowerIntoRustBuffer[[]AccountTag](c, value)
}

func (c FfiConverterSequenceAccountTag) LowerExternal(value []AccountTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]AccountTag](c, value))
}

func (c FfiConverterSequenceAccountTag) Write(writer io.Writer, value []AccountTag) {
	if len(value) > math.MaxInt32 {
		panic("[]AccountTag is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterAccountTagINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceAccountTag struct{}

func (FfiDestroyerSequenceAccountTag) Destroy(sequence []AccountTag) {
	for _, value := range sequence {
		FfiDestroyerAccountTag{}.Destroy(value)
	}
}

type FfiConverterSequenceBulkOperationResult struct{}

var FfiConverterSequenceBulkOperationResultINSTANCE = FfiConverterSequenceBulkOperationResult{}

func (c FfiConverterSequenceBulkOperationResult) Lift(rb RustBufferI) []BulkOperationResult {
	return LiftFromRustBuffer[[]BulkOperationResult](c, rb)
}

func (c FfiConverterSequenceBulkOperationResult) Read(reader io.Reader) []BulkOperationResult {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]BulkOperationResult, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterBulkOperationResultINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceBulkOperationResult) Lower(value []BulkOperationResult) C.RustBuffer {
	return LowerIntoRustBuffer[[]BulkOperationResult](c, value)
}

func (c FfiConverterSequenceBulkOperationResult) LowerExternal(value []BulkOperationResult) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]BulkOperationResult](c, value))
}

func (c FfiConverterSequenceBulkOperationResult) Write(writer io.Writer, value []BulkOperationResult) {
	if len(value) > math.MaxInt32 {
		panic("[]BulkOperationResult is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterBulkOperationResultINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceBulkOperationResult struct{}

func (FfiDestroyerSequenceBulkOperationResult) Destroy(sequence []BulkOperationResult) {
	for _, value := range sequence {
		FfiDestroyerBulkOperationResult{}.Destroy(value)
	}
}

type FfiConverterSequenceChain struct{}

var FfiConverterSequenceChainINSTANCE = FfiConverterSequenceChain{}

func (c FfiConverterSequenceChain) Lift(rb RustBufferI) []Chain {
	return LiftFromRustBuffer[[]Chain](c, rb)
}

func (c FfiConverterSequenceChain) Read(reader io.Reader) []Chain {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Chain, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterChainINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceChain) Lower(value []Chain) C.RustBuffer {
	return LowerIntoRustBuffer[[]Chain](c, value)
}

func (c FfiConverterSequenceChain) LowerExternal(value []Chain) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Chain](c, value))
}

func (c FfiConverterSequenceChain) Write(writer io.Writer, value []Chain) {
	if len(value) > math.MaxInt32 {
		panic("[]Chain is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterChainINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceChain struct{}

func (FfiDestroyerSequenceChain) Destroy(sequence []Chain) {
	for _, value := range sequence {
		FfiDestroyerChain{}.Destroy(value)
	}
}

type FfiConverterSequenceChainNetwork struct{}

var FfiConverterSequenceChainNetworkINSTANCE = FfiConverterSequenceChainNetwork{}

func (c FfiConverterSequenceChainNetwork) Lift(rb RustBufferI) []ChainNetwork {
	return LiftFromRustBuffer[[]ChainNetwork](c, rb)
}

func (c FfiConverterSequenceChainNetwork) Read(reader io.Reader) []ChainNetwork {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ChainNetwork, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterChainNetworkINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceChainNetwork) Lower(value []ChainNetwork) C.RustBuffer {
	return LowerIntoRustBuffer[[]ChainNetwork](c, value)
}

func (c FfiConverterSequenceChainNetwork) LowerExternal(value []ChainNetwork) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]ChainNetwork](c, value))
}

func (c FfiConverterSequenceChainNetwork) Write(writer io.Writer, value []ChainNetwork) {
	if len(value) > math.MaxInt32 {
		panic("[]ChainNetwork is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterChainNetworkINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceChainNetwork struct{}

func (FfiDestroyerSequenceChainNetwork) Destroy(sequence []ChainNetwork) {
	for _, value := range sequence {
		FfiDestroyerChainNetwork{}.Destroy(value)
	}
}

type FfiConverterSequenceChainUsage struct{}

var FfiConverterSequenceChainUsageINSTANCE = FfiConverterSequenceChainUsage{}

func (c FfiConverterSequenceChainUsage) Lift(rb RustBufferI) []ChainUsage {
	return LiftFromRustBuffer[[]ChainUsage](c, rb)
}

func (c FfiConverterSequenceChainUsage) Read(reader io.Reader) []ChainUsage {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ChainUsage, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterChainUsageINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceChainUsage) Lower(value []ChainUsage) C.RustBuffer {
	return LowerIntoRustBuffer[[]ChainUsage](c, value)
}

func (c FfiConverterSequenceChainUsage) LowerExternal(value []ChainUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]ChainUsage](c, value))
}

func (c FfiConverterSequenceChainUsage) Write(writer io.Writer, value []ChainUsage) {
	if len(value) > math.MaxInt32 {
		panic("[]ChainUsage is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterChainUsageINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceChainUsage struct{}

func (FfiDestroyerSequenceChainUsage) Destroy(sequence []ChainUsage) {
	for _, value := range sequence {
		FfiDestroyerChainUsage{}.Destroy(value)
	}
}

type FfiConverterSequenceColumnMeta struct{}

var FfiConverterSequenceColumnMetaINSTANCE = FfiConverterSequenceColumnMeta{}

func (c FfiConverterSequenceColumnMeta) Lift(rb RustBufferI) []ColumnMeta {
	return LiftFromRustBuffer[[]ColumnMeta](c, rb)
}

func (c FfiConverterSequenceColumnMeta) Read(reader io.Reader) []ColumnMeta {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ColumnMeta, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterColumnMetaINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceColumnMeta) Lower(value []ColumnMeta) C.RustBuffer {
	return LowerIntoRustBuffer[[]ColumnMeta](c, value)
}

func (c FfiConverterSequenceColumnMeta) LowerExternal(value []ColumnMeta) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]ColumnMeta](c, value))
}

func (c FfiConverterSequenceColumnMeta) Write(writer io.Writer, value []ColumnMeta) {
	if len(value) > math.MaxInt32 {
		panic("[]ColumnMeta is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterColumnMetaINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceColumnMeta struct{}

func (FfiDestroyerSequenceColumnMeta) Destroy(sequence []ColumnMeta) {
	for _, value := range sequence {
		FfiDestroyerColumnMeta{}.Destroy(value)
	}
}

type FfiConverterSequenceColumnSchema struct{}

var FfiConverterSequenceColumnSchemaINSTANCE = FfiConverterSequenceColumnSchema{}

func (c FfiConverterSequenceColumnSchema) Lift(rb RustBufferI) []ColumnSchema {
	return LiftFromRustBuffer[[]ColumnSchema](c, rb)
}

func (c FfiConverterSequenceColumnSchema) Read(reader io.Reader) []ColumnSchema {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]ColumnSchema, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterColumnSchemaINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceColumnSchema) Lower(value []ColumnSchema) C.RustBuffer {
	return LowerIntoRustBuffer[[]ColumnSchema](c, value)
}

func (c FfiConverterSequenceColumnSchema) LowerExternal(value []ColumnSchema) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]ColumnSchema](c, value))
}

func (c FfiConverterSequenceColumnSchema) Write(writer io.Writer, value []ColumnSchema) {
	if len(value) > math.MaxInt32 {
		panic("[]ColumnSchema is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterColumnSchemaINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceColumnSchema struct{}

func (FfiDestroyerSequenceColumnSchema) Destroy(sequence []ColumnSchema) {
	for _, value := range sequence {
		FfiDestroyerColumnSchema{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpoint struct{}

var FfiConverterSequenceEndpointINSTANCE = FfiConverterSequenceEndpoint{}

func (c FfiConverterSequenceEndpoint) Lift(rb RustBufferI) []Endpoint {
	return LiftFromRustBuffer[[]Endpoint](c, rb)
}

func (c FfiConverterSequenceEndpoint) Read(reader io.Reader) []Endpoint {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Endpoint, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpoint) Lower(value []Endpoint) C.RustBuffer {
	return LowerIntoRustBuffer[[]Endpoint](c, value)
}

func (c FfiConverterSequenceEndpoint) LowerExternal(value []Endpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Endpoint](c, value))
}

func (c FfiConverterSequenceEndpoint) Write(writer io.Writer, value []Endpoint) {
	if len(value) > math.MaxInt32 {
		panic("[]Endpoint is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpoint struct{}

func (FfiDestroyerSequenceEndpoint) Destroy(sequence []Endpoint) {
	for _, value := range sequence {
		FfiDestroyerEndpoint{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointDomainMask struct{}

var FfiConverterSequenceEndpointDomainMaskINSTANCE = FfiConverterSequenceEndpointDomainMask{}

func (c FfiConverterSequenceEndpointDomainMask) Lift(rb RustBufferI) []EndpointDomainMask {
	return LiftFromRustBuffer[[]EndpointDomainMask](c, rb)
}

func (c FfiConverterSequenceEndpointDomainMask) Read(reader io.Reader) []EndpointDomainMask {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointDomainMask, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointDomainMaskINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointDomainMask) Lower(value []EndpointDomainMask) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointDomainMask](c, value)
}

func (c FfiConverterSequenceEndpointDomainMask) LowerExternal(value []EndpointDomainMask) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointDomainMask](c, value))
}

func (c FfiConverterSequenceEndpointDomainMask) Write(writer io.Writer, value []EndpointDomainMask) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointDomainMask is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointDomainMaskINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointDomainMask struct{}

func (FfiDestroyerSequenceEndpointDomainMask) Destroy(sequence []EndpointDomainMask) {
	for _, value := range sequence {
		FfiDestroyerEndpointDomainMask{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointIp struct{}

var FfiConverterSequenceEndpointIpINSTANCE = FfiConverterSequenceEndpointIp{}

func (c FfiConverterSequenceEndpointIp) Lift(rb RustBufferI) []EndpointIp {
	return LiftFromRustBuffer[[]EndpointIp](c, rb)
}

func (c FfiConverterSequenceEndpointIp) Read(reader io.Reader) []EndpointIp {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointIp, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointIpINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointIp) Lower(value []EndpointIp) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointIp](c, value)
}

func (c FfiConverterSequenceEndpointIp) LowerExternal(value []EndpointIp) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointIp](c, value))
}

func (c FfiConverterSequenceEndpointIp) Write(writer io.Writer, value []EndpointIp) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointIp is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointIpINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointIp struct{}

func (FfiDestroyerSequenceEndpointIp) Destroy(sequence []EndpointIp) {
	for _, value := range sequence {
		FfiDestroyerEndpointIp{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointJwt struct{}

var FfiConverterSequenceEndpointJwtINSTANCE = FfiConverterSequenceEndpointJwt{}

func (c FfiConverterSequenceEndpointJwt) Lift(rb RustBufferI) []EndpointJwt {
	return LiftFromRustBuffer[[]EndpointJwt](c, rb)
}

func (c FfiConverterSequenceEndpointJwt) Read(reader io.Reader) []EndpointJwt {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointJwt, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointJwtINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointJwt) Lower(value []EndpointJwt) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointJwt](c, value)
}

func (c FfiConverterSequenceEndpointJwt) LowerExternal(value []EndpointJwt) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointJwt](c, value))
}

func (c FfiConverterSequenceEndpointJwt) Write(writer io.Writer, value []EndpointJwt) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointJwt is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointJwtINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointJwt struct{}

func (FfiDestroyerSequenceEndpointJwt) Destroy(sequence []EndpointJwt) {
	for _, value := range sequence {
		FfiDestroyerEndpointJwt{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointLog struct{}

var FfiConverterSequenceEndpointLogINSTANCE = FfiConverterSequenceEndpointLog{}

func (c FfiConverterSequenceEndpointLog) Lift(rb RustBufferI) []EndpointLog {
	return LiftFromRustBuffer[[]EndpointLog](c, rb)
}

func (c FfiConverterSequenceEndpointLog) Read(reader io.Reader) []EndpointLog {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointLog, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointLogINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointLog) Lower(value []EndpointLog) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointLog](c, value)
}

func (c FfiConverterSequenceEndpointLog) LowerExternal(value []EndpointLog) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointLog](c, value))
}

func (c FfiConverterSequenceEndpointLog) Write(writer io.Writer, value []EndpointLog) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointLog is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointLogINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointLog struct{}

func (FfiDestroyerSequenceEndpointLog) Destroy(sequence []EndpointLog) {
	for _, value := range sequence {
		FfiDestroyerEndpointLog{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointMetric struct{}

var FfiConverterSequenceEndpointMetricINSTANCE = FfiConverterSequenceEndpointMetric{}

func (c FfiConverterSequenceEndpointMetric) Lift(rb RustBufferI) []EndpointMetric {
	return LiftFromRustBuffer[[]EndpointMetric](c, rb)
}

func (c FfiConverterSequenceEndpointMetric) Read(reader io.Reader) []EndpointMetric {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointMetric, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointMetricINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointMetric) Lower(value []EndpointMetric) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointMetric](c, value)
}

func (c FfiConverterSequenceEndpointMetric) LowerExternal(value []EndpointMetric) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointMetric](c, value))
}

func (c FfiConverterSequenceEndpointMetric) Write(writer io.Writer, value []EndpointMetric) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointMetric is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointMetricINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointMetric struct{}

func (FfiDestroyerSequenceEndpointMetric) Destroy(sequence []EndpointMetric) {
	for _, value := range sequence {
		FfiDestroyerEndpointMetric{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointReferrer struct{}

var FfiConverterSequenceEndpointReferrerINSTANCE = FfiConverterSequenceEndpointReferrer{}

func (c FfiConverterSequenceEndpointReferrer) Lift(rb RustBufferI) []EndpointReferrer {
	return LiftFromRustBuffer[[]EndpointReferrer](c, rb)
}

func (c FfiConverterSequenceEndpointReferrer) Read(reader io.Reader) []EndpointReferrer {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointReferrer, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointReferrerINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointReferrer) Lower(value []EndpointReferrer) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointReferrer](c, value)
}

func (c FfiConverterSequenceEndpointReferrer) LowerExternal(value []EndpointReferrer) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointReferrer](c, value))
}

func (c FfiConverterSequenceEndpointReferrer) Write(writer io.Writer, value []EndpointReferrer) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointReferrer is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointReferrerINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointReferrer struct{}

func (FfiDestroyerSequenceEndpointReferrer) Destroy(sequence []EndpointReferrer) {
	for _, value := range sequence {
		FfiDestroyerEndpointReferrer{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointRequestFilter struct{}

var FfiConverterSequenceEndpointRequestFilterINSTANCE = FfiConverterSequenceEndpointRequestFilter{}

func (c FfiConverterSequenceEndpointRequestFilter) Lift(rb RustBufferI) []EndpointRequestFilter {
	return LiftFromRustBuffer[[]EndpointRequestFilter](c, rb)
}

func (c FfiConverterSequenceEndpointRequestFilter) Read(reader io.Reader) []EndpointRequestFilter {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointRequestFilter, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointRequestFilterINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointRequestFilter) Lower(value []EndpointRequestFilter) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointRequestFilter](c, value)
}

func (c FfiConverterSequenceEndpointRequestFilter) LowerExternal(value []EndpointRequestFilter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointRequestFilter](c, value))
}

func (c FfiConverterSequenceEndpointRequestFilter) Write(writer io.Writer, value []EndpointRequestFilter) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointRequestFilter is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointRequestFilterINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointRequestFilter struct{}

func (FfiDestroyerSequenceEndpointRequestFilter) Destroy(sequence []EndpointRequestFilter) {
	for _, value := range sequence {
		FfiDestroyerEndpointRequestFilter{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointTag struct{}

var FfiConverterSequenceEndpointTagINSTANCE = FfiConverterSequenceEndpointTag{}

func (c FfiConverterSequenceEndpointTag) Lift(rb RustBufferI) []EndpointTag {
	return LiftFromRustBuffer[[]EndpointTag](c, rb)
}

func (c FfiConverterSequenceEndpointTag) Read(reader io.Reader) []EndpointTag {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointTag, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointTagINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointTag) Lower(value []EndpointTag) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointTag](c, value)
}

func (c FfiConverterSequenceEndpointTag) LowerExternal(value []EndpointTag) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointTag](c, value))
}

func (c FfiConverterSequenceEndpointTag) Write(writer io.Writer, value []EndpointTag) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointTag is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointTagINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointTag struct{}

func (FfiDestroyerSequenceEndpointTag) Destroy(sequence []EndpointTag) {
	for _, value := range sequence {
		FfiDestroyerEndpointTag{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointToken struct{}

var FfiConverterSequenceEndpointTokenINSTANCE = FfiConverterSequenceEndpointToken{}

func (c FfiConverterSequenceEndpointToken) Lift(rb RustBufferI) []EndpointToken {
	return LiftFromRustBuffer[[]EndpointToken](c, rb)
}

func (c FfiConverterSequenceEndpointToken) Read(reader io.Reader) []EndpointToken {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointToken, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointTokenINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointToken) Lower(value []EndpointToken) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointToken](c, value)
}

func (c FfiConverterSequenceEndpointToken) LowerExternal(value []EndpointToken) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointToken](c, value))
}

func (c FfiConverterSequenceEndpointToken) Write(writer io.Writer, value []EndpointToken) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointToken is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointTokenINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointToken struct{}

func (FfiDestroyerSequenceEndpointToken) Destroy(sequence []EndpointToken) {
	for _, value := range sequence {
		FfiDestroyerEndpointToken{}.Destroy(value)
	}
}

type FfiConverterSequenceEndpointUsage struct{}

var FfiConverterSequenceEndpointUsageINSTANCE = FfiConverterSequenceEndpointUsage{}

func (c FfiConverterSequenceEndpointUsage) Lift(rb RustBufferI) []EndpointUsage {
	return LiftFromRustBuffer[[]EndpointUsage](c, rb)
}

func (c FfiConverterSequenceEndpointUsage) Read(reader io.Reader) []EndpointUsage {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]EndpointUsage, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterEndpointUsageINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceEndpointUsage) Lower(value []EndpointUsage) C.RustBuffer {
	return LowerIntoRustBuffer[[]EndpointUsage](c, value)
}

func (c FfiConverterSequenceEndpointUsage) LowerExternal(value []EndpointUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]EndpointUsage](c, value))
}

func (c FfiConverterSequenceEndpointUsage) Write(writer io.Writer, value []EndpointUsage) {
	if len(value) > math.MaxInt32 {
		panic("[]EndpointUsage is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterEndpointUsageINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceEndpointUsage struct{}

func (FfiDestroyerSequenceEndpointUsage) Destroy(sequence []EndpointUsage) {
	for _, value := range sequence {
		FfiDestroyerEndpointUsage{}.Destroy(value)
	}
}

type FfiConverterSequenceInvoice struct{}

var FfiConverterSequenceInvoiceINSTANCE = FfiConverterSequenceInvoice{}

func (c FfiConverterSequenceInvoice) Lift(rb RustBufferI) []Invoice {
	return LiftFromRustBuffer[[]Invoice](c, rb)
}

func (c FfiConverterSequenceInvoice) Read(reader io.Reader) []Invoice {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Invoice, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterInvoiceINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceInvoice) Lower(value []Invoice) C.RustBuffer {
	return LowerIntoRustBuffer[[]Invoice](c, value)
}

func (c FfiConverterSequenceInvoice) LowerExternal(value []Invoice) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Invoice](c, value))
}

func (c FfiConverterSequenceInvoice) Write(writer io.Writer, value []Invoice) {
	if len(value) > math.MaxInt32 {
		panic("[]Invoice is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterInvoiceINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceInvoice struct{}

func (FfiDestroyerSequenceInvoice) Destroy(sequence []Invoice) {
	for _, value := range sequence {
		FfiDestroyerInvoice{}.Destroy(value)
	}
}

type FfiConverterSequenceInvoiceLine struct{}

var FfiConverterSequenceInvoiceLineINSTANCE = FfiConverterSequenceInvoiceLine{}

func (c FfiConverterSequenceInvoiceLine) Lift(rb RustBufferI) []InvoiceLine {
	return LiftFromRustBuffer[[]InvoiceLine](c, rb)
}

func (c FfiConverterSequenceInvoiceLine) Read(reader io.Reader) []InvoiceLine {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]InvoiceLine, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterInvoiceLineINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceInvoiceLine) Lower(value []InvoiceLine) C.RustBuffer {
	return LowerIntoRustBuffer[[]InvoiceLine](c, value)
}

func (c FfiConverterSequenceInvoiceLine) LowerExternal(value []InvoiceLine) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]InvoiceLine](c, value))
}

func (c FfiConverterSequenceInvoiceLine) Write(writer io.Writer, value []InvoiceLine) {
	if len(value) > math.MaxInt32 {
		panic("[]InvoiceLine is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterInvoiceLineINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceInvoiceLine struct{}

func (FfiDestroyerSequenceInvoiceLine) Destroy(sequence []InvoiceLine) {
	for _, value := range sequence {
		FfiDestroyerInvoiceLine{}.Destroy(value)
	}
}

type FfiConverterSequenceKvSetEntry struct{}

var FfiConverterSequenceKvSetEntryINSTANCE = FfiConverterSequenceKvSetEntry{}

func (c FfiConverterSequenceKvSetEntry) Lift(rb RustBufferI) []KvSetEntry {
	return LiftFromRustBuffer[[]KvSetEntry](c, rb)
}

func (c FfiConverterSequenceKvSetEntry) Read(reader io.Reader) []KvSetEntry {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]KvSetEntry, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterKvSetEntryINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceKvSetEntry) Lower(value []KvSetEntry) C.RustBuffer {
	return LowerIntoRustBuffer[[]KvSetEntry](c, value)
}

func (c FfiConverterSequenceKvSetEntry) LowerExternal(value []KvSetEntry) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]KvSetEntry](c, value))
}

func (c FfiConverterSequenceKvSetEntry) Write(writer io.Writer, value []KvSetEntry) {
	if len(value) > math.MaxInt32 {
		panic("[]KvSetEntry is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterKvSetEntryINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceKvSetEntry struct{}

func (FfiDestroyerSequenceKvSetEntry) Destroy(sequence []KvSetEntry) {
	for _, value := range sequence {
		FfiDestroyerKvSetEntry{}.Destroy(value)
	}
}

type FfiConverterSequenceMethodRateLimiter struct{}

var FfiConverterSequenceMethodRateLimiterINSTANCE = FfiConverterSequenceMethodRateLimiter{}

func (c FfiConverterSequenceMethodRateLimiter) Lift(rb RustBufferI) []MethodRateLimiter {
	return LiftFromRustBuffer[[]MethodRateLimiter](c, rb)
}

func (c FfiConverterSequenceMethodRateLimiter) Read(reader io.Reader) []MethodRateLimiter {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]MethodRateLimiter, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterMethodRateLimiterINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceMethodRateLimiter) Lower(value []MethodRateLimiter) C.RustBuffer {
	return LowerIntoRustBuffer[[]MethodRateLimiter](c, value)
}

func (c FfiConverterSequenceMethodRateLimiter) LowerExternal(value []MethodRateLimiter) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]MethodRateLimiter](c, value))
}

func (c FfiConverterSequenceMethodRateLimiter) Write(writer io.Writer, value []MethodRateLimiter) {
	if len(value) > math.MaxInt32 {
		panic("[]MethodRateLimiter is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterMethodRateLimiterINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceMethodRateLimiter struct{}

func (FfiDestroyerSequenceMethodRateLimiter) Destroy(sequence []MethodRateLimiter) {
	for _, value := range sequence {
		FfiDestroyerMethodRateLimiter{}.Destroy(value)
	}
}

type FfiConverterSequenceMethodUsage struct{}

var FfiConverterSequenceMethodUsageINSTANCE = FfiConverterSequenceMethodUsage{}

func (c FfiConverterSequenceMethodUsage) Lift(rb RustBufferI) []MethodUsage {
	return LiftFromRustBuffer[[]MethodUsage](c, rb)
}

func (c FfiConverterSequenceMethodUsage) Read(reader io.Reader) []MethodUsage {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]MethodUsage, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterMethodUsageINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceMethodUsage) Lower(value []MethodUsage) C.RustBuffer {
	return LowerIntoRustBuffer[[]MethodUsage](c, value)
}

func (c FfiConverterSequenceMethodUsage) LowerExternal(value []MethodUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]MethodUsage](c, value))
}

func (c FfiConverterSequenceMethodUsage) Write(writer io.Writer, value []MethodUsage) {
	if len(value) > math.MaxInt32 {
		panic("[]MethodUsage is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterMethodUsageINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceMethodUsage struct{}

func (FfiDestroyerSequenceMethodUsage) Destroy(sequence []MethodUsage) {
	for _, value := range sequence {
		FfiDestroyerMethodUsage{}.Destroy(value)
	}
}

type FfiConverterSequencePayment struct{}

var FfiConverterSequencePaymentINSTANCE = FfiConverterSequencePayment{}

func (c FfiConverterSequencePayment) Lift(rb RustBufferI) []Payment {
	return LiftFromRustBuffer[[]Payment](c, rb)
}

func (c FfiConverterSequencePayment) Read(reader io.Reader) []Payment {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Payment, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterPaymentINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequencePayment) Lower(value []Payment) C.RustBuffer {
	return LowerIntoRustBuffer[[]Payment](c, value)
}

func (c FfiConverterSequencePayment) LowerExternal(value []Payment) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Payment](c, value))
}

func (c FfiConverterSequencePayment) Write(writer io.Writer, value []Payment) {
	if len(value) > math.MaxInt32 {
		panic("[]Payment is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterPaymentINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequencePayment struct{}

func (FfiDestroyerSequencePayment) Destroy(sequence []Payment) {
	for _, value := range sequence {
		FfiDestroyerPayment{}.Destroy(value)
	}
}

type FfiConverterSequenceRateLimitEntry struct{}

var FfiConverterSequenceRateLimitEntryINSTANCE = FfiConverterSequenceRateLimitEntry{}

func (c FfiConverterSequenceRateLimitEntry) Lift(rb RustBufferI) []RateLimitEntry {
	return LiftFromRustBuffer[[]RateLimitEntry](c, rb)
}

func (c FfiConverterSequenceRateLimitEntry) Read(reader io.Reader) []RateLimitEntry {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]RateLimitEntry, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterRateLimitEntryINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceRateLimitEntry) Lower(value []RateLimitEntry) C.RustBuffer {
	return LowerIntoRustBuffer[[]RateLimitEntry](c, value)
}

func (c FfiConverterSequenceRateLimitEntry) LowerExternal(value []RateLimitEntry) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]RateLimitEntry](c, value))
}

func (c FfiConverterSequenceRateLimitEntry) Write(writer io.Writer, value []RateLimitEntry) {
	if len(value) > math.MaxInt32 {
		panic("[]RateLimitEntry is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterRateLimitEntryINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceRateLimitEntry struct{}

func (FfiDestroyerSequenceRateLimitEntry) Destroy(sequence []RateLimitEntry) {
	for _, value := range sequence {
		FfiDestroyerRateLimitEntry{}.Destroy(value)
	}
}

type FfiConverterSequenceSecurityOption struct{}

var FfiConverterSequenceSecurityOptionINSTANCE = FfiConverterSequenceSecurityOption{}

func (c FfiConverterSequenceSecurityOption) Lift(rb RustBufferI) []SecurityOption {
	return LiftFromRustBuffer[[]SecurityOption](c, rb)
}

func (c FfiConverterSequenceSecurityOption) Read(reader io.Reader) []SecurityOption {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]SecurityOption, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterSecurityOptionINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceSecurityOption) Lower(value []SecurityOption) C.RustBuffer {
	return LowerIntoRustBuffer[[]SecurityOption](c, value)
}

func (c FfiConverterSequenceSecurityOption) LowerExternal(value []SecurityOption) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]SecurityOption](c, value))
}

func (c FfiConverterSequenceSecurityOption) Write(writer io.Writer, value []SecurityOption) {
	if len(value) > math.MaxInt32 {
		panic("[]SecurityOption is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterSecurityOptionINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceSecurityOption struct{}

func (FfiDestroyerSequenceSecurityOption) Destroy(sequence []SecurityOption) {
	for _, value := range sequence {
		FfiDestroyerSecurityOption{}.Destroy(value)
	}
}

type FfiConverterSequenceStream struct{}

var FfiConverterSequenceStreamINSTANCE = FfiConverterSequenceStream{}

func (c FfiConverterSequenceStream) Lift(rb RustBufferI) []Stream {
	return LiftFromRustBuffer[[]Stream](c, rb)
}

func (c FfiConverterSequenceStream) Read(reader io.Reader) []Stream {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Stream, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterStreamINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceStream) Lower(value []Stream) C.RustBuffer {
	return LowerIntoRustBuffer[[]Stream](c, value)
}

func (c FfiConverterSequenceStream) LowerExternal(value []Stream) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Stream](c, value))
}

func (c FfiConverterSequenceStream) Write(writer io.Writer, value []Stream) {
	if len(value) > math.MaxInt32 {
		panic("[]Stream is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterStreamINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceStream struct{}

func (FfiDestroyerSequenceStream) Destroy(sequence []Stream) {
	for _, value := range sequence {
		FfiDestroyerStream{}.Destroy(value)
	}
}

type FfiConverterSequenceTableSchema struct{}

var FfiConverterSequenceTableSchemaINSTANCE = FfiConverterSequenceTableSchema{}

func (c FfiConverterSequenceTableSchema) Lift(rb RustBufferI) []TableSchema {
	return LiftFromRustBuffer[[]TableSchema](c, rb)
}

func (c FfiConverterSequenceTableSchema) Read(reader io.Reader) []TableSchema {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TableSchema, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTableSchemaINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTableSchema) Lower(value []TableSchema) C.RustBuffer {
	return LowerIntoRustBuffer[[]TableSchema](c, value)
}

func (c FfiConverterSequenceTableSchema) LowerExternal(value []TableSchema) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TableSchema](c, value))
}

func (c FfiConverterSequenceTableSchema) Write(writer io.Writer, value []TableSchema) {
	if len(value) > math.MaxInt32 {
		panic("[]TableSchema is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTableSchemaINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTableSchema struct{}

func (FfiDestroyerSequenceTableSchema) Destroy(sequence []TableSchema) {
	for _, value := range sequence {
		FfiDestroyerTableSchema{}.Destroy(value)
	}
}

type FfiConverterSequenceTagUsage struct{}

var FfiConverterSequenceTagUsageINSTANCE = FfiConverterSequenceTagUsage{}

func (c FfiConverterSequenceTagUsage) Lift(rb RustBufferI) []TagUsage {
	return LiftFromRustBuffer[[]TagUsage](c, rb)
}

func (c FfiConverterSequenceTagUsage) Read(reader io.Reader) []TagUsage {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TagUsage, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTagUsageINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTagUsage) Lower(value []TagUsage) C.RustBuffer {
	return LowerIntoRustBuffer[[]TagUsage](c, value)
}

func (c FfiConverterSequenceTagUsage) LowerExternal(value []TagUsage) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TagUsage](c, value))
}

func (c FfiConverterSequenceTagUsage) Write(writer io.Writer, value []TagUsage) {
	if len(value) > math.MaxInt32 {
		panic("[]TagUsage is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTagUsageINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTagUsage struct{}

func (FfiDestroyerSequenceTagUsage) Destroy(sequence []TagUsage) {
	for _, value := range sequence {
		FfiDestroyerTagUsage{}.Destroy(value)
	}
}

type FfiConverterSequenceTeamEndpoint struct{}

var FfiConverterSequenceTeamEndpointINSTANCE = FfiConverterSequenceTeamEndpoint{}

func (c FfiConverterSequenceTeamEndpoint) Lift(rb RustBufferI) []TeamEndpoint {
	return LiftFromRustBuffer[[]TeamEndpoint](c, rb)
}

func (c FfiConverterSequenceTeamEndpoint) Read(reader io.Reader) []TeamEndpoint {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TeamEndpoint, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTeamEndpointINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTeamEndpoint) Lower(value []TeamEndpoint) C.RustBuffer {
	return LowerIntoRustBuffer[[]TeamEndpoint](c, value)
}

func (c FfiConverterSequenceTeamEndpoint) LowerExternal(value []TeamEndpoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TeamEndpoint](c, value))
}

func (c FfiConverterSequenceTeamEndpoint) Write(writer io.Writer, value []TeamEndpoint) {
	if len(value) > math.MaxInt32 {
		panic("[]TeamEndpoint is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTeamEndpointINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTeamEndpoint struct{}

func (FfiDestroyerSequenceTeamEndpoint) Destroy(sequence []TeamEndpoint) {
	for _, value := range sequence {
		FfiDestroyerTeamEndpoint{}.Destroy(value)
	}
}

type FfiConverterSequenceTeamSummary struct{}

var FfiConverterSequenceTeamSummaryINSTANCE = FfiConverterSequenceTeamSummary{}

func (c FfiConverterSequenceTeamSummary) Lift(rb RustBufferI) []TeamSummary {
	return LiftFromRustBuffer[[]TeamSummary](c, rb)
}

func (c FfiConverterSequenceTeamSummary) Read(reader io.Reader) []TeamSummary {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TeamSummary, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTeamSummaryINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTeamSummary) Lower(value []TeamSummary) C.RustBuffer {
	return LowerIntoRustBuffer[[]TeamSummary](c, value)
}

func (c FfiConverterSequenceTeamSummary) LowerExternal(value []TeamSummary) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TeamSummary](c, value))
}

func (c FfiConverterSequenceTeamSummary) Write(writer io.Writer, value []TeamSummary) {
	if len(value) > math.MaxInt32 {
		panic("[]TeamSummary is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTeamSummaryINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTeamSummary struct{}

func (FfiDestroyerSequenceTeamSummary) Destroy(sequence []TeamSummary) {
	for _, value := range sequence {
		FfiDestroyerTeamSummary{}.Destroy(value)
	}
}

type FfiConverterSequenceTeamUser struct{}

var FfiConverterSequenceTeamUserINSTANCE = FfiConverterSequenceTeamUser{}

func (c FfiConverterSequenceTeamUser) Lift(rb RustBufferI) []TeamUser {
	return LiftFromRustBuffer[[]TeamUser](c, rb)
}

func (c FfiConverterSequenceTeamUser) Read(reader io.Reader) []TeamUser {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TeamUser, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTeamUserINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTeamUser) Lower(value []TeamUser) C.RustBuffer {
	return LowerIntoRustBuffer[[]TeamUser](c, value)
}

func (c FfiConverterSequenceTeamUser) LowerExternal(value []TeamUser) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TeamUser](c, value))
}

func (c FfiConverterSequenceTeamUser) Write(writer io.Writer, value []TeamUser) {
	if len(value) > math.MaxInt32 {
		panic("[]TeamUser is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTeamUserINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTeamUser struct{}

func (FfiDestroyerSequenceTeamUser) Destroy(sequence []TeamUser) {
	for _, value := range sequence {
		FfiDestroyerTeamUser{}.Destroy(value)
	}
}

type FfiConverterSequenceWebhook struct{}

var FfiConverterSequenceWebhookINSTANCE = FfiConverterSequenceWebhook{}

func (c FfiConverterSequenceWebhook) Lift(rb RustBufferI) []Webhook {
	return LiftFromRustBuffer[[]Webhook](c, rb)
}

func (c FfiConverterSequenceWebhook) Read(reader io.Reader) []Webhook {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Webhook, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterWebhookINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceWebhook) Lower(value []Webhook) C.RustBuffer {
	return LowerIntoRustBuffer[[]Webhook](c, value)
}

func (c FfiConverterSequenceWebhook) LowerExternal(value []Webhook) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Webhook](c, value))
}

func (c FfiConverterSequenceWebhook) Write(writer io.Writer, value []Webhook) {
	if len(value) > math.MaxInt32 {
		panic("[]Webhook is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterWebhookINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceWebhook struct{}

func (FfiDestroyerSequenceWebhook) Destroy(sequence []Webhook) {
	for _, value := range sequence {
		FfiDestroyerWebhook{}.Destroy(value)
	}
}

type FfiConverterSequenceDestinationAttributes struct{}

var FfiConverterSequenceDestinationAttributesINSTANCE = FfiConverterSequenceDestinationAttributes{}

func (c FfiConverterSequenceDestinationAttributes) Lift(rb RustBufferI) []DestinationAttributes {
	return LiftFromRustBuffer[[]DestinationAttributes](c, rb)
}

func (c FfiConverterSequenceDestinationAttributes) Read(reader io.Reader) []DestinationAttributes {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]DestinationAttributes, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterDestinationAttributesINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceDestinationAttributes) Lower(value []DestinationAttributes) C.RustBuffer {
	return LowerIntoRustBuffer[[]DestinationAttributes](c, value)
}

func (c FfiConverterSequenceDestinationAttributes) LowerExternal(value []DestinationAttributes) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]DestinationAttributes](c, value))
}

func (c FfiConverterSequenceDestinationAttributes) Write(writer io.Writer, value []DestinationAttributes) {
	if len(value) > math.MaxInt32 {
		panic("[]DestinationAttributes is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterDestinationAttributesINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceDestinationAttributes struct{}

func (FfiDestroyerSequenceDestinationAttributes) Destroy(sequence []DestinationAttributes) {
	for _, value := range sequence {
		FfiDestroyerDestinationAttributes{}.Destroy(value)
	}
}

type FfiConverterSequenceSequenceInt64 struct{}

var FfiConverterSequenceSequenceInt64INSTANCE = FfiConverterSequenceSequenceInt64{}

func (c FfiConverterSequenceSequenceInt64) Lift(rb RustBufferI) [][]int64 {
	return LiftFromRustBuffer[[][]int64](c, rb)
}

func (c FfiConverterSequenceSequenceInt64) Read(reader io.Reader) [][]int64 {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([][]int64, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterSequenceInt64INSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceSequenceInt64) Lower(value [][]int64) C.RustBuffer {
	return LowerIntoRustBuffer[[][]int64](c, value)
}

func (c FfiConverterSequenceSequenceInt64) LowerExternal(value [][]int64) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[][]int64](c, value))
}

func (c FfiConverterSequenceSequenceInt64) Write(writer io.Writer, value [][]int64) {
	if len(value) > math.MaxInt32 {
		panic("[][]int64 is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterSequenceInt64INSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceSequenceInt64 struct{}

func (FfiDestroyerSequenceSequenceInt64) Destroy(sequence [][]int64) {
	for _, value := range sequence {
		FfiDestroyerSequenceInt64{}.Destroy(value)
	}
}

type FfiConverterSequenceTypeJsonValue struct{}

var FfiConverterSequenceTypeJsonValueINSTANCE = FfiConverterSequenceTypeJsonValue{}

func (c FfiConverterSequenceTypeJsonValue) Lift(rb RustBufferI) []JsonValue {
	return LiftFromRustBuffer[[]JsonValue](c, rb)
}

func (c FfiConverterSequenceTypeJsonValue) Read(reader io.Reader) []JsonValue {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]JsonValue, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTypeJsonValueINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTypeJsonValue) Lower(value []JsonValue) C.RustBuffer {
	return LowerIntoRustBuffer[[]JsonValue](c, value)
}

func (c FfiConverterSequenceTypeJsonValue) LowerExternal(value []JsonValue) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]JsonValue](c, value))
}

func (c FfiConverterSequenceTypeJsonValue) Write(writer io.Writer, value []JsonValue) {
	if len(value) > math.MaxInt32 {
		panic("[]JsonValue is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTypeJsonValueINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTypeJsonValue struct{}

func (FfiDestroyerSequenceTypeJsonValue) Destroy(sequence []JsonValue) {
	for _, value := range sequence {
		FfiDestroyerTypeJsonValue{}.Destroy(value)
	}
}

type FfiConverterMapStringString struct{}

var FfiConverterMapStringStringINSTANCE = FfiConverterMapStringString{}

func (c FfiConverterMapStringString) Lift(rb RustBufferI) map[string]string {
	return LiftFromRustBuffer[map[string]string](c, rb)
}

func (_ FfiConverterMapStringString) Read(reader io.Reader) map[string]string {
	result := make(map[string]string)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterStringINSTANCE.Read(reader)
		value := FfiConverterStringINSTANCE.Read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapStringString) Lower(value map[string]string) C.RustBuffer {
	return LowerIntoRustBuffer[map[string]string](c, value)
}

func (c FfiConverterMapStringString) LowerExternal(value map[string]string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[map[string]string](c, value))
}

func (_ FfiConverterMapStringString) Write(writer io.Writer, mapValue map[string]string) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]string is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterStringINSTANCE.Write(writer, key)
		FfiConverterStringINSTANCE.Write(writer, value)
	}
}

type FfiDestroyerMapStringString struct{}

func (_ FfiDestroyerMapStringString) Destroy(mapValue map[string]string) {
	for key, value := range mapValue {
		FfiDestroyerString{}.Destroy(key)
		FfiDestroyerString{}.Destroy(value)
	}
}

type FfiConverterMapStringEndpointUrl struct{}

var FfiConverterMapStringEndpointUrlINSTANCE = FfiConverterMapStringEndpointUrl{}

func (c FfiConverterMapStringEndpointUrl) Lift(rb RustBufferI) map[string]EndpointUrl {
	return LiftFromRustBuffer[map[string]EndpointUrl](c, rb)
}

func (_ FfiConverterMapStringEndpointUrl) Read(reader io.Reader) map[string]EndpointUrl {
	result := make(map[string]EndpointUrl)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterStringINSTANCE.Read(reader)
		value := FfiConverterEndpointUrlINSTANCE.Read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapStringEndpointUrl) Lower(value map[string]EndpointUrl) C.RustBuffer {
	return LowerIntoRustBuffer[map[string]EndpointUrl](c, value)
}

func (c FfiConverterMapStringEndpointUrl) LowerExternal(value map[string]EndpointUrl) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[map[string]EndpointUrl](c, value))
}

func (_ FfiConverterMapStringEndpointUrl) Write(writer io.Writer, mapValue map[string]EndpointUrl) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]EndpointUrl is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterStringINSTANCE.Write(writer, key)
		FfiConverterEndpointUrlINSTANCE.Write(writer, value)
	}
}

type FfiDestroyerMapStringEndpointUrl struct{}

func (_ FfiDestroyerMapStringEndpointUrl) Destroy(mapValue map[string]EndpointUrl) {
	for key, value := range mapValue {
		FfiDestroyerString{}.Destroy(key)
		FfiDestroyerEndpointUrl{}.Destroy(value)
	}
}

/**
 * Typealias from the type name used in the UDL file to the builtin type.  This
 * is needed because the UDL type name is used in function/method signatures.
 * It's also what we have an external type that references a custom type.
 */
type JsonValue = string
type FfiConverterTypeJsonValue = FfiConverterString
type FfiDestroyerTypeJsonValue = FfiDestroyerString

var FfiConverterTypeJsonValueINSTANCE = FfiConverterString{}

func LiftFromExternalTypeJsonValue(value ExternalCRustBuffer) JsonValue {
	return FfiConverterTypeJsonValueINSTANCE.Lift(RustBufferFromExternal(value))
}

func LowerToExternalTypeJsonValue(value JsonValue) ExternalCRustBuffer {
	return RustBufferFromC(FfiConverterTypeJsonValueINSTANCE.Lower(value))
}
