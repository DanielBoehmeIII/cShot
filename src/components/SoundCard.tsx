const TAG_COLORS: Record<string, string> = {
  kick: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  snare: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  closed_hat: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  open_hat: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  clap: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  tom: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  perc: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  bass: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  fx: "bg-[#6C5CE7]/15 text-[#6C5CE7]",
  other: "bg-[#636E72]/15 text-[#636E72]",
  bright: "bg-[#FDCB6E]/15 text-[#FDCB6E]",
  dark: "bg-[#2D3436]/30 text-[#B2BEC3]",
  warm: "bg-[#E17055]/15 text-[#E17055]",
  punchy: "bg-[#E17055]/20 text-[#E17055]",
  short: "bg-[#00D2D3]/15 text-[#00D2D3]",
  long: "bg-[#00B894]/15 text-[#00B894]",
  distorted: "bg-[#D63031]/20 text-[#D63031]",
  clean: "bg-[#00B894]/15 text-[#00B894]",
  sub: "bg-[#6C5CE7]/20 text-[#6C5CE7]",
};

function tagStyle(tag: string): string {
  return TAG_COLORS[tag] || "bg-[#1E1E2E] text-[#636E72]";
}

function scoreColor(score: number): string {
  if (score >= 70) return "text-[#00B894]";
  if (score >= 50) return "text-[#FDCB6E]";
  return "text-[#D63031]";
}

function scoreBg(score: number): string {
  if (score >= 70) return "bg-[#00B894]/10 border-[#00B894]/20";
  if (score >= 50) return "bg-[#FDCB6E]/10 border-[#FDCB6E]/20";
  return "bg-[#D63031]/10 border-[#D63031]/20";
}

interface SoundCardProps {
  id: string;
  waveform: number[];
  soundType: string;
  tags: string[];
  durationMs: number;
  prompt: string;
  score?: number;
  model?: string;
  isFavorited: boolean;
  isPlaying: boolean;
  isLoading?: boolean;
  onPlay: (id: string) => void;
  onStop: () => void;
  onFavorite: (id: string, favorited: boolean) => void;
  onExport: (id: string) => void;
}

export function SoundCard({
  id,
  waveform,
  soundType,
  tags,
  durationMs,
  prompt,
  score,
  model,
  isFavorited,
  isPlaying,
  isLoading,
  onPlay,
  onStop,
  onFavorite,
  onExport,
}: SoundCardProps) {
  const handlePlay = () => {
    if (isLoading) return;
    if (isPlaying) { onStop(); }
    else { onPlay(id); }
  };

  const handleFavorite = () => {
    if (isLoading) return;
    onFavorite(id, !isFavorited);
  };

  const handleExport = () => {
    if (isLoading) return;
    onExport(id);
  };

  const svgWidth = 240;
  const svgHeight = 64;
  const centerY = svgHeight / 2;

  const pathD = waveform
    .map((val, i) => {
      const x = (i / (waveform.length - 1)) * svgWidth;
      const y = centerY - val * (centerY - 4);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  const pathD2 = waveform
    .map((val, i) => {
      const x = (i / (waveform.length - 1)) * svgWidth;
      const y = centerY + val * (centerY - 4);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  return (
    <div
      className={`rounded-xl border bg-[#14141F] p-5 transition-all ${
        isFavorited
          ? "border-[#FDCB6E]/30 shadow-[0_0_16px_rgba(253,203,110,0.06)]"
          : isPlaying
            ? "border-[#6C5CE7]/50 shadow-[0_0_20px_rgba(108,92,231,0.08)]"
            : "border-[#2A2A3F] hover:border-[#3A3A5F]"
      }`}
    >
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-2 min-w-0">
          <span className="rounded-md bg-[#1E1E2E] px-2 py-0.5 text-[10px] font-mono font-medium uppercase tracking-wider text-[#636E72]">
            {soundType}
          </span>
          <span className="text-[10px] text-[#636E72] font-mono">
            {durationMs < 1000
              ? `${Math.round(durationMs)}ms`
              : `${(durationMs / 1000).toFixed(1)}s`}
          </span>
          {model && (
            <span className="text-[9px] text-[#2A2A3F] font-mono">
              {model}
            </span>
          )}
        </div>
        {score !== undefined && (
          <div
            className={`shrink-0 flex items-center gap-1 rounded-md border px-2 py-0.5 ${scoreBg(score)}`}
          >
            <span className={`text-[10px] font-mono font-bold ${scoreColor(score)}`}>
              {score}
            </span>
          </div>
        )}
      </div>

      <button
        onClick={handlePlay}
        disabled={isLoading}
        className={`group relative mb-4 w-full cursor-pointer ${isLoading ? "opacity-50" : ""}`}
      >
        <svg
          viewBox={`0 0 ${svgWidth} ${svgHeight}`}
          className={`w-full h-16 transition-all duration-200 ${
            isPlaying ? "opacity-100" : "opacity-50 group-hover:opacity-80"
          }`}
        >
          <defs>
            <linearGradient id={`wave-grad-${id}`} x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%" stopColor="#6C5CE7" />
              <stop offset="100%" stopColor="#00D2D3" />
            </linearGradient>
          </defs>
          <path d={pathD} fill="none" stroke={`url(#wave-grad-${id})`} strokeWidth="1.5" />
          <path d={pathD2} fill="none" stroke={`url(#wave-grad-${id})`} strokeWidth="1.5" />
          {isPlaying && (
            <rect x={0} y={0} width={3} height={svgHeight} fill="#00D2D3" opacity="0.5">
              <animate
                attributeName="x"
                values={`0;${svgWidth - 3};0`}
                dur={waveform.length > 0 ? `${(durationMs || 1000) / 1000}s` : "2s"}
                repeatCount="indefinite"
              />
            </rect>
          )}
        </svg>
        <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
          <div className="rounded-full bg-black/70 p-3 backdrop-blur-sm">
            {isLoading ? (
              <div className="spinner w-5 h-5 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
            ) : isPlaying ? (
              <svg className="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
                <rect x="6" y="4" width="4" height="16" />
                <rect x="14" y="4" width="4" height="16" />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
                <polygon points="6,4 20,12 6,20" />
              </svg>
            )}
          </div>
        </div>
      </button>

      {tags.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-3">
          {tags.map((tag) => (
            <span key={tag} className={`rounded-full px-2 py-0.5 text-[10px] font-mono ${tagStyle(tag)}`}>
              {tag}
            </span>
          ))}
        </div>
      )}

      <p className="text-xs text-[#636E72] truncate mb-4">{prompt}</p>

      <div className="flex items-center gap-2">
        <button
          onClick={handlePlay}
          disabled={isLoading}
          className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs transition-colors font-mono ${
            isLoading
              ? "bg-[#1E1E2E]/50 text-[#636E72] cursor-not-allowed"
              : isPlaying
                ? "bg-[#00D2D3]/15 text-[#00D2D3]"
                : "bg-[#1E1E2E] text-[#636E72] hover:bg-[#2A2A3F] hover:text-[#DFE6E9]"
          }`}
        >
          {isLoading ? (
            <>
              <div className="spinner w-3 h-3 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
              Loading
            </>
          ) : isPlaying ? (
            <>
              <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                <rect x="6" y="4" width="4" height="16" />
                <rect x="14" y="4" width="4" height="16" />
              </svg>
              Stop
            </>
          ) : (
            <>
              <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                <polygon points="6,4 20,12 6,20" />
              </svg>
              Play
            </>
          )}
        </button>

        <button
          onClick={handleFavorite}
          disabled={isLoading}
          className={`rounded-lg px-3 py-1.5 text-xs transition-colors ${
            isLoading
              ? "bg-[#1E1E2E]/50 text-[#636E72] cursor-not-allowed"
              : isFavorited
                ? "bg-[#FDCB6E]/15 text-[#FDCB6E]"
                : "bg-[#1E1E2E] text-[#636E72] hover:bg-[#2A2A3F] hover:text-[#DFE6E9]"
          }`}
        >
          {isFavorited ? "\u2665" : "\u2661"}
        </button>

        <button
          onClick={handleExport}
          disabled={isLoading}
          className="flex items-center gap-1.5 rounded-lg bg-[#1E1E2E] px-3 py-1.5 text-xs font-mono text-[#636E72] hover:bg-[#2A2A3F] hover:text-[#DFE6E9] transition-colors"
        >
          <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
          </svg>
          Export
        </button>
      </div>
    </div>
  );
}
