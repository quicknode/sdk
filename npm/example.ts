import { add, divide, getExternalUuid } from "./index";

async function main() {
  console.log("Testing my-sdk Node.js bindings (TypeScript)\n");

  // Test add
  const sum: number = add(2, 3);
  console.log("add(2, 3) =", sum);

  // Test divide
  const quotient: number = divide(10, 2);
  console.log("divide(10, 2) =", quotient);

  // Test divide by zero (should throw)
  try {
    const result: number = divide(1, 0);
    console.log("divide(1, 0) =", result);
  } catch (e) {
    console.log("divide(1, 0) threw:", (e as Error).message);
  }

  const uuid = await getExternalUuid();
  console.log(uuid);
}

main();
console.log("\nAll tests passed!");
