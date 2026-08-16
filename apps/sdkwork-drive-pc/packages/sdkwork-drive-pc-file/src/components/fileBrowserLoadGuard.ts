export interface LatestRequestGuard {
  begin(scope?: string): number;
  setCurrentScope(scope: string): void;
  isCurrent(sequence: number, scope?: string): boolean;
  isCurrentScope(scope: string): boolean;
}

export function createLatestRequestGuard(): LatestRequestGuard {
  let currentSequence = 0;
  let currentScope: string | undefined;

  return {
    begin(scope) {
      currentSequence += 1;
      if (scope !== undefined) {
        currentScope = scope;
      }
      return currentSequence;
    },
    setCurrentScope(scope) {
      currentScope = scope;
    },
    isCurrent(sequence, scope) {
      return sequence === currentSequence && (scope === undefined || scope === currentScope);
    },
    isCurrentScope(scope) {
      return scope === currentScope;
    },
  };
}
