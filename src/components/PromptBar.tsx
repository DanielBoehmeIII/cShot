import { useRef, useEffect, useState } from "react";

interface PromptBarProps {
  prompt: string;
  onPromptChange: (prompt: string) => void;
  onGenerate: (prompt: string, referencePath?: string) => void;
  onQuickGenerate: (prompt: string) => void;
  isGenerating: boolean;
  onReferenceUpload?: (path: string) => void;
  referencePath?: string | null;
}

const RECIPE_CHIPS = [
  { label: "Trap Kick", prompt: "punchy trap kick 140bpm, punchy, short decay, subby, clicky transient", genre: "trap" },
  { label: "Drill 808", prompt: "dark drill 808 hit 155bpm, heavy sub, distorted, punchy attack", genre: "drill" },
  { label: "Hyperpop Snare", prompt: "hyperpop snare 170bpm, bright, glossy, pitchy, short, crisp, loud", genre: "hyperpop" },
  { label: "House Kick", prompt: "round warm house kick 125bpm, four-on-the-floor, moderate attack, sub", genre: "house" },
  { label: "Techno Kick", prompt: "driving techno kick 130bpm, industrial, gritty, distorted, long decay", genre: "techno" },
  { label: "Cinematic Impact", prompt: "epic cinematic impact with sub boom, massive, orchestral, reverb", genre: "cinematic" },
  { label: "Pop Perc", prompt: "clean pop percussion hit, polished, bright, tight, expensive", genre: "pop" },
  { label: "Rimshot", prompt: "crisp rimshot, wooden tone, bright, short, 100ms", genre: "pop" },
  { label: "Closed Hat", prompt: "tight closed hi-hat, bright, short, crisp, 50ms", genre: "trap" },
  { label: "Open Hat", prompt: "open hi-hat, washy, bright, long decay, 400ms", genre: "house" },
  { label: "Clap", prompt: "crisp layered clap, room ambience, warm, 250ms", genre: "house" },
  { label: "Crash", prompt: "bright crash cymbal, shimmering, long decay, 2s", genre: "cinematic" },
  { label: "Snare", prompt: "bright snare with crack, layered, resonant, 300ms", genre: "drill" },
  { label: "808 Sub", prompt: "dark 808 sub bass 140bpm, long decay, distorted, punchy", genre: "trap" },
  { label: "Perc Hit", prompt: "percussion hit, metallic, bright, short, 150ms", genre: "techno" },
  { label: "UI Click", prompt: "UI click sound, clean, short, digital, 50ms", genre: "game" },
  { label: "Sword Swish", prompt: "sword swing whoosh, sharp attack, airy, 500ms", genre: "game" },
  { label: "Footstep", prompt: "heavy footstep impact, thud, short, 200ms", genre: "game" },
];

const SOUND_TYPES = [
  { label: "Kick", prompt: "punchy kick" },
  { label: "Snare", prompt: "bright snare with crack" },
  { label: "Hi-Hat", prompt: "tight hi-hat" },
  { label: "Clap", prompt: "crisp clap" },
  { label: "Perc", prompt: "percussion hit" },
  { label: "FX", prompt: "cinematic fx" },
];

export function PromptBar({
  prompt,
  onPromptChange,
  onGenerate,
  onQuickGenerate,
  isGenerating,
  onReferenceUpload,
  referencePath: externalPath,
}: PromptBarProps) {
  const [localRefPath, setLocalRefPath] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const refPath = externalPath ?? localRefPath;

  useEffect(() => {
    if (externalPath) {
      setLocalRefPath(null);
    }
  }, [externalPath]);

  useEffect(() => {
    if (!isGenerating && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isGenerating]);

  const handleSubmit = () => {
    const trimmed = prompt.trim();
    if (!trimmed || isGenerating) return;
    onGenerate(trimmed, refPath ?? undefined);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSubmit();
  };

  const handlePickReference = async () => {
    const { openReferenceDialog } = await import("../lib/api");
    const path = await openReferenceDialog();
    if (path) {
      setLocalRefPath(path);
      onReferenceUpload?.(path);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file && file.name.endsWith(".wav")) {
      setLocalRefPath(file.name);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = () => setIsDragOver(false);

  return (
    <div className="flex flex-col gap-4">
      <div
        className={`relative flex items-center gap-2 rounded-xl border-2 bg-[#14141F] p-1 transition-all duration-200 ${
          isDragOver
            ? "border-[#6C5CE7] shadow-[0_0_24px_rgba(108,92,231,0.15)]"
            : refPath
              ? "border-[#00D2D3]/40"
              : "border-[#2A2A3F] hover:border-[#3A3A5F]"
        }`}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
      >
        <input
          ref={inputRef}
          type="text"
          value={prompt}
          onChange={(e) => onPromptChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            refPath
              ? "describe the variant..."
              : "describe a sound, or pick a recipe below"
          }
          className="flex-1 bg-transparent px-3 py-3 text-sm outline-none placeholder:text-[#636E72]"
          disabled={isGenerating}
        />
        <button
          onClick={handlePickReference}
          className={`shrink-0 rounded-lg px-2.5 py-1.5 text-xs font-medium transition-colors ${
            refPath
              ? "bg-[#00D2D3]/20 text-[#00D2D3]"
              : "bg-[#1E1E2E] text-[#636E72] hover:bg-[#2A2A3F] hover:text-[#DFE6E9]"
          }`}
          title="Upload reference WAV"
          disabled={isGenerating}
        >
          {refPath ? "REF" : "WAV"}
        </button>
        <button
          onClick={handleSubmit}
          disabled={!prompt.trim() || isGenerating}
          className="shrink-0 rounded-lg bg-[#6C5CE7] px-4 py-2.5 text-sm font-medium text-white transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed"
        >
          {isGenerating ? (
            <span className="flex items-center gap-2">
              <span className="spinner inline-block w-3 h-3 rounded-full border-2 border-white/30 border-t-white" />
              Generating
            </span>
          ) : (
            "Generate"
          )}
        </button>
      </div>

      {refPath && (
        <div className="flex items-center gap-2 px-1">
          <span className="text-xs text-[#00D2D3]">reference loaded</span>
          <button
            onClick={() => setLocalRefPath(null)}
            className="text-xs text-[#636E72] hover:text-[#DFE6E9]"
          >
            clear
          </button>
        </div>
      )}

      <div>
        <p className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider mb-2 px-1">
          Professional Recipes
        </p>
        <div className="flex flex-wrap gap-1.5 px-1">
          {RECIPE_CHIPS.map((chip) => (
            <button
              key={chip.label}
              onClick={() => onQuickGenerate(chip.prompt)}
              className="group relative rounded-lg border border-[#2A2A3F] bg-[#14141F] px-2.5 py-1.5 text-[11px] font-mono text-[#636E72] transition-all hover:border-[#6C5CE7]/50 hover:bg-[#6C5CE7]/10 hover:text-[#DFE6E9]"
              disabled={isGenerating}
            >
              <span className="text-[9px] text-[#4A4A6F] mr-1">▸</span>
              {chip.label}
              <span className="ml-1.5 text-[9px] text-[#2A2A3F]">{chip.genre}</span>
            </button>
          ))}
        </div>
      </div>

      <div>
        <p className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider mb-2 px-1">
          Quick Sound Types
        </p>
        <div className="flex flex-wrap gap-1.5 px-1">
          {SOUND_TYPES.map((chip) => {
            const isActive = prompt.toLowerCase().includes(chip.prompt.split(" ")[0]);
            return (
              <button
                key={chip.label}
                onClick={() => onPromptChange(chip.prompt)}
                className={`rounded-full px-2.5 py-1 text-[11px] font-mono transition-all ${
                  isActive
                    ? "bg-[#6C5CE7]/20 text-[#6C5CE7] border border-[#6C5CE7]/40"
                    : "bg-[#1E1E2E] text-[#636E72] border border-[#2A2A3F] hover:border-[#3A3A5F] hover:text-[#DFE6E9]"
                }`}
              >
                {chip.label}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
