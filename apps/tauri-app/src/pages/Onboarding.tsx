import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { useAddRepository } from "@/hooks/useRepositories";

type Step = 1 | 2;

export function Onboarding() {
  const [step, setStep] = useState<Step>(1);
  const [provider, setProvider] = useState("github");
  const [token, setToken] = useState("");
  const [isValidating, setIsValidating] = useState(false);
  const [tokenValid, setTokenValid] = useState<boolean | null>(null);
  const [username, setUsername] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [repoValidated, setRepoValidated] = useState(false);
  const [repoName, setRepoName] = useState("");
  const [defaultBranch, setDefaultBranch] = useState("");
  const navigate = useNavigate();
  const addRepository = useAddRepository();

  const validateToken = async () => {
    setIsValidating(true);
    try {
      // This would actually validate the token with the VCS provider
      // For now, we'll simulate a successful validation
      await new Promise((resolve) => setTimeout(resolve, 1000));
      setTokenValid(true);
      setUsername("@user");
    } catch {
      setTokenValid(false);
    } finally {
      setIsValidating(false);
    }
  };

  const validateRepository = async () => {
    setIsValidating(true);
    try {
      // Simulate repository validation
      await new Promise((resolve) => setTimeout(resolve, 500));
      const name = repoUrl.split("/").pop()?.replace(".git", "") || "repo";
      setRepoName(name);
      setDefaultBranch("main");
      setRepoValidated(true);
    } catch {
      setRepoValidated(false);
    } finally {
      setIsValidating(false);
    }
  };

  const handleSkip = () => {
    if (step === 1) {
      setStep(2);
    } else {
      navigate("/");
    }
  };

  const handleNext = () => {
    setStep(2);
  };

  const handleGetStarted = async () => {
    try {
      await addRepository.mutateAsync({
        remoteUrl: repoUrl,
        name: repoName,
        defaultBranch: defaultBranch,
      });
      navigate("/");
    } catch (error) {
      console.error("Failed to add repository:", error);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-8">
      <div className="w-full max-w-lg space-y-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold">Welcome to DeliDev</h1>
          <p className="mt-2 text-[hsl(var(--muted-foreground))]">
            {step === 1 ? "Connect your VCS Provider" : "Add Your First Repository"}
          </p>
          <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
            Step {step} of 2
          </p>
        </div>

        {step === 1 && (
          <Card>
            <CardHeader>
              <CardTitle>VCS Provider</CardTitle>
              <CardDescription>
                Select a provider and enter your access token.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Provider</label>
                <Select
                  value={provider}
                  onChange={(e) => setProvider(e.target.value)}
                >
                  <option value="github">GitHub</option>
                  <option value="gitlab">GitLab</option>
                  <option value="bitbucket">Bitbucket</option>
                </Select>
              </div>

              <div className="space-y-2">
                <label className="text-sm font-medium">
                  Personal Access Token
                </label>
                <Input
                  type="password"
                  placeholder={
                    provider === "github"
                      ? "ghp_..."
                      : provider === "gitlab"
                        ? "glpat-..."
                        : "..."
                  }
                  value={token}
                  onChange={(e) => {
                    setToken(e.target.value);
                    setTokenValid(null);
                  }}
                />
                <p className="text-xs text-[hsl(var(--muted-foreground))]">
                  Required scopes: repo, read:user, workflow
                </p>
                <a
                  href={
                    provider === "github"
                      ? "https://github.com/settings/tokens/new"
                      : provider === "gitlab"
                        ? "https://gitlab.com/-/user_settings/personal_access_tokens"
                        : "https://bitbucket.org/account/settings/app-passwords/"
                  }
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs text-[hsl(var(--primary))] hover:underline"
                >
                  Create token on {provider === "github" ? "GitHub" : provider === "gitlab" ? "GitLab" : "Bitbucket"} →
                </a>
              </div>

              {token && (
                <Button
                  variant="outline"
                  onClick={validateToken}
                  disabled={isValidating}
                  className="w-full"
                >
                  {isValidating ? "Validating..." : "Validate Token"}
                </Button>
              )}

              {tokenValid === true && (
                <div className="rounded-md bg-green-50 p-3 dark:bg-green-900/20">
                  <div className="flex items-center gap-2">
                    <svg
                      className="h-5 w-5 text-green-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-sm font-medium text-green-800 dark:text-green-200">
                      Connection successful
                    </span>
                  </div>
                  <p className="mt-1 text-sm text-green-700 dark:text-green-300">
                    Authenticated as: {username}
                  </p>
                </div>
              )}

              {tokenValid === false && (
                <div className="rounded-md bg-red-50 p-3 dark:bg-red-900/20">
                  <p className="text-sm text-red-800 dark:text-red-200">
                    Invalid token. Please check and try again.
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {step === 2 && (
          <Card>
            <CardHeader>
              <CardTitle>Repository URL</CardTitle>
              <CardDescription>
                Enter a repository URL to get started.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Repository URL</label>
                <div className="flex gap-2">
                  <Input
                    placeholder="https://github.com/user/my-app"
                    value={repoUrl}
                    onChange={(e) => {
                      setRepoUrl(e.target.value);
                      setRepoValidated(false);
                    }}
                  />
                  <Button
                    variant="outline"
                    onClick={validateRepository}
                    disabled={!repoUrl || isValidating}
                  >
                    Validate
                  </Button>
                </div>
              </div>

              {repoValidated && (
                <div className="rounded-md bg-green-50 p-3 dark:bg-green-900/20">
                  <div className="flex items-center gap-2">
                    <svg
                      className="h-5 w-5 text-green-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-sm font-medium text-green-800 dark:text-green-200">
                      Repository found
                    </span>
                  </div>
                  <div className="mt-2 space-y-1 text-sm text-green-700 dark:text-green-300">
                    <p>Name: {repoName}</p>
                    <p>Default Branch: {defaultBranch}</p>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        <div className="flex items-center justify-between border-t border-[hsl(var(--border))] pt-4">
          <Button variant="ghost" onClick={handleSkip}>
            Skip
          </Button>
          <div className="flex gap-2">
            {step === 2 && (
              <Button variant="outline" onClick={() => setStep(1)}>
                ← Back
              </Button>
            )}
            {step === 1 ? (
              <Button onClick={handleNext} disabled={!tokenValid}>
                Next →
              </Button>
            ) : (
              <Button
                onClick={handleGetStarted}
                disabled={!repoValidated || addRepository.isPending}
              >
                {addRepository.isPending ? "Adding..." : "Get Started"}
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
