// Typed error classes. The Rust binding throws a plain napi Error whose
// message is tagged "[<kind>|<status>|<body_len>]<msg>\x1f<body>"; parseAndRethrow
// decodes that and throws an instance of the matching subclass below.

class QuickNodeError extends Error {
  constructor(message) {
    super(message);
    this.name = "QuickNodeError";
  }
}

class ConfigError extends QuickNodeError {
  constructor(message) {
    super(message);
    this.name = "ConfigError";
  }
}

class HttpError extends QuickNodeError {
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

class ApiError extends QuickNodeError {
  constructor(message, status, body) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

class DecodeError extends QuickNodeError {
  constructor(message, body) {
    super(message);
    this.name = "DecodeError";
    this.body = body;
  }
}

const TAG_RE = /^\[(Config|Http|Timeout|Connect|Api|Decode)\|([^|]+)\|([^\]]+)\](.*)$/s;

function fromNapiError(err) {
  if (!(err instanceof Error)) return err;
  const m = err.message.match(TAG_RE);
  if (!m) return err;
  const [, kind, statusStr, , rest] = m;
  // rest = "<msg>\x1f<body>" — split on the unit separator
  const sepIdx = rest.indexOf("\x1f");
  const msg = sepIdx === -1 ? rest : rest.slice(0, sepIdx);
  const body = sepIdx === -1 ? "" : rest.slice(sepIdx + 1);
  switch (kind) {
    case "Config": return new ConfigError(msg);
    case "Timeout": return new TimeoutError(msg);
    case "Connect": return new ConnectionError(msg);
    case "Http": return new HttpError(msg);
    case "Api": return new ApiError(msg, Number(statusStr), body);
    case "Decode": return new DecodeError(msg, body);
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
  QuickNodeError,
  ConfigError,
  HttpError,
  TimeoutError,
  ConnectionError,
  ApiError,
  DecodeError,
  fromNapiError,
  wrapClient,
};
