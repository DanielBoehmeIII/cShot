import type { ReferenceAnalysis } from "../lib/api";

interface ReferenceCardProps {
  analysis: ReferenceAnalysis;
  isPlaying: boolean;
  onPlay: () => void;
  onClear: () => void;
}

export function ReferenceCard({
  analysis,
  isPlaying,
  onPlay,
  onClear,
}: ReferenceCardProps) {
  const svgWidth = 240;
  const svgHeight = 48;
  const centerY = svgHeight / 2;

  const pathD = analysis.waveform
    .map((val, i) => {
      const x = (i / (analysis.waveform.length - 1)) * svgWidth;
      const y = centerY - val * (centerY - 2);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  const pathD2 = analysis.waveform
    .map((val, i) => {
      const x = (i / (analysis.waveform.length - 1)) * svgWidth;
      const y = centerY + val * (centerY - 2);
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  return (
    <div className="rounded-xl border border-[#00D2D3]/20 bg-[#14141F] p-4 fade-slide-up">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 min-w-0">
          <span className="rounded-md bg-[#00D2D3]/10 px-2 py-0.5 text-[10px] font-mono font-medium uppercase tracking-wider text-[#00D2D3] shrink-0">
            Reference
          </span>
          <span className="text-xs text-[#636E72] truncate">
            {analysis.filename}
          </span>
        </div>
        <button
          onClick={onClear}
          className="text-[10px] text-[#636E72] hover:text-[#DFE6E9] shrink-0 ml-2"
        >
          clear
        </button>
      </div>

      <button
        onClick={onPlay}
        className="relative mb-3 w-full cursor-pointer group"
      >
        <svg
          viewBox={`0 0 ${svgWidth} ${svgHeight}`}
          className={`w-full h-12 transition-opacity ${
            isPlaying ? "opacity-100" : "opacity-60 group-hover:opacity-100"
          }`}
        >
          <path
            d={pathD}
            fill="none"
            stroke="#00D2D3"
            strokeWidth="1.5"
            opacity="0.7"
          />
          <path
            d={pathD2}
            fill="none"
            stroke="#00D2D3"
            strokeWidth="1.5"
            opacity="0.7"
          />
          {isPlaying && (
            <rect x={0} y={0} width={4} height={svgHeight} fill="#00D2D3">
              <animate
                attributeName="x"
                values={`0;${svgWidth - 4};0`}
                dur="2s"
                repeatCount="indefinite"
              />
            </rect>
          )}
        </svg>
        <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
          <div className="rounded-full bg-black/70 p-2 backdrop-blur-sm">
            {isPlaying ? (
              <svg
                className="w-5 h-5 text-white"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <rect x="6" y="4" width="4" height="16" />
                <rect x="14" y="4" width="4" height="16" />
              </svg>
            ) : (
              <svg
                className="w-5 h-5 text-white"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <polygon points="6,4 20,12 6,20" />
              </svg>
            )}
          </div>
        </div>
      </button>

      <div className="grid grid-cols-4 gap-3 text-center">
        <div>
          <div className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider">
            Duration
          </div>
          <div className="text-xs text-[#DFE6E9] mt-0.5 font-mono">
            {analysis.duration_ms < 1000
              ? `${Math.round(analysis.duration_ms)}ms`
              : `${(analysis.duration_ms / 1000).toFixed(1)}s`}
          </div>
        </div>
        <div>
          <div className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider">
            Rate
          </div>
          <div className="text-xs text-[#DFE6E9] mt-0.5 font-mono">
            {analysis.sample_rate / 1000}kHz
          </div>
        </div>
        <div>
          <div className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider">
            Channels
          </div>
          <div className="text-xs text-[#DFE6E9] mt-0.5 font-mono">
            {analysis.channels === 1
              ? "Mono"
              : analysis.channels === 2
                ? "Stereo"
                : `${analysis.channels}`}
          </div>
        </div>
        <div>
          <div className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider">
            Format
          </div>
          <div className="text-xs text-[#DFE6E9] mt-0.5 font-mono uppercase">
            {analysis.file_type}
          </div>
        </div>
      </div>
    </div>
  );
}
