import React from 'react';

interface DriveErrorBoundaryProps {
  children: React.ReactNode;
  fallbackTitle?: string;
  fallbackDescription?: string;
  retryLabel?: string;
}

interface DriveErrorBoundaryState {
  error: Error | null;
}

export class DriveErrorBoundary extends React.Component<
  DriveErrorBoundaryProps,
  DriveErrorBoundaryState
> {
  constructor(props: DriveErrorBoundaryProps) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): DriveErrorBoundaryState {
    return { error };
  }

  override componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[sdkwork-drive-pc] render boundary caught error', error, info);
  }

  private handleRetry = () => {
    this.setState({ error: null });
  };

  override render() {
    if (this.state.error) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-background p-6">
          <div className="max-w-md rounded-xl border border-border bg-card p-6 text-center shadow-sm">
            <h1 className="text-lg font-semibold text-foreground">
              {this.props.fallbackTitle ?? 'Something went wrong'}
            </h1>
            <p className="mt-2 text-sm text-muted-foreground">
              {this.props.fallbackDescription ??
                'The application hit an unexpected error. You can retry without reloading the page.'}
            </p>
            <button
              type="button"
              className="mt-4 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground"
              onClick={this.handleRetry}
            >
              {this.props.retryLabel ?? 'Retry'}
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
