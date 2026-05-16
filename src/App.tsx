import { useState, useCallback, useEffect, useRef } from "react";
import { PromptBar } from "./components/PromptBar";
import { SoundCard } from "./components/SoundCard";
import { VariantCard } from "./components/VariantCard";
import { ReferenceCard } from "./components/ReferenceCard";
import { LoadingSpinner } from "./components/LoadingSpinner";
import { ToastContainer } from "./components/ToastContainer";
import { LibraryView } from "./components/LibraryView";
import { ProviderSelector } from "./components/ProviderSelector";
import { useAudioPlayback } from "./hooks/useAudioPlayback";
import { useToast } from "./hooks/useToast";
import {
  generateSound,
  generateVariants,
  getAudioData,
  exportWav,
  toggleFavorite,
  getFavorites,
  analyzeReference,
  openExportFolder,
  type SoundResult,
  type VariantResult,
  type SoundMetadata,
  type ReferenceAnalysis,
  type ExportResult,
} from "./lib/api";

type View = "generator" | "library";

export default function App() {
  const [promptText, setPromptText] = useState("");
  const [sound, setSound] = useState<SoundResult | null>(null);
  const [variants, setVariants] = useState<VariantResult[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const {
    play,
    stop,
    preload,
    isPlaying,
    isLoading: isAudioLoading,
    activeId,
  } = useAudioPlayback();
  const { toasts, removeToast, success: toastSuccess, error: toastError, info: toastInfo } = useToast();
  const [referenceAnalysis, setReferenceAnalysis] = useState<ReferenceAnalysis | null>(null);
  const [referencePath, setReferencePath] = useState<string | null>(null);
  const [showExportFolder, setShowExportFolder] = useState(false);
  const [currentView, setCurrentView] = useState<View>("generator");
  const soundRef = useRef<SoundResult | null>(null);
  const variantsRef = useRef<VariantResult[]>([]);
  const shouldAutoPlayRef = useRef(false);
  const promptRef = useRef<string>("");
  const [showShortcuts, setShowShortcuts] = useState(false);

  promptRef.current = promptText;

  const loadFavorites = useCallback(async () => {
    try {
      const favs = await getFavorites();
      setFavorites(new Set(favs.map((f: SoundMetadata) => f.id)));
    } catch {}
  }, []);

  useEffect(() => {
    loadFavorites();
  }, [loadFavorites]);

  function handleGenerate(prompt: string, refPath?: string) {
    setIsGenerating(true);
    setError(null);
    setSound(null);
    setVariants([]);
    setExportResult(null);
    stop();
    generateSound(prompt, refPath)
      .then((result) => {
        setSound(result);
        shouldAutoPlayRef.current = true;
        if (result) {
          preloadAudio(result.id);
          toastSuccess("Sound generated!");
          generateVariants(prompt, result.id, 4)
            .then((variantResults) => {
              setVariants(variantResults);
              for (const v of variantResults) {
                preloadAudio(v.id);
              }
            })
            .catch(() => {});
        }
      })
      .catch((e) => {
        setError(String(e));
        toastError(String(e));
      })
      .finally(() => {
        setIsGenerating(false);
      });
  }

  const handleReferenceUpload = useCallback(async (path: string) => {
    setReferencePath(path);
    try {
      const analysis = await analyzeReference(path);
      setReferenceAnalysis(analysis);
      toastInfo("Reference loaded");
    } catch {
      setReferencePath(null);
      setReferenceAnalysis(null);
    }
  }, [toastInfo]);

  const handleClearReference = useCallback(() => {
    setReferencePath(null);
    setReferenceAnalysis(null);
  }, []);

  const handlePlayReference = useCallback(async () => {
    if (!referencePath) return;
    stop();
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const data: number[] = await invoke("read_audio_file", { path: referencePath });
      await play("ref", data, 44100);
    } catch {}
  }, [referencePath, play, stop]);

  const handlePlaySound = useCallback(
    async (soundId: string) => {
      try {
        if (isPlaying && activeId === soundId) {
          stop();
          return;
        }
        const data = await getAudioData(soundId);
        await play(soundId, data, 44100);
      } catch {}
    },
    [play, stop, isPlaying, activeId],
  );

  const handleStop = useCallback(() => {
    stop();
  }, [stop]);

  const handleFavorite = useCallback(
    async (soundId: string, _favorited: boolean) => {
      const target = sound?.id === soundId ? sound : variants.find((v) => v.id === soundId);
      if (!target) return;
      try {
        const result = await toggleFavorite(
          soundId, target.prompt, target.sound_type,
          target.duration_ms, target.seed,
          "variant_name" in target ? target.variant_name : undefined,
        );
        if (result) {
          setFavorites((prev) => new Set(prev).add(soundId));
          toastSuccess("Added to favorites");
        } else {
          setFavorites((prev) => {
            const next = new Set(prev);
            next.delete(soundId);
            return next;
          });
          toastInfo("Removed from favorites");
        }
      } catch {}
    },
    [sound, variants, toastSuccess, toastInfo],
  );

  const handleExport = useCallback(async (soundId: string) => {
    setIsExporting(true);
    setError(null);
    try {
      const result = await exportWav(soundId);
      setExportResult(result);
      setShowExportFolder(true);
      toastSuccess(`Exported: ${result.filename}`);
      setTimeout(() => {
        setExportResult(null);
        setShowExportFolder(false);
      }, 6000);
    } catch (e) {
      setError(String(e));
      toastError(String(e));
    } finally {
      setIsExporting(false);
    }
  }, [toastSuccess, toastError]);

  const handleExportAllVariants = useCallback(async () => {
    if (!sound) return;
    setIsExporting(true);
    setError(null);
    try {
      await exportWav(sound.id);
      for (const v of variants) {
        await exportWav(v.id);
      }
      setExportResult({ path: "", filename: `${variants.length + 1} sounds exported`, file_size_bytes: 0 });
      setShowExportFolder(true);
      toastSuccess(`${variants.length + 1} sounds exported`);
      setTimeout(() => {
        setExportResult(null);
        setShowExportFolder(false);
      }, 6000);
    } catch (e) {
      setError(String(e));
      toastError(String(e));
    } finally {
      setIsExporting(false);
    }
  }, [sound, variants, toastSuccess, toastError]);

  useEffect(() => {
    soundRef.current = sound;
    variantsRef.current = variants;
  }, [sound, variants]);

  useEffect(() => {
    if (sound && shouldAutoPlayRef.current) {
      shouldAutoPlayRef.current = false;
      getAudioData(sound.id).then((data) => play(sound.id, data, 44100)).catch(() => {});
    }
  }, [sound, play]);

  const preloadAudio = useCallback(async (soundId: string) => {
    try {
      const data = await getAudioData(soundId);
      await preload(soundId, data, 44100);
    } catch {}
  }, [preload]);

  const handleSuggestedPrompt = useCallback((suggestion: string) => {
    setPromptText(suggestion);
  }, []);

  const handleFavoriteToggle = useCallback((id: string) => {
    setFavorites((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const handleLibraryPlay = useCallback(async (id: string) => {
    try {
      const data = await getAudioData(id);
      await play(id, data, 44100);
    } catch {}
  }, [play]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "?" && !e.metaKey && !e.ctrlKey) {
        setShowShortcuts((p) => !p);
        return;
      }
      if (e.key === "Escape") {
        setShowShortcuts(false);
        return;
      }

      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case "1":
            e.preventDefault();
            setCurrentView("generator");
            break;
          case "2":
            e.preventDefault();
            setCurrentView("library");
            break;
          case "e":
          case "E":
            e.preventDefault();
            if (sound && currentView === "generator") {
              handleExport(sound.id);
            }
            break;
          case "f":
          case "F":
            e.preventDefault();
            if (sound && currentView === "generator") {
              handleFavorite(sound.id, favorites.has(sound.id));
            }
            break;
          case "Enter":
            e.preventDefault();
            if (currentView === "generator" && promptRef.current.trim() && !isGenerating) {
              handleGenerate(promptRef.current, referencePath ?? undefined);
            }
            break;
        }
        return;
      }

      if (e.key === " " && e.target === document.body && currentView === "generator") {
        e.preventDefault();
        const current = soundRef.current;
        if (!current) return;
        stop();
        handlePlaySound(current.id);
      }

      if (e.key === "Enter" && currentView === "generator" && promptRef.current.trim() && !isGenerating && e.target === document.body) {
        e.preventDefault();
        handleGenerate(promptRef.current, referencePath ?? undefined);
      }

      if (e.key === "r" && currentView === "generator" && sound && !e.metaKey && !e.ctrlKey && e.target === document.body) {
        e.preventDefault();
        handleGenerate(promptRef.current || sound.prompt, referencePath ?? undefined);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [currentView, sound, isGenerating, referencePath, handleExport, handleFavorite, favorites, stop, handlePlaySound]);

  return (
    <div className="flex min-h-screen flex-col bg-[#0A0A0F]">
      <header className="border-b border-[#2A2A3F]/50 px-6 py-3">
        <div className="mx-auto flex max-w-xl items-center justify-between">
          <div className="flex items-center gap-3">
            <button
              onClick={() => setCurrentView("generator")}
              className={`text-base font-semibold tracking-tight font-mono transition-colors ${
                currentView === "generator" ? "text-[#DFE6E9]" : "text-[#636E72] hover:text-[#DFE6E9]"
              }`}
            >
              cShot
            </button>
            <span className="hidden sm:inline text-[11px] text-[#636E72] font-mono">
              {currentView === "generator" ? "describe → generate → preview → export" : "browse → search → export"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <ProviderSelector />
            <button
              onClick={() => setCurrentView(currentView === "generator" ? "library" : "generator")}
              className={`rounded-lg border px-2.5 py-1.5 text-[10px] font-mono transition-all ${
                currentView === "library"
                  ? "border-[#6C5CE7]/50 bg-[#6C5CE7]/10 text-[#6C5CE7]"
                  : "border-[#2A2A3F] bg-[#1E1E2E] text-[#636E72] hover:border-[#3A3A5F] hover:text-[#DFE6E9]"
              }`}
            >
              {currentView === "generator" ? "Library" : "Generator"}
            </button>
          </div>
        </div>
      </header>

      <main className="flex-1 px-6 py-8">
        <div className="mx-auto max-w-xl">
          {currentView === "generator" && (
            <>
              <PromptBar
                prompt={promptText}
                onPromptChange={setPromptText}
                onGenerate={referencePath ? (prompt) => handleGenerate(prompt, referencePath) : handleGenerate}
                onQuickGenerate={(p) => {
                  setPromptText(p);
                  handleGenerate(p, referencePath ?? undefined);
                }}
                isGenerating={isGenerating}
                onReferenceUpload={handleReferenceUpload}
                referencePath={referencePath}
              />

              {referenceAnalysis && (
                <div className="mt-4 fade-slide-up">
                  <ReferenceCard
                    analysis={referenceAnalysis}
                    isPlaying={isPlaying && activeId === "ref"}
                    onPlay={handlePlayReference}
                    onClear={handleClearReference}
                  />
                </div>
              )}

              {error && (
                <div className="mt-4 rounded-lg border border-[#D63031]/40 bg-[#D63031]/10 px-4 py-3 fade-slide-up">
                  <p className="text-sm text-[#D63031] font-mono">{error}</p>
                  <button
                    onClick={() => {
                      setError(null);
                      if (promptText.trim()) {
                        handleGenerate(promptText, referencePath ?? undefined);
                      }
                    }}
                    className="mt-2 rounded bg-[#D63031]/20 px-3 py-1 text-xs text-[#D63031] font-mono hover:bg-[#D63031]/30 transition-colors"
                  >
                    Try Again
                  </button>
                </div>
              )}

              {exportResult && (
                <div className="mt-4 rounded-lg border border-[#00D2D3]/30 bg-[#00D2D3]/10 px-4 py-3 fade-slide-up">
                  <p className="text-xs text-[#00D2D3] font-mono break-all">
                    Exported — {exportResult.filename}
                  </p>
                  {showExportFolder && (
                    <button
                      onClick={async () => {
                        try { await openExportFolder(); } catch {}
                      }}
                      className="mt-2 rounded bg-[#00D2D3]/20 px-2 py-1 text-[10px] text-[#00D2D3] font-mono hover:bg-[#00D2D3]/30 transition-colors"
                    >
                      Open Folder
                    </button>
                  )}
                </div>
              )}

              <div className="mt-6">
                {isGenerating && <LoadingSpinner message="generating your sound..." />}

                {!isGenerating && !sound && !error && (
                  <div className="flex flex-col items-center gap-6 py-16 text-center fade-slide-up">
                    <div className="text-center">
                      <p className="text-sm text-[#636E72] font-mono mb-2">
                        Describe a sound to generate
                      </p>
                      <p className="text-xs text-[#2A2A3F] font-mono">
                        Type a prompt above and press Enter, or pick a recipe below
                      </p>
                    </div>
                    <div className="w-px h-8 bg-[#2A2A3F]" />
                    <div>
                      <p className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider mb-3">
                        Quick start
                      </p>
                      <div className="flex flex-wrap justify-center gap-2">
                        {["punchy kick 140", "bright snare", "tight hi-hat", "warm sub bass", "cinematic impact"].map((p) => (
                          <button
                            key={p}
                            onClick={() => handleSuggestedPrompt(p)}
                            className="rounded-full border border-[#2A2A3F] bg-[#14141F] px-3 py-1.5 text-xs text-[#636E72] font-mono transition-colors hover:border-[#3A3A5F] hover:text-[#DFE6E9]"
                          >
                            {p}
                          </button>
                        ))}
                      </div>
                    </div>
                    <div className="w-px h-8 bg-[#2A2A3F]" />
                    <div>
                      <p className="text-[9px] text-[#2A2A3F] font-mono">
                        ⌘1 Generator · ⌘2 Library · ? Shortcuts
                      </p>
                    </div>
                  </div>
                )}

                {!isGenerating && sound && (
                  <div className="space-y-4">
                    {referenceAnalysis && (
                      <div className="fade-slide-up">
                        <ReferenceCard
                          analysis={referenceAnalysis}
                          isPlaying={isPlaying && activeId === "ref"}
                          onPlay={handlePlayReference}
                          onClear={handleClearReference}
                        />
                      </div>
                    )}

                    <div className="fade-slide-up">
                      <SoundCard
                        key={sound.id}
                        id={sound.id}
                        waveform={sound.waveform}
                        soundType={sound.sound_type}
                        tags={sound.tags}
                        durationMs={sound.duration_ms}
                        prompt={sound.prompt}
                        score={sound.score}
                        model={sound.model}
                        isFavorited={favorites.has(sound.id)}
                        isPlaying={isPlaying && activeId === sound.id}
                        isLoading={isAudioLoading && activeId === sound.id}
                        onPlay={handlePlaySound}
                        onStop={handleStop}
                        onFavorite={handleFavorite}
                        onExport={handleExport}
                      />
                    </div>

                    {variants.length > 0 && (
                      <div className="fade-slide-up">
                        <div className="flex items-center justify-between mb-3">
                          <div className="flex items-center gap-2">
                            <h2 className="text-[10px] font-mono font-medium text-[#636E72] uppercase tracking-wider">
                              Variations
                            </h2>
                            <span className="text-[10px] text-[#2A2A3F] font-mono">
                              ({variants.length})
                            </span>
                          </div>
                          <button
                            onClick={handleExportAllVariants}
                            disabled={isExporting}
                            className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-2.5 py-1 text-[9px] font-mono text-[#636E72] hover:text-[#DFE6E9] transition-colors disabled:opacity-30"
                          >
                            Export All
                          </button>
                        </div>
                        <div className="grid grid-cols-1 gap-2">
                          {variants.map((v) => (
                            <VariantCard
                              key={v.id}
                              id={v.id}
                              waveform={v.waveform}
                              variantName={v.variant_name}
                              soundType={v.sound_type}
                              durationMs={v.duration_ms}
                              isFavorited={favorites.has(v.id)}
                              isPlaying={isPlaying && activeId === v.id}
                              isLoading={isAudioLoading && activeId === v.id}
                              onPlay={handlePlaySound}
                              onStop={handleStop}
                              onFavorite={handleFavorite}
                              onExport={handleExport}
                            />
                          ))}
                        </div>
                      </div>
                    )}

                    <div className="flex justify-center">
                      <button
                        onClick={() => handleGenerate(promptText, referencePath ?? undefined)}
                        disabled={isGenerating}
                        className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-4 py-2 text-[10px] font-mono text-[#636E72] hover:border-[#6C5CE7]/50 hover:text-[#DFE6E9] transition-all disabled:opacity-30"
                      >
                        Regenerate (R)
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </>
          )}

          {currentView === "library" && (
            <LibraryView
              onPlay={handleLibraryPlay}
              onStop={handleStop}
              isPlaying={isPlaying}
              activeId={activeId}
              isAudioLoading={isAudioLoading}
              favorites={favorites}
              onFavoriteToggle={handleFavoriteToggle}
              onError={toastError}
              onSuccess={toastSuccess}
              navigateToGenerator={() => setCurrentView("generator")}
            />
          )}
        </div>
      </main>

      <footer className="border-t border-[#2A2A3F]/50 px-6 py-3">
        <div className="mx-auto flex max-w-xl items-center justify-between">
          <span className="text-[10px] text-[#2A2A3F] font-mono">
            {currentView === "generator"
              ? isExporting
                ? "exporting..."
                : sound
                  ? `${sound.sound_type} · ${favorites.size} faved · ${variants.length} variations`
                  : "ready"
              : "library"}
          </span>
          <span className="text-[10px] text-[#2A2A3F] font-mono">
            {currentView === "generator" ? "(space) play · (r) regenerate" : ""}
          </span>
        </div>
      </footer>

      {showShortcuts && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="rounded-2xl border border-[#2A2A3F] bg-[#1A1A2E] p-8 max-w-md w-full mx-4 shadow-2xl">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-sm font-semibold text-[#DFE6E9] font-mono">Keyboard Shortcuts</h2>
              <button onClick={() => setShowShortcuts(false)} className="text-[#636E72] hover:text-[#DFE6E9]">
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div className="space-y-3">
              {[
                { keys: "⌘1", desc: "Generator view" },
                { keys: "⌘2", desc: "Library view" },
                { keys: "Space", desc: "Play / Stop current sound" },
                { keys: "Enter", desc: "Generate (from input)" },
                { keys: "⌘Enter", desc: "Generate from anywhere" },
                { keys: "R", desc: "Regenerate sound" },
                { keys: "⌘E", desc: "Export current sound" },
                { keys: "⌘F", desc: "Favorite current sound" },
                { keys: "?", desc: "Toggle this panel" },
                { keys: "Esc", desc: "Close panel" },
              ].map(({ keys, desc }) => (
                <div key={keys} className="flex items-center justify-between">
                  <kbd className="rounded border border-[#2A2A3F] bg-[#14141F] px-2 py-1 text-[10px] font-mono text-[#6C5CE7] min-w-[80px] text-center">
                    {keys}
                  </kbd>
                  <span className="text-xs text-[#636E72] font-mono">{desc}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </div>
  );
}
