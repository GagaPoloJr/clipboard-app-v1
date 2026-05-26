import { useState, useEffect } from "react";
import { useClipboardStore } from "../stores/clipboardStore";
import type { ClipboardItem as ClipboardItemType } from "../types";
import { CopyCheck } from "lucide-react";

interface Props {
  item: ClipboardItemType;
}

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diff = now - then;

  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return "just now";

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function highlightText(text: string, query: string) {
  if (!query) return text;

  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  const parts: { text: string; highlight: boolean }[] = [];
  let idx = 0;

  while (idx < text.length) {
    const matchIdx = lower.indexOf(q, idx);
    if (matchIdx === -1) {
      parts.push({ text: text.slice(idx), highlight: false });
      break;
    }
    if (matchIdx > idx) {
      parts.push({ text: text.slice(idx, matchIdx), highlight: false });
    }
    parts.push({ text: text.slice(matchIdx, matchIdx + q.length), highlight: true });
    idx = matchIdx + q.length;
  }

  return parts.map((part, i) =>
    part.highlight ? (
      <span key={i} className="text-yellow-200 bg-yellow-400/15 rounded-sm px-0.5">
        {part.text}
      </span>
    ) : (
      part.text
    )
  );
}

export default function ClipboardItem({ item }: Props) {
  const {
    copyToClipboard,
    deleteItem,
    togglePin,
    selectedIndex,
    items,
    searchQuery,
  } = useClipboardStore();
  const [isCopied, setIsCopied] = useState(false);

  const list = searchQuery
    ? items.filter(
        (i) =>
          i.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
          i.preview.toLowerCase().includes(searchQuery.toLowerCase()),
      )
    : items;

  const isSelected = list.findIndex((i) => i.id === item.id) === selectedIndex;

  useEffect(() => {
    if (isCopied) {
      const timer = setTimeout(() => setIsCopied(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [isCopied]);

  return (
    <div
      className={`group relative flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors ${
        isCopied
          ? "bg-green-500/30 ring-1 ring-inset ring-green-500/50"
          : isSelected
            ? "bg-white/20"
            : "hover:bg-white/10"
      }`}
      onClick={() => {
        copyToClipboard(item.id);
        setIsCopied(true);
      }}
    >
      {isCopied && (
        <div className="fixed top-2 right-4 z-50">
          <div className="bg-green-500/30 ring-1 ring-inset ring-green-500/50 px-4 py-2 rounded-lg">
            <CopyCheck className="w-4 h-4 text-white/90" />
          </div>
        </div>
      )}
      <div className="flex-1 min-w-0">
        {item.content_type === "image" ? (
          <div className="flex items-center gap-2">
            <img
              src={`data:image/png;base64,${item.content}`}
              alt={item.preview}
              className="h-7 w-12 object-cover rounded"
              loading="lazy"
            />
            <p className="text-xs text-white/60">{item.preview}</p>
          </div>
        ) : (
          <p className="text-sm text-white/90 truncate">
            {searchQuery ? highlightText(item.preview, searchQuery) : item.preview}
          </p>
        )}
        <p className="text-[10px] text-white/30 mt-0.5">
          {timeAgo(item.created_at)}
        </p>
      </div>
      <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={(e) => {
            e.stopPropagation();
            togglePin(item.id);
          }}
          className="p-1 rounded hover:bg-white/15 transition-colors"
          title={item.is_pinned ? "Unpin" : "Pin"}
        >
          <svg
            className={`w-3.5 h-3.5 ${item.is_pinned ? "text-yellow-400" : "text-white/30"}`}
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path d="M10 2a1 1 0 011 1v7h3a1 1 0 01.832 1.555L13 14v3a1 1 0 01-1 1H8a1 1 0 01-1-1v-3l-1.832-2.445A1 1 0 016 10h3V3a1 1 0 011-1z" />
          </svg>
        </button>
        <button
          onClick={(e) => {
            e.stopPropagation();
            deleteItem(item.id);
          }}
          className="p-1 rounded hover:bg-white/15 transition-colors"
          title="Delete"
        >
          <svg
            className="w-3.5 h-3.5 text-white/30 hover:text-red-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
