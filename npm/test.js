const sdk = require("./sdk.js");

async function main() {
  sdk.init("test-key");
  const uuid = await sdk.httpbin.getUuid();
  console.log("UUID:", uuid);
}

main();
