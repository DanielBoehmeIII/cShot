import { useState, useEffect, useCallback, useRef } from "react";
import {
  getAudioData,
  deleteSound,
  toggleFavorite,
  exportWav,
  openExportFolder,
  type SoundEntry,
} from "../lib/api";

interface LibraryViewProps {
  onPlay: (id: string) => void;
  onStop: () => void;
  isPlaying: boolean;
  activeId: string | null;
  isAudioLoading: boolean;
  favorites: Set<string>;
  onFavoriteToggle: (id: string) => void;
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
  navigateToGenerator: () => void;
}

type FilterMode = "all" | "favorites";

export function LibraryView({
  onPlay,
  onStop,
  isPlaying,
  activeId,
  isAudioLoading,
  favorites,
  onFavoriteToggle,
  onError,
  onSuccess,
  navigateToGenerator,
}: LibraryViewProps) {
  const [sounds, setSounds] = useState<SoundEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterMode, setFilterMode] = useState<FilterMode>("all");
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [stats, setStats] = useState({ total: 0, favCount: 0 });
  const [exportState, setExportState] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const PAGE_SIZE = 25;
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const loadMoreRef = useRef<HTMLDivElement | null>(null);

  const loadSounds = useCallback(async (query: string, favoritesOnly: boolean, append = false) => {
    const isAppend = append;
    if (!isAppend) {
      setIsLoading(true);
      setOffset(0);
      setHasMore(true);
    } else {
      setIsLoadingMore(true);
    }
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const currentOffset = isAppend ? offset + PAGE_SIZE : 0;
      if (query.trim()) {
        const results = await invoke<SoundEntry[]>("search_sounds", { query });
        if (isAppend) {
          setSounds(prev => [...prev, ...results]);
        } else {
          setSounds(results);
        }
        setHasMore(false);
      } else if (favoritesOnly) {
        const results = await invoke<SoundEntry[]>("get_favorites");
        if (isAppend) {
          setSounds(prev => [...prev, ...results]);
        } else {
          setSounds(results);
        }
        setHasMore(false);
      } else {
        const results = await invoke<SoundEntry[]>("list_sounds", { limit: PAGE_SIZE, offset: currentOffset });
        if (isAppend) {
          setSounds(prev => [...prev, ...results]);
          setOffset(currentOffset);
        } else {
          setSounds(results);
          setOffset(PAGE_SIZE);
        }
        setHasMore(results.length === PAGE_SIZE);
      }
    } catch (e) {
      onError(`Failed to load library: ${e}`);
      if (!isAppend) setSounds([]);
    } finally {
      if (isAppend) {
        setIsLoadingMore(false);
      } else {
        setIsLoading(false);
      }
    }
  }, [onError, offset]);

  const handleLoadMore = useCallback(() => {
    loadSounds(searchQuery, filterMode === "favorites", true);
  }, [loadSounds, searchQuery, filterMode]);

  const refreshStats = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const [total, favCount] = await Promise.all([
        invoke<number>("count_library_sounds"),
        invoke<number>("count_favorite_sounds"),
      ]);
      setStats({ total, favCount });
    } catch {}
  }, []);

  useEffect(() => {
    refreshStats();
  }, [refreshStats]);

  useEffect(() => {
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    searchTimerRef.current = setTimeout(() => {
      loadSounds(searchQuery, filterMode === "favorites");
    }, 150);
    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, [searchQuery, filterMode, loadSounds]);

  const handlePlay = useCallback(
    async (id: string) => {
      if (isPlaying && activeId === id) {
        onStop();
        return;
      }
      if (isAudioLoading) return;
      try {
        await getAudioData(id);
        onPlay(id);
      } catch {
        onError("Could not play sound");
      }
    },
    [isPlaying, activeId, isAudioLoading, onPlay, onStop, onError],
  );

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await deleteSound(id);
        setSounds((prev) => prev.filter((s) => s.id !== id));
        setConfirmDelete(null);
        onSuccess("Sound deleted");
        refreshStats();
      } catch (e) {
        onError(`Failed to delete: ${e}`);
      }
    },
    [onSuccess, onError, refreshStats],
  );

  const handleToggleFavorite = useCallback(
    async (id: string) => {
      const sound = sounds.find((s) => s.id === id);
      if (!sound) return;
      try {
        await toggleFavorite(id, sound.prompt, sound.sound_type, sound.duration_ms);
        onFavoriteToggle(id);
        if (favorites.has(id)) {
          onSuccess("Removed from favorites");
        } else {
          onSuccess("Added to favorites");
        }
        refreshStats();
      } catch {
        onError("Failed to toggle favorite");
      }
    },
    [sounds, onFavoriteToggle, onError, onSuccess, favorites, refreshStats],
  );

  const handleExport = useCallback(
    async (id: string) => {
      try {
        setExportState(id);
        const result = await exportWav(id);
        onSuccess(`Exported: ${result.filename}`);
        try { await openExportFolder(); } catch {}
      } catch (e) {
        onError(`Export failed: ${e}`);
      } finally {
        setExportState(null);
      }
    },
    [onSuccess, onError],
  );

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${Math.round(ms)}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  const sourceIcon = (source: string) => {
    switch (source) {
      case "generated":
        return (
          <svg className="w-3 h-3 text-[#6C5CE7]" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
            <path d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        );
      case "variant":
        return (
          <svg className="w-3 h-3 text-[#00D2D3]" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z M2 17l10 5 10-5 M2 12l10 5 10-5" />
          </svg>
        );
      case "imported":
        return (
          <svg className="w-3 h-3 text-[#FDCB6E]" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
            <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1M8 12l4 4 4-4M12 3v13" />
          </svg>
        );
      default:
        return null;
    }
  };

  const parseTags = (tagsJson: string): string[] => {
    try {
      return JSON.parse(tagsJson);
    } catch {
      return [];
    }
  };

  return (
    <div className="fade-slide-up">
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-sm font-semibold text-[#DFE6E9] font-mono">Library</h2>
          <p className="text-[10px] text-[#636E72] font-mono mt-0.5">
            {stats.total} sounds · {stats.favCount} favorited
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={navigateToGenerator}
            className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-1.5 text-[10px] font-mono text-[#636E72] hover:border-[#6C5CE7]/50 hover:text-[#DFE6E9] transition-all"
          >
            ← Generator
          </button>
        </div>
      </div>

      <div className="flex items-center gap-2 mb-4">
        <div className="relative flex-1">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search by prompt, tag, type..."
            className="w-full rounded-lg border border-[#2A2A3F] bg-[#14141F] px-3 py-2 text-xs font-mono text-[#DFE6E9] placeholder:text-[#636E72] outline-none focus:border-[#6C5CE7]/50 transition-colors"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-[#636E72] hover:text-[#DFE6E9] p-1"
            >
              <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>
      </div>

      <div className="flex gap-1.5 mb-4 flex-wrap">
        {(["all", "favorites"] as FilterMode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => setFilterMode(mode)}
            className={`rounded-lg px-2.5 py-1 text-[10px] font-mono transition-all ${
              filterMode === mode
                ? "bg-[#6C5CE7]/20 text-[#6C5CE7] border border-[#6C5CE7]/40"
                : "bg-[#1E1E2E] text-[#636E72] border border-[#2A2A3F] hover:border-[#3A3A5F] hover:text-[#DFE6E9]"
            }`}
          >
            {mode === "all" ? "All" : `Favorites (${stats.favCount})`}
          </button>
        ))}
        <button
          onClick={refreshStats}
          className="rounded-lg bg-[#1E1E2E] px-2 py-1 text-[10px] font-mono text-[#636E72] hover:text-[#DFE6E9] border border-[#2A2A3F] transition-all"
          title="Refresh"
        >
          ↻
        </button>
      </div>

      {sounds.length === 0 && !isLoading && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-sm text-[#636E72] font-mono mb-2">No sounds found</p>
          <p className="text-xs text-[#2A2A3F] font-mono mb-4">
            {searchQuery
              ? "Try a different search"
              : filterMode === "favorites"
                ? "Favorite some sounds to see them here"
                : "Generate your first sound to see it here"}
          </p>
          {!searchQuery && filterMode !== "favorites" && (
            <button
              onClick={navigateToGenerator}
              className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-4 py-2 text-xs font-mono text-[#636E72] hover:border-[#6C5CE7]/50 hover:text-[#DFE6E9] transition-all"
            >
              Generate a sound
            </button>
          )}
        </div>
      )}

      {isLoading && sounds.length === 0 && (
        <div className="flex items-center justify-center py-16">
          <div className="spinner w-8 h-8 rounded-full border-2 border-[#2A2A3F] border-t-[#6C5CE7]" />
        </div>
      )}

      <div className="space-y-1.5">
        {sounds.map((sound) => {
          const tags = parseTags(sound.tags);
          const isFav = favorites.has(sound.id);
          return (
            <div
              key={sound.id}
              className={`group rounded-lg border bg-[#14141F]/80 p-3 transition-all ${
                confirmDelete === sound.id
                  ? "border-[#D63031]/50"
                  : isFav
                    ? "border-[#FDCB6E]/20 hover:border-[#FDCB6E]/40"
                    : "border-[#2A2A3F] hover:border-[#3A3A5F]"
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="shrink-0 w-4 flex justify-center">
                  {sourceIcon(sound.source)}
                </div>

                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handlePlay(sound.id);
                  }}
                  className="shrink-0 rounded-lg p-1.5 hover:bg-[#1E1E2E] transition-colors"
                >
                  {isPlaying && activeId === sound.id ? (
                    <svg className="w-4 h-4 text-[#00D2D3]" fill="currentColor" viewBox="0 0 24 24">
                      <rect x="6" y="4" width="4" height="16" />
                      <rect x="14" y="4" width="4" height="16" />
                    </svg>
                  ) : (
                    <svg className="w-4 h-4 text-[#636E72] group-hover:text-[#DFE6E9]" fill="currentColor" viewBox="0 0 24 24">
                      <polygon points="6,4 20,12 6,20" />
                    </svg>
                  )}
                </button>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="rounded bg-[#1E1E2E] px-1.5 py-0.5 text-[10px] font-mono font-medium uppercase tracking-wider text-[#636E72]">
                      {sound.sound_type}
                    </span>
                    <span className="text-[10px] text-[#636E72] font-mono">
                      {formatDuration(sound.duration_ms)}
                    </span>
                    {isFav && (
                      <span className="text-[10px] text-[#FDCB6E]">★</span>
                    )}
                    <span className="text-[9px] text-[#2A2A3F] font-mono ml-auto">
                      {sound.model}
                    </span>
                  </div>
                  <p className="text-xs text-[#636E72] truncate mt-0.5">
                    {sound.prompt}
                  </p>
                  {tags.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1">
                      {tags.slice(0, 6).map((tag) => (
                        <span
                          key={tag}
                          className="text-[9px] text-[#4A4A6F] font-mono"
                        >
                          #{tag}
                        </span>
                      ))}
                      {tags.length > 6 && (
                        <span className="text-[9px] text-[#2A2A3F] font-mono">
                          +{tags.length - 6}
                        </span>
                      )}
                    </div>
                  )}
                </div>

                <div className="shrink-0 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleToggleFavorite(sound.id);
                    }}
                    className={`rounded p-1.5 text-sm transition-colors ${
                      isFav
                        ? "text-[#FDCB6E]"
                        : "text-[#636E72] hover:text-[#DFE6E9]"
                    }`}
                    title={isFav ? "Unfavorite" : "Favorite"}
                  >
                    {isFav ? "★" : "☆"}
                  </button>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleExport(sound.id);
                    }}
                    disabled={exportState === sound.id}
                    className="rounded p-1.5 text-xs text-[#636E72] hover:text-[#DFE6E9] transition-colors disabled:opacity-30"
                    title="Export"
                  >
                    {exportState === sound.id ? (
                      <div className="spinner w-3.5 h-3.5 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
                    ) : (
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
                      </svg>
                    )}
                  </button>

                  {confirmDelete === sound.id ? (
                    <div className="flex items-center gap-1 ml-1">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(sound.id);
                        }}
                        className="rounded bg-[#D63031]/20 px-2 py-1 text-[10px] text-[#D63031] font-mono hover:bg-[#D63031]/30 transition-colors"
                      >
                        Delete
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setConfirmDelete(null);
                        }}
                        className="rounded bg-[#1E1E2E] px-2 py-1 text-[10px] text-[#636E72] font-mono hover:text-[#DFE6E9] transition-colors"
                      >
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setConfirmDelete(sound.id);
                      }}
                      className="rounded p-1.5 text-xs text-[#636E72] hover:text-[#D63031] transition-colors"
                      title="Delete"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                        <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7M10 11v5M14 11v5M3 7h18M15 7V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3" />
                      </svg>
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {isLoading && sounds.length > 0 && (
        <div className="mt-4 flex justify-center">
          <div className="spinner w-5 h-5 rounded-full border-2 border-[#2A2A3F] border-t-[#6C5CE7]" />
        </div>
      )}

      {hasMore && sounds.length > 0 && !isLoading && !searchQuery && filterMode !== "favorites" && (
        <div className="mt-4 flex justify-center" ref={loadMoreRef}>
          <button
            onClick={handleLoadMore}
            disabled={isLoadingMore}
            className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-6 py-2 text-[10px] font-mono text-[#636E72] hover:border-[#6C5CE7]/50 hover:text-[#DFE6E9] transition-all disabled:opacity-30"
          >
            {isLoadingMore ? "Loading..." : `Load More (${sounds.length} / ${stats.total})`}
          </button>
        </div>
      )}

      {sounds.length > 0 && stats.total > 0 && !isLoading && !searchQuery && (
        <div className="mt-3 text-center">
          <p className="text-[8px] text-[#2A2A3F] font-mono">
            Showing {sounds.length} of {stats.total} sounds · {stats.favCount} favorites
          </p>
        </div>
      )}
    </div>
  );
}
