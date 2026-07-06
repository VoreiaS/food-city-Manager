import { Component, type ReactNode, type ErrorInfo } from "react";
import { AlertTriangle, RefreshCw, Home } from "lucide-react";
import { Link } from "react-router-dom";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, errorInfo);
    this.setState({ errorInfo });
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (!this.state.hasError) return this.props.children;

    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="card max-w-md w-full p-6 text-center">
          <AlertTriangle size={48} className="mx-auto text-amber-500" />
          <h1 className="mt-4 font-display text-xl font-bold">Something went wrong</h1>
          <p className="mt-2 text-sm text-gray-500">
            An unexpected error occurred. Our team has been notified.
          </p>

          {import.meta.env.DEV && this.state.error && (
            <details className="mt-4 text-left">
              <summary className="cursor-pointer text-xs text-gray-500">
                Error details (dev only)
              </summary>
              <pre className="mt-2 max-h-48 overflow-auto rounded bg-gray-100 p-2 text-xs text-red-600">
                {this.state.error.toString()}
                {this.state.errorInfo?.componentStack}
              </pre>
            </details>
          )}

          <div className="mt-6 flex gap-2 justify-center">
            <button
              onClick={this.handleReset}
              className="btn-primary"
            >
              <RefreshCw size={14} /> Try again
            </button>
            <Link to="/" className="btn-secondary" onClick={this.handleReset}>
              <Home size={14} /> Home
            </Link>
          </div>
        </div>
      </div>
    );
  }
}
