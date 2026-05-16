import { useState, useCallback, useEffect, useRef } from "react";
import { PromptBar } from "./components/PromptBar";
import { SoundCard } from "./components/SoundCard";
import { VariantCard } from "./components/VariantCard";
import { ReferenceCard } from "./components/ReferenceCard";
import { LoadingSpinner } from "./components/LoadingSpinner";
import { useAudioPlayback } from "./hooks/useAudioPlayback";
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
  const [referenceAnalysis, setReferenceAnalysis] =
    useState<ReferenceAnalysis | null>(null);
  const [referencePath, setReferencePath] = useState<string | null>(null);
  const [showExportFolder, setShowExportFolder] = useState(false);
  const soundRef = useRef<SoundResult | null>(null);
  const variantsRef = useRef<VariantResult[]>([]);
  const shouldAutoPlayRef = useRef(false);

  const loadFavorites = useCallback(async () => {
    try {
      const favs = await getFavorites();
      setFavorites(new Set(favs.map((f: SoundMetadata) => f.id)));
    } catch {
      /* ok */
    }
  }, []);

  useEffect(() => {
    loadFavorites();
  }, [loadFavorites]);

  const handleReferenceUpload = useCallback(async (path: string) => {
    setReferencePath(path);
    try {
      const analysis = await analyzeReference(path);
      setReferenceAnalysis(analysis);
    } catch {
      setReferencePath(null);
      setReferenceAnalysis(null);
    }
  }, []);

  const handleClearReference = useCallback(() => {
    setReferencePath(null);
    setReferenceAnalysis(null);
  }, []);

  const handlePlayReference = useCallback(async () => {
    if (!referencePath) return;
    stop();
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const data: number[] = await invoke("read_audio_file", {
        path: referencePath,
      });
      await play("ref", data, 44100);
    } catch {
      /* ok */
    }
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
      } catch {
        /* ok */
      }
    },
    [play, stop, isPlaying, activeId],
  );

  const handleStop = useCallback(() => {
    stop();
  }, [stop]);

  const handleGenerate = useCallback(
    async (prompt: string, refPath?: string) => {
      setIsGenerating(true);
      setError(null);
      setSound(null);
      setVariants([]);
      setExportResult(null);
      stop();
      try {
        const result = await generateSound(prompt, refPath);
        setSound(result);
        shouldAutoPlayRef.current = true;
        if (result) {
          preloadAudio(result.id);
          const variantResults = await generateVariants(prompt, result.id, 4);
          setVariants(variantResults);
          for (const v of variantResults) {
            preloadAudio(v.id);
          }
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setIsGenerating(false);
      }
    },
    [stop],
  );

  const handleFavorite = useCallback(
    async (soundId: string, _favorited: boolean) => {
      const target =
        sound?.id === soundId
          ? sound
          : variants.find((v) => v.id === soundId);
      if (!target) return;
      try {
        const result = await toggleFavorite(
          soundId,
          target.prompt,
          target.sound_type,
          target.duration_ms,
          target.seed,
          "variant_name" in target ? target.variant_name : undefined,
        );
        if (result) {
          setFavorites((prev) => new Set(prev).add(soundId));
        } else {
          setFavorites((prev) => {
            const next = new Set(prev);
            next.delete(soundId);
            return next;
          });
        }
      } catch {
        /* ok */
      }
    },
    [sound, variants],
  );

  const handleExport = useCallback(async (soundId: string) => {
    setIsExporting(true);
    setError(null);
    try {
      const result = await exportWav(soundId);
      setExportResult(result);
      setShowExportFolder(true);
      setTimeout(() => {
        setExportResult(null);
        setShowExportFolder(false);
      }, 6000);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsExporting(false);
    }
  }, []);

  const handleExportAllVariants = useCallback(async () => {
    if (!sound) return;
    setIsExporting(true);
    setError(null);
    try {
      await exportWav(sound.id);
      for (const v of variants) {
        await exportWav(v.id);
      }
      setExportResult({
        path: "",
        filename: `${variants.length + 1} sounds exported`,
        file_size_bytes: 0,
      });
      setShowExportFolder(true);
      setTimeout(() => {
        setExportResult(null);
        setShowExportFolder(false);
      }, 6000);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsExporting(false);
    }
  }, [sound, variants]);

  useEffect(() => {
    soundRef.current = sound;
    variantsRef.current = variants;
  }, [sound, variants]);

  const handlePlaySoundRef = useRef(handlePlaySound);
  handlePlaySoundRef.current = handlePlaySound;
  const stopRef = useRef(stop);
  stopRef.current = stop;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === " " && e.target === document.body) {
        e.preventDefault();
        const current = soundRef.current;
        if (!current) return;
        stopRef.current();
        handlePlaySoundRef.current(current.id);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  useEffect(() => {
    if (sound && shouldAutoPlayRef.current) {
      shouldAutoPlayRef.current = false;
      const id = sound.id;
      const dataPromise = getAudioData(id);
      dataPromise.then((data) => play(id, data, 44100)).catch(() => {});
    }
  }, [sound]);

  const preloadAudio = useCallback(async (soundId: string) => {
    try {
      const data = await getAudioData(soundId);
      await preload(soundId, data, 44100);
    } catch {
      /* ok */
    }
  }, [preload]);

  const handleSuggestedPrompt = useCallback((suggestion: string) => {
    setPromptText(suggestion);
  }, []);

  return (
    <div className="flex min-h-screen flex-col bg-[#0A0A0F]">
      <header className="border-b border-[#2A2A3F]/50 px-6 py-3">
        <div className="mx-auto flex max-w-xl items-center justify-between">
          <div className="flex items-center gap-3">
            <h1 className="text-base font-semibold tracking-tight text-[#DFE6E9] font-mono">
              cShot
            </h1>
            <span className="hidden sm:inline text-[11px] text-[#636E72] font-mono">
              describe &rarr; generate &rarr; preview &rarr; export
            </span>
          </div>
        </div>
      </header>

      <main className="flex-1 px-6 py-8">
        <div className="mx-auto max-w-xl">
          <PromptBar
            prompt={promptText}
            onPromptChange={setPromptText}
            onGenerate={
              referencePath
                ? (prompt) => handleGenerate(prompt, referencePath)
                : handleGenerate
            }
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
                    handleGenerate(
                      promptText,
                      referencePath ?? undefined,
                    );
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
                Exported &mdash; {exportResult.filename}
              </p>
              {showExportFolder && (
                <button
                  onClick={async () => {
                    try {
                      await openExportFolder();
                    } catch {}
                  }}
                  className="mt-2 rounded bg-[#00D2D3]/20 px-2 py-1 text-[10px] text-[#00D2D3] font-mono hover:bg-[#00D2D3]/30 transition-colors"
                >
                  Open Folder
                </button>
              )}
            </div>
          )}

          <div className="mt-6">
            {isGenerating && (
              <LoadingSpinner message="generating your sound..." />
            )}

            {!isGenerating && !sound && !error && (
              <div className="flex flex-col items-center gap-6 py-16 text-center fade-slide-up">
                <div className="text-center">
                  <p className="text-sm text-[#636E72] font-mono mb-2">
                    Describe a sound to generate
                  </p>
                  <p className="text-xs text-[#2A2A3F] font-mono">
                    Type a prompt above and press Enter, or drag in a reference WAV
                  </p>
                </div>
                <div className="w-px h-8 bg-[#2A2A3F]" />
                <div>
                  <p className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider mb-3">
                    Try typing
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
              </div>
            )}
          </div>
        </div>
      </main>

      <footer className="border-t border-[#2A2A3F]/50 px-6 py-3">
        <div className="mx-auto flex max-w-xl items-center justify-between">
          <span className="text-[10px] text-[#2A2A3F] font-mono">
            {isExporting
              ? "exporting..."
              : sound
                ? `${sound.sound_type} · ${favorites.size} faved · ${variants.length} variations`
                : "ready"}
          </span>
        </div>
      </footer>
    </div>
  );
}
