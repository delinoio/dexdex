import { Component, type ReactNode, type ErrorInfo } from "react";
import { Button } from "@/components/ui/Button";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error to console for debugging
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex h-full flex-col items-center justify-center p-8">
          <div className="max-w-md text-center">
            <h2 className="mb-4 text-xl font-semibold text-[hsl(var(--destructive))]">
              Something went wrong
            </h2>
            <p className="mb-6 text-sm text-[hsl(var(--muted-foreground))]">
              An unexpected error occurred. Please try again or reload the application.
            </p>
            <div className="flex justify-center gap-2">
              <Button variant="outline" onClick={this.handleReset}>
                Try Again
              </Button>
              <Button onClick={() => window.location.reload()}>
                Reload App
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
