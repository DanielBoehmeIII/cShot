import { useState, useEffect, useCallback, useRef } from "react";
import { analyzePrompt, type PromptDspControls } from "../lib/api";

interface PromptChipsProps {
  prompt: string;
  onParamClick?: (descriptor: string) => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  transient: "bg-[#E17055]/15 text-[#E17055] border-[#E17055]/30",
  spectral: "bg-[#6C5CE7]/15 text-[#6C5CE7] border-[#6C5CE7]/30",
  distortion: "bg-[#D63031]/15 text-[#D63031] border-[#D63031]/30",
  temporal: "bg-[#00D2D3]/15 text-[#00D2D3] border-[#00D2D3]/30",
  sub: "bg-[#00B894]/15 text-[#00B894] border-[#00B894]/30",
  noise: "bg-[#636E72]/20 text-[#636E72] border-[#636E72]/30",
  fx: "bg-[#FDCB6E]/15 text-[#FDCB6E] border-[#FDCB6E]/30",
  body: "bg-[#E17055]/10 text-[#E17055] border-[#E17055]/20",
};

function categoryStyle(cat: string): string {
  return CATEGORY_COLORS[cat] || "bg-[#2A2A3F]/50 text-[#636E72] border-[#2A2A3F]";
}

export function PromptChips({ prompt, onParamClick }: PromptChipsProps) {
  const [controls, setControls] = useState<PromptDspControls | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();
  const lastPromptRef = useRef("");

  useEffect(() => {
    if (!prompt.trim()) {
      setControls(null);
      return;
    }

    if (debounceRef.current) clearTimeout(debounceRef.current);

    debounceRef.current = setTimeout(async () => {
      if (prompt === lastPromptRef.current) return;
      lastPromptRef.current = prompt;
      setIsLoading(true);
      try {
        const result = await analyzePrompt(prompt);
        setControls(result);
      } catch {
        setControls(null);
      } finally {
        setIsLoading(false);
      }
    }, 300);
  }, [prompt]);

  const handleChipClick = useCallback((word: string) => {
    onParamClick?.(word);
  }, [onParamClick]);

  if (!prompt.trim() || (!controls?.descriptors.length && !controls?.genre_hints.length)) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-1.5 mt-2 min-h-[24px]">
      {isLoading && (
        <div className="spinner w-3 h-3 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7] self-center" />
      )}

      {!isLoading && controls && (
        <>
          {controls.descriptors.slice(0, 8).map((desc) => (
            <button
              key={desc.word}
              onClick={() => handleChipClick(desc.word)}
              title={`${desc.description} (${(desc.confidence * 100).toFixed(0)}%)`}
              className={`rounded-full border px-2 py-0.5 text-[9px] font-mono transition-all hover:opacity-80 ${categoryStyle(desc.category)}`}
            >
              {desc.word}
            </button>
          ))}

          {controls.genre_hints.slice(0, 3).map((genre) => (
            <span
              key={genre}
              className="rounded-full border border-[#6C5CE7]/30 bg-[#6C5CE7]/10 px-2 py-0.5 text-[9px] font-mono text-[#6C5CE7]"
            >
              {genre}
            </span>
          ))}

          {controls.bpm && (
            <span className="rounded-full border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[9px] font-mono text-[#636E72]">
              {controls.bpm} BPM
            </span>
          )}
        </>
      )}
    </div>
  );
}
