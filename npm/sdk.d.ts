// sdk.d.ts
import { QuickNodeSdk as _QuickNodeSdk, SdkFullConfig } from "./index";

export class QuickNodeSdk {
  constructor(config: SdkFullConfig);
  static fromEnv(): QuickNodeSdk;
  admin: _QuickNodeSdk["admin"];
}
