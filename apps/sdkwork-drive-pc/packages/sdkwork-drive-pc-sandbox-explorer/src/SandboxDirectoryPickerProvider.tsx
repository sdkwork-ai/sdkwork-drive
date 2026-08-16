import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type { SandboxExplorerPort, SandboxSelection } from './contracts';
import { SandboxDirectoryPickerDialog } from './SandboxDirectoryPickerDialog';

export interface PickSandboxDirectoryOptions {
  readonly title?: string;
}

export interface SandboxDirectoryPickerController {
  pickDirectory(options?: PickSandboxDirectoryOptions): Promise<SandboxSelection | null>;
}

export interface SandboxDirectoryPickerProviderProps {
  readonly children: ReactNode;
  readonly port?: SandboxExplorerPort;
}

interface PendingDirectoryPickerRequest {
  readonly options: PickSandboxDirectoryOptions;
  readonly resolve: (selection: SandboxSelection | null) => void;
}

const SandboxDirectoryPickerContext = createContext<SandboxDirectoryPickerController | null>(null);

export function SandboxDirectoryPickerProvider({
  children,
  port,
}: SandboxDirectoryPickerProviderProps) {
  const pendingRequestRef = useRef<PendingDirectoryPickerRequest | null>(null);
  const [pendingRequest, setPendingRequest] = useState<PendingDirectoryPickerRequest | null>(null);

  const settleRequest = useCallback((selection: SandboxSelection | null) => {
    const request = pendingRequestRef.current;
    if (!request) return;
    pendingRequestRef.current = null;
    setPendingRequest(null);
    request.resolve(selection);
  }, []);

  const pickDirectory = useCallback((options: PickSandboxDirectoryOptions = {}) => {
    if (pendingRequestRef.current) {
      return Promise.reject(new Error('A sandbox directory picker request is already active.'));
    }

    return new Promise<SandboxSelection | null>((resolve) => {
      const request = { options, resolve };
      pendingRequestRef.current = request;
      setPendingRequest(request);
    });
  }, []);

  useEffect(() => () => {
    const request = pendingRequestRef.current;
    pendingRequestRef.current = null;
    request?.resolve(null);
  }, []);

  const controller = useMemo<SandboxDirectoryPickerController>(
    () => ({ pickDirectory }),
    [pickDirectory],
  );

  return (
    <SandboxDirectoryPickerContext.Provider value={controller}>
      {children}
      <SandboxDirectoryPickerDialog
        open={pendingRequest !== null}
        port={port}
        title={pendingRequest?.options.title}
        onCancel={() => settleRequest(null)}
        onDirectorySelected={settleRequest}
      />
    </SandboxDirectoryPickerContext.Provider>
  );
}

export function useSandboxDirectoryPicker(): SandboxDirectoryPickerController {
  const controller = useContext(SandboxDirectoryPickerContext);
  if (!controller) {
    throw new Error('useSandboxDirectoryPicker must be used within SandboxDirectoryPickerProvider.');
  }
  return controller;
}
