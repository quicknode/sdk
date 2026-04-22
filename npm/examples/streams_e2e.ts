import {
  QuicknodeSdk,
  StreamDataset,
  StreamMetadataLocation,
  StreamRegion,
  StreamStatus,
} from "../sdk";
import type { CreateStreamParams, UpdateStreamParams } from "../sdk";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  const qn = QuicknodeSdk.fromEnv();

  const before = await qn.streams.listStreams();
  console.log(`streams before: ${before.pageInfo.total}`);

  const count = await qn.streams.getEnabledCount();
  console.log(`enabled count: ${count.total}`);

  const filterResult = await qn.streams.testFilter({
    network: "ethereum-mainnet",
    dataset: StreamDataset.Block,
    block: "17811625",
    filterFunction: "ZnVuY3Rpb24gbWFpbihkYXRhKSB7IHJldHVybiBkYXRhOyB9",
  });
  console.log(`filter logs: ${JSON.stringify(filterResult.logs)}`);
  await sleep(1000);

  const createParams: CreateStreamParams = {
    name: "E2E Test Stream",
    network: "ethereum-mainnet",
    dataset: StreamDataset.Block,
    region: StreamRegion.UsaEast,
    startRange: 24691804,
    endRange: 24691904,
    destinationAttributes: {
      destination: "webhook",
      attributes: {
        url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
        maxRetry: 3,
        retryIntervalSec: 1,
        postTimeoutSec: 10,
        compression: "none",
      },
    },
    extraDestinations: [
      {
        destination: "webhook",
        attributes: {
          url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
          maxRetry: 3,
          retryIntervalSec: 1,
          postTimeoutSec: 10,
          compression: "none",
        },
      },
    ],
    plan: "growth_plan",
    thresholdFetchBuffer: 1000,
    datasetBatchSize: 1,
    includeStreamMetadata: StreamMetadataLocation.Body,
    fixBlockReorgs: 0,
    keepDistanceFromTip: 0,
    elasticBatchEnabled: true,
    status: StreamStatus.Active,
  };

  const stream = await qn.streams.createStream(createParams);
  const id = stream.id;
  console.log(`created: ${id} | ${stream.status}`);

  const fetched = await qn.streams.getStream(id);
  console.log(`fetched: ${fetched.id} | ${fetched.name}`);

  const updateParams: UpdateStreamParams = { name: "E2E Test Stream Updated" };
  const updated = await qn.streams.updateStream(id, updateParams);
  console.log(`updated name: ${updated.name}`);
  await sleep(1000);

  await qn.streams.pauseStream(id);
  console.log("paused");

  await qn.streams.activateStream(id);
  console.log("activated");

  await qn.streams.deleteStream(id);
  console.log(`deleted: ${id}`);
  await sleep(1000);

  const after = await qn.streams.listStreams();
  console.log(`streams after: ${after.pageInfo.total}`);
}

main();
