// Hook to handle global chat hotkey from Tauri
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useChatStore } from "@/stores/chatStore";

export function useChatHotkey() {
  const { toggleChat } = useChatStore();

  useEffect(() => {
    // Listen for toggle-chat event from Tauri backend
    const unlisten = listen("toggle-chat", () => {
      toggleChat();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [toggleChat]);
}
