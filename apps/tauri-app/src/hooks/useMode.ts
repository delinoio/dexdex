// React Query hooks for mode management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getMode, setMode } from "@/api/client";

// Query keys
export const modeKeys = {
  all: ["mode"] as const,
  current: () => [...modeKeys.all, "current"] as const,
};

// Query hooks

export function useMode() {
  return useQuery({
    queryKey: modeKeys.current(),
    queryFn: () => getMode(),
  });
}

// Mutation hooks

export function useSetMode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      mode,
      serverUrl,
    }: {
      mode: "local" | "remote";
      serverUrl?: string;
    }) => setMode(mode, serverUrl),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: modeKeys.current() });
    },
  });
}
