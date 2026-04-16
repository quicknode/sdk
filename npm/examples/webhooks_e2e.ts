import { QuickNodeSdk, TemplateArgs, WebhookStartFrom } from "../sdk";
import type {
  CreateWebhookFromTemplateParams,
  UpdateWebhookParams,
} from "../sdk";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  const qn = QuickNodeSdk.fromEnv();

  const before = await qn.webhooks.listWebhooks();
  console.log(`webhooks before: ${before.data.length}`);

  const count = await qn.webhooks.getEnabledCount();
  console.log(`enabled count: ${count.total}`);

  const createParams: CreateWebhookFromTemplateParams = {
    name: "E2E Test Webhook",
    network: "ethereum-mainnet",
    destinationAttributes: {
      url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
    },
    templateArgs: TemplateArgs.evmWalletFilter({
      wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
    }),
  };

  const webhook = await qn.webhooks.createWebhookFromTemplate(createParams);
  const id = webhook.id;
  console.log(`created: ${id} | ${webhook.status}`);

  const fetched = await qn.webhooks.getWebhook(id);
  console.log(`fetched: ${fetched.id} | ${fetched.name}`);

  const updateParams: UpdateWebhookParams = { name: "E2E Test Webhook Updated" };
  const updated = await qn.webhooks.updateWebhook(id, updateParams);
  console.log(`updated name: ${updated.name}`);

  await qn.webhooks.pauseWebhook(id);
  console.log("paused");

  await qn.webhooks.activateWebhook(id, { startFrom: WebhookStartFrom.Latest });
  console.log("activated");

  await qn.webhooks.deleteWebhook(id);
  console.log(`deleted: ${id}`);
  await sleep(1000);

  const after = await qn.webhooks.listWebhooks();
  console.log(`webhooks after: ${after.data.length}`);
}

main();
