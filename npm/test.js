const { add, divide } = require("./index.js");

console.log("Testing my-sdk Node.js bindings\n");

console.log("add(2, 3) =", add(2, 3));
console.log("divide(10, 2) =", divide(10, 2));

try {
  console.log("divide(1, 0) =", divide(1, 0));
} catch (e) {
  console.log("divide(1, 0) threw:", e.message);
}

console.log("\nAll tests passed!");
