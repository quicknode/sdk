import {
  DestinationAttributes,
  QuickNodeSdk,
  StreamDataset,
  StreamMetadataLocation,
  StreamRegion,
  StreamStatus,
} from "..";
import type { CreateStreamParams } from "..";

async function main() {
  const qn = QuickNodeSdk.fromEnv();
  const params: CreateStreamParams = {
    name: "My Stream",
    network: "ethereum-mainnet",
    dataset: StreamDataset.Block,
    region: StreamRegion.UsaEast,
    startRange: 24691804,
    endRange: 24691904,
    destinationAttributes: DestinationAttributes.webhook({
      url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
      maxRetry: 3,
      retryIntervalSec: 1,
      postTimeoutSec: 10,
    }),
    plan: "growth_plan",
    thresholdFetchBuffer: 1000,
    datasetBatchSize: 1,
    includeStreamMetadata: StreamMetadataLocation.Body,
    fixBlockReorgs: 0,
    keepDistanceFromTip: 0,
    elasticBatchEnabled: true,
    status: StreamStatus.Active,
  };
  const stream = await qn.streams.createStream(params);
  console.log(`${stream.id} | ${stream.name} | ${stream.status}`);
}

main();
