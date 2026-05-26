export type ContentType = "text" | "image" | "file" | "rich_text";

export interface ClipboardItem {
  id: string;
  content: string;
  content_type: ContentType;
  preview: string;
  app_name?: string;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface ClipboardStore {
  items: ClipboardItem[];
  searchQuery: string;
  selectedIndex: number;
  isLoading: boolean;
  fetchHistory: () => Promise<void>;
  copyAndPaste: (id: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  setSearch: (q: string) => void;
  setSelectedIndex: (i: number) => void;
}
