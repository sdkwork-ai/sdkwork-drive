import { describe, expect, it } from "vitest";
import { runBatchSettledOperations } from "./fileBrowserBatchUtils";

describe("runBatchSettledOperations", () => {
  it("counts succeeded and failed operations", async () => {
    const outcome = await runBatchSettledOperations([
      async () => "ok",
      async () => {
        throw new Error("failed");
      },
      async () => "also-ok",
    ]);

    expect(outcome.succeededCount).toBe(2);
    expect(outcome.failedCount).toBe(1);
    expect(outcome.firstFailure?.status).toBe("rejected");
  });
});
