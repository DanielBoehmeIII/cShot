interface VariantCardProps {
  id: string;
  waveform: number[];
  variantName: string;
  soundType: string;
  durationMs: number;
  isFavorited: boolean;
  isPlaying: boolean;
  isLoading?: boolean;
  onPlay: (id: string) => void;
  onStop: () => void;
  onFavorite: (id: string, favorited: boolean) => void;
  onExport: (id: string) => void;
}

export function VariantCard({
  id,
  waveform,
  variantName,
  durationMs,
  isFavorited,
  isPlaying,
  isLoading,
  onPlay,
  onStop,
  onFavorite,
  onExport,
}: VariantCardProps) {
  const svgWidth = 160;
  const svgHeight = 32;
  const centerY = svgHeight / 2;

  const pathD = waveform
    .map((val, i) => {
      const x = (i / (waveform.length - 1)) * svgWidth;
      const y = centerY - val * (centerY - 2);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  const pathD2 = waveform
    .map((val, i) => {
      const x = (i / (waveform.length - 1)) * svgWidth;
      const y = centerY + val * (centerY - 2);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  const handlePlay = () => {
    if (isLoading) return;
    if (isPlaying) {
      onStop();
    } else {
      onPlay(id);
    }
  };

  const handleFavorite = () => {
    if (isLoading) return;
    onFavorite(id, !isFavorited);
  };

  const handleExport = () => {
    if (isLoading) return;
    onExport(id);
  };

  return (
    <div
      className={`rounded-lg border bg-[#14141F]/80 p-3 transition-all ${
        isPlaying
          ? "border-[#6C5CE7]/40"
          : isFavorited
            ? "border-[#FDCB6E]/20"
            : "border-[#2A2A3F] hover:border-[#3A3A5F]"
      }`}
    >
      <div className="flex items-center gap-3">
        <div className="shrink-0">
          <span className="rounded-md bg-[#6C5CE7]/10 px-1.5 py-0.5 text-[10px] font-mono font-bold uppercase tracking-wider text-[#6C5CE7]">
            {variantName}
          </span>
        </div>

        <button onClick={handlePlay} disabled={isLoading} className="shrink-0">
          {isLoading ? (
            <div className="spinner w-4 h-4 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
          ) : isPlaying ? (
            <svg
              className="w-4 h-4 text-[#00D2D3]"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <rect x="6" y="4" width="4" height="16" />
              <rect x="14" y="4" width="4" height="16" />
            </svg>
          ) : (
            <svg
              className="w-4 h-4 text-[#636E72] hover:text-[#DFE6E9]"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <polygon points="6,4 20,12 6,20" />
            </svg>
          )}
        </button>

        <svg
          viewBox={`0 0 ${svgWidth} ${svgHeight}`}
          className={`flex-1 h-8 ${isPlaying ? "opacity-100" : "opacity-40"}`}
        >
          <defs>
            <linearGradient id={`var-grad-${id}`} x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%" stopColor="#6C5CE7" />
              <stop offset="100%" stopColor="#00D2D3" />
            </linearGradient>
          </defs>
          <path
            d={pathD}
            fill="none"
            stroke={`url(#var-grad-${id})`}
            strokeWidth="1"
          />
          <path
            d={pathD2}
            fill="none"
            stroke={`url(#var-grad-${id})`}
            strokeWidth="1"
          />
        </svg>

        <span className="shrink-0 text-[10px] text-[#636E72] font-mono">
          {durationMs < 1000
            ? `${Math.round(durationMs)}ms`
            : `${(durationMs / 1000).toFixed(1)}s`}
        </span>

        <button
          onClick={handleFavorite}
          disabled={isLoading}
          className={`shrink-0 text-sm transition-colors ${
            isLoading
              ? "text-[#2A2A3F] cursor-not-allowed"
              : isFavorited
                ? "text-[#FDCB6E]"
                : "text-[#2A2A3F] hover:text-[#636E72]"
          }`}
        >
          {isFavorited ? "\u2665" : "\u2661"}
        </button>

        <button
          onClick={handleExport}
          disabled={isLoading}
          className={`shrink-0 rounded px-2 py-1 text-[10px] transition-colors font-mono ${
            isLoading
              ? "bg-[#1E1E2E]/50 text-[#636E72] cursor-not-allowed"
              : "bg-[#1E1E2E] text-[#636E72] hover:bg-[#2A2A3F] hover:text-[#DFE6E9]"
          }`}
        >
          Export
        </button>
      </div>
    </div>
  );
}
