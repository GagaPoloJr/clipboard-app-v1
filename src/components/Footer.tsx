import { useClipboardStore } from "../stores/clipboardStore";

interface Props {
  itemCount: number;
  onOpenSettings: () => void;
}

export default function Footer({ itemCount, onOpenSettings }: Props) {
  const clearHistory = useClipboardStore((s) => s.clearHistory);

  return (
    <div className="px-3 py-2 flex items-center justify-between text-[10px] text-white/25 border-t border-white/5">
      <div className="flex items-center gap-2">
        <span>{itemCount} item{itemCount !== 1 ? "s" : ""}</span>
      </div>
      <div className="flex items-center gap-1">
        {itemCount > 0 && (
          <button
            onClick={() => {
              if (confirm("Clear all clipboard history?")) {
                clearHistory();
              }
            }}
            className="hover:text-white/50 transition-colors px-1"
          >
            Clear all
          </button>
        )}
        <button
          onClick={onOpenSettings}
          className="p-0.5 rounded hover:bg-white/10 transition-colors"
          title="Settings"
        >
          <svg className="w-3 h-3 text-white/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
