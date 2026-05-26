import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ClipboardItem } from "../types";

interface ClipboardStore {
  items: ClipboardItem[];
  searchQuery: string;
  selectedIndex: number;
  isLoading: boolean;

  fetchHistory: (search?: string) => Promise<void>;
  copyToClipboard: (id: string) => Promise<void>;
  copyAndPaste: (id: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  setSearch: (q: string) => void;
  setSelectedIndex: (i: number) => void;
}

export const useClipboardStore = create<ClipboardStore>((set) => ({
  items: [],
  searchQuery: "",
  selectedIndex: -1,
  isLoading: false,

  fetchHistory: async (search?: string) => {
    set({ isLoading: true });
    try {
      const items = await invoke<ClipboardItem[]>("get_history", {
        search: search || null,
      });
      set({ items });
    } catch (err) {
      console.error("Failed to fetch history:", err);
    } finally {
      set({ isLoading: false });
    }
  },

  copyToClipboard: async (id: string) => {
    try {
      await invoke("copy_to_clipboard", { id });
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  },

  copyAndPaste: async (id: string) => {
    try {
      await invoke("copy_and_paste", { id });
    } catch (err) {
      console.error("Failed to copy and paste:", err);
    }
  },

  deleteItem: async (id: string) => {
    try {
      await invoke("delete_item", { id });
      set((state) => ({
        items: state.items.filter((item) => item.id !== id),
      }));
    } catch (err) {
      console.error("Failed to delete item:", err);
    }
  },

  togglePin: async (id: string) => {
    try {
      await invoke("toggle_pin", { id });
      set((state) => ({
        items: state.items.map((item) =>
          item.id === id ? { ...item, is_pinned: !item.is_pinned } : item
        ),
      }));
    } catch (err) {
      console.error("Failed to toggle pin:", err);
    }
  },

  clearHistory: async () => {
    try {
      await invoke("clear_history");
      set({ items: [] });
    } catch (err) {
      console.error("Failed to clear history:", err);
    }
  },

  setSearch: (q: string) => {
    set({ searchQuery: q, selectedIndex: -1 });
  },

  setSelectedIndex: (i: number) => {
    set({ selectedIndex: i });
  },
}));

export function initClipboardListener() {
  const unlisten = listen<ClipboardItem>("clipboard-new-item", (event) => {
    useClipboardStore.setState((state) => ({
      items: [event.payload, ...state.items],
    }));
  });
  return unlisten;
}
