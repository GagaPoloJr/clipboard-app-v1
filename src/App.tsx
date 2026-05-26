import { useEffect, useCallback, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useClipboardStore, initClipboardListener } from "./stores/clipboardStore";
import SearchBar from "./components/SearchBar";
import ClipboardList from "./components/ClipboardList";
import Footer from "./components/Footer";
import SettingsPanel from "./components/SettingsPanel";
import "./App.css";

const appWindow = getCurrentWebviewWindow();

function App() {
  const { fetchHistory, searchQuery, copyToClipboard, selectedIndex, setSelectedIndex, items, isLoading } =
    useClipboardStore();
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    const unlisten = initClipboardListener();
    fetchHistory();
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    fetchHistory(searchQuery);
  }, [searchQuery]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        appWindow.hide();
        return;
      }

      const list = searchQuery
        ? items.filter(
            (item) =>
              item.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
              item.preview.toLowerCase().includes(searchQuery.toLowerCase())
          )
        : items;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex(Math.min(selectedIndex + 1, list.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex(Math.max(selectedIndex - 1, -1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < list.length) {
          copyToClipboard(list[selectedIndex].id);
        }
      }
    },
    [items, searchQuery, selectedIndex, setSelectedIndex, copyToClipboard]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const filteredItems = searchQuery
    ? items.filter(
        (item) =>
          item.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
          item.preview.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : items;

  return (
    <div className="h-full w-full flex flex-col bg-gray-900 rounded-xl overflow-hidden select-none relative border-2 border-blue-500">
      <SearchBar />
      <div className="flex-1 min-h-0">
        <ClipboardList items={filteredItems} isLoading={isLoading} />
      </div>
      <Footer itemCount={items.length} onOpenSettings={() => setShowSettings(true)} />
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
