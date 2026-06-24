// Typed error classes. The Rust binding throws a plain napi Error whose
// message is tagged "[<kind>|<status>|<body_len>]<msg>\x1f<body>"; parseAndRethrow
// decodes that and throws an instance of the matching subclass below.

class QuicknodeError extends Error {
  constructor(message) {
    super(message);
    this.name = "QuicknodeError";
  }
}

class ConfigError extends QuicknodeError {
  constructor(message) {
    super(message);
    this.name = "ConfigError";
  }
}

class HttpError extends QuicknodeError {
  constructor(message) {
    super(message);
    this.name = "HttpError";
  }
}

class TimeoutError extends HttpError {
  constructor(message) {
    super(message);
    this.name = "TimeoutError";
  }
}

class ConnectionError extends HttpError {
  constructor(message) {
    super(message);
    this.name = "ConnectionError";
  }
}

class ApiError extends QuicknodeError {
  constructor(message, status, body) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

class DecodeError extends QuicknodeError {
  constructor(message, body) {
    super(message);
    this.name = "DecodeError";
    this.body = body;
  }
}

class RpcError extends QuicknodeError {
  constructor(message, code) {
    super(message);
    this.name = "RpcError";
    this.code = code;
  }
}

const TAG_RE = /^\[(Config|Http|Timeout|Connect|Api|Decode|Rpc)\|([^|]+)\|([^\]]+)\](.*)$/s;

function fromNapiError(err) {
  if (!(err instanceof Error)) return err;
  const m = err.message.match(TAG_RE);
  if (!m) return err;
  const [, kind, statusStr, bodyLenStr, rest] = m;
  // rest = "<msg>\x1f<body>". Use body_len (byte length from Rust) to split
  // deterministically — the body may itself contain \x1f, and Api messages
  // embed the body in msg, so scanning for the first separator is unsafe.
  let msg = rest;
  let body = "";
  if (bodyLenStr !== "-") {
    const bodyLen = Number(bodyLenStr);
    const bodyBytes = Buffer.from(rest, "utf8");
    const bodyStart = bodyBytes.length - bodyLen;
    if (bodyStart >= 1 && bodyBytes[bodyStart - 1] === 0x1f) {
      msg = bodyBytes.slice(0, bodyStart - 1).toString("utf8");
      body = bodyBytes.slice(bodyStart).toString("utf8");
    }
  }
  switch (kind) {
    case "Config": return new ConfigError(msg);
    case "Timeout": return new TimeoutError(msg);
    case "Connect": return new ConnectionError(msg);
    case "Http": return new HttpError(msg);
    case "Api": return new ApiError(msg, Number(statusStr), body);
    case "Decode": return new DecodeError(msg, body);
    // For Rpc, statusStr is the JSON-RPC code and body is its message.
    case "Rpc": return new RpcError(body || msg, Number(statusStr));
    default: return err;
  }
}

// Wraps an object's methods so thrown napi errors get retagged as typed
// subclasses. Handles both sync throws and rejected promises.
function wrapClient(client) {
  return new Proxy(client, {
    get(target, prop) {
      const val = target[prop];
      if (typeof val !== "function") return val;
      return function (...args) {
        try {
          const result = val.apply(target, args);
          if (result && typeof result.then === "function") {
            return result.catch((e) => { throw fromNapiError(e); });
          }
          return result;
        } catch (e) {
          throw fromNapiError(e);
        }
      };
    },
  });
}

module.exports = {
  QuicknodeError,
  ConfigError,
  HttpError,
  TimeoutError,
  ConnectionError,
  ApiError,
  DecodeError,
  RpcError,
  fromNapiError,
  wrapClient,
};
