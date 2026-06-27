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
			return C.uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_get_endpoints()
		})
		if checksum != 43823 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_method_quicknodesdkclient_get_endpoints: UniFFI API checksum mismatch")
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
			return C.uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new_with_admin_base_url()
		})
		if checksum != 59851 {
			// If this happens try cleaning and rebuilding your project
			panic("quicknode_sdk: uniffi_quicknode_sdk_checksum_constructor_quicknodesdkclient_new_with_admin_base_url: UniFFI API checksum mismatch")
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

// Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and exposes its
// methods synchronously.
type QuicknodeSdkClientInterface interface {
	// List endpoints on the account. See [`GetEndpointsRequest`] for filters.
	GetEndpoints(params GetEndpointsRequest) (GetEndpointsResponse, error)
}

// Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and exposes its
// methods synchronously.
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

// Construct an SDK client overriding the admin API base URL. Primarily for
// testing against a mock server; production callers use [`Self::new`].
func QuicknodeSdkClientNewWithAdminBaseUrl(apiKey string, adminBaseUrl string) (*QuicknodeSdkClient, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_quicknode_sdk_fn_constructor_quicknodesdkclient_new_with_admin_base_url(FfiConverterStringINSTANCE.Lower(apiKey), FfiConverterStringINSTANCE.Lower(adminBaseUrl), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *QuicknodeSdkClient
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterQuicknodeSdkClientINSTANCE.Lift(_uniffiRV), nil
	}
}

// List endpoints on the account. See [`GetEndpointsRequest`] for filters.
func (_self *QuicknodeSdkClient) GetEndpoints(params GetEndpointsRequest) (GetEndpointsResponse, error) {
	_pointer := _self.ffiObject.incrementPointer("*QuicknodeSdkClient")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*QuicknodeError](FfiConverterQuicknodeError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_quicknode_sdk_fn_method_quicknodesdkclient_get_endpoints(
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
