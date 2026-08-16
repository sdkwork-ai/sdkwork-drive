export const MAX_PARALLEL_UPLOADS = 3;

export function getSettledBatchMessage(
  result: PromiseSettledResult<unknown>,
  fallbackMessage: string,
): string {
  if (result.status !== "rejected") {
    return "";
  }
  const reason = result.reason;
  if (reason instanceof Error && reason.message.trim()) {
    return reason.message;
  }
  return fallbackMessage;
}

export async function runWithConcurrency<T>(
  items: readonly T[],
  limit: number,
  worker: (item: T) => Promise<void>,
): Promise<void> {
  if (items.length === 0) {
    return;
  }

  let nextIndex = 0;
  const runners = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (nextIndex < items.length) {
      const currentIndex = nextIndex;
      nextIndex += 1;
      await worker(items[currentIndex]!);
    }
  });
  await Promise.all(runners);
}

export type BatchSettledOutcome = {
  succeededCount: number;
  failedCount: number;
  firstFailure?: PromiseSettledResult<unknown>;
};

export async function runBatchSettledOperations(
  operations: ReadonlyArray<() => Promise<unknown>>,
): Promise<BatchSettledOutcome> {
  const results = await Promise.allSettled(operations.map((operation) => operation()));
  const failed = results.filter((result) => result.status === "rejected");
  return {
    succeededCount: results.length - failed.length,
    failedCount: failed.length,
    firstFailure: failed[0],
  };
}
