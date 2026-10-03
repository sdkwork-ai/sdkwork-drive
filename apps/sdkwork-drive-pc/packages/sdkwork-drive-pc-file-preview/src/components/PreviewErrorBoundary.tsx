import React from 'react';

export interface PreviewErrorBoundaryProps {
  children: React.ReactNode;
  /** 出错时展示的内容；由宿主决定文案，边界本身不认识任何词条。 */
  fallback: (retry: () => void) => React.ReactNode;
}

interface PreviewErrorBoundaryState {
  failed: boolean;
}

/**
 * 懒加载块（Monaco 编辑器内核，数 MB）的下载失败边界。
 *
 * `React.lazy` 的 chunk 请求可能失败（离线、代理拦截、部署后旧 chunk 被清理），此时
 * React 会把异常抛到最近的 error boundary；没有边界就会整块界面变白。这里只做一件事：
 * 把失败收敛成一个可重试的面板，重试时重新挂载子树，让 `React.lazy` 再取一次。
 */
export class PreviewErrorBoundary extends React.Component<
  PreviewErrorBoundaryProps,
  PreviewErrorBoundaryState
> {
  state: PreviewErrorBoundaryState = { failed: false };

  static getDerivedStateFromError(): PreviewErrorBoundaryState {
    return { failed: true };
  }

  private readonly retry = () => this.setState({ failed: false });

  render(): React.ReactNode {
    if (this.state.failed) {
      return this.props.fallback(this.retry);
    }
    return this.props.children;
  }
}
