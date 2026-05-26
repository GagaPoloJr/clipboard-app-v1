import { useState } from "react";
import { useClipboardStore } from "../stores/clipboardStore";

interface Props {
  onClose: () => void;
}

export default function SettingsPanel({ onClose }: Props) {
  const clearHistory = useClipboardStore((s) => s.clearHistory);
  const itemCount = useClipboardStore((s) => s.items.length);
  const [confirmClear, setConfirmClear] = useState(false);

  const handleClear = () => {
    if (!confirmClear) {
      setConfirmClear(true);
      return;
    }
    clearHistory();
    setConfirmClear(false);
  };

  return (
    <div className="absolute inset-0 z-50 flex flex-col bg-black/90 backdrop-blur-xl rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 border-b border-white/5">
        <h2 className="text-sm font-medium text-white/80">Settings</h2>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-white/10 transition-colors"
        >
          <svg className="w-4 h-4 text-white/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="flex-1 p-4 space-y-4">
        <div>
          <label className="text-[11px] text-white/40 uppercase tracking-wide">
            Max items
          </label>
          <p className="text-sm text-white/60 mt-1">500 items (auto-purges oldest unpinned)</p>
        </div>

        <div>
          <label className="text-[11px] text-white/40 uppercase tracking-wide">
            History
          </label>
          <p className="text-sm text-white/60 mt-1">
            {itemCount} item{itemCount !== 1 ? "s" : ""} stored
          </p>
          <button
            onClick={handleClear}
            className={`mt-2 px-3 py-1.5 text-xs rounded-md transition-colors ${
              confirmClear
                ? "bg-red-500/80 text-white hover:bg-red-500"
                : "bg-white/5 text-white/50 hover:bg-white/10"
            }`}
          >
            {confirmClear ? "Confirm clear all" : "Clear all unpinned"}
          </button>
        </div>

        <div>
          <label className="text-[11px] text-white/40 uppercase tracking-wide">
            Shortcut
          </label>
          <p className="text-sm text-white/60 mt-1">Cmd + Shift + V</p>
        </div>

        <div>
          <label className="text-[11px] text-white/40 uppercase tracking-wide">
            About
          </label>
          <p className="text-sm text-white/60 mt-1">Clipboard App v0.1.0</p>
          <p className="text-xs text-white/30 mt-0.5">
            A minimalist clipboard history manager for macOS
          </p>
        </div>
      </div>
    </div>
  );
}
