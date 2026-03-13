// sdk.d.ts
import { QuickNodeSdk as _QuickNodeSdk } from "./index";

export class QuickNodeSdk {
  constructor(apiKey: string);
  admin: _QuickNodeSdk["admin"];
}
