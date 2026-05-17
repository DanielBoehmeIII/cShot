import { useState, useCallback, useEffect, useRef } from "react";
import { parseNaturalLanguageQuery, searchSounds, type SemanticQuery, type SoundEntry } from "../lib/api";

interface SemanticSearchProps {
  onResults: (sounds: SoundEntry[], query: string) => void;
  onError: (msg: string) => void;
}

const EXAMPLE_QUERIES = [
  "hard dark trap kick",
  "short clean cinematic hit",
  "warm analog clap",
  "metallic UI click",
  "bright snare with crack",
  "deep sub bass 808",
  "punchy kick 140bpm",
  "airymetallic open hat",
];

export function SemanticSearch({ onResults, onError }: SemanticSearchProps) {
  const [query, setQuery] = useState("");
  const [parsed, setParsed] = useState<SemanticQuery | null>(null);
  const [isSearching, setIsSearching] = useState(false);
  const [showParsed, setShowParsed] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleSearch = useCallback(async (q: string) => {
    if (!q.trim()) {
      onResults([], "");
      setParsed(null);
      return;
    }
    setIsSearching(true);
    try {
      const parsedQuery = await parseNaturalLanguageQuery(q);
      setParsed(parsedQuery);
      const results = await searchSounds(q);
      onResults(results, q);
    } catch (e) {
      onError(`Search failed: ${e}`);
    } finally {
      setIsSearching(false);
    }
  }, [onResults, onError]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      handleSearch(query);
    }, 200);
    return () => { if (debounceRef.current) clearTimeout(debounceRef.current); };
  }, [query, handleSearch]);

  const handleQuickQuery = useCallback((q: string) => {
    setQuery(q);
  }, []);

  return (
    <div className="space-y-2">
      <div className="relative">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => setShowParsed(true)}
          onBlur={() => setTimeout(() => setShowParsed(false), 300)}
          placeholder='e.g. "hard dark trap kick" or "warm analog clap"'
          className="w-full rounded-lg border border-[#2A2A3F] bg-[#14141F] px-3 py-2 text-xs font-mono text-[#DFE6E9] placeholder:text-[#636E72] outline-none focus:border-[#6C5CE7]/50 transition-colors pr-8"
        />
        {isSearching && (
          <div className="absolute right-2 top-1/2 -translate-y-1/2">
            <div className="spinner w-3.5 h-3.5 rounded-full border-2 border-[#2A2A3F] border-t-[#6C5CE7]" />
          </div>
        )}
        {query && !isSearching && (
          <button
            onClick={() => { setQuery(""); onResults([], ""); }}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-[#636E72] hover:text-[#DFE6E9]"
          >
            <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>

      {showParsed && parsed && (
        <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 fade-slide-up">
          <p className="text-[8px] text-[#4A4A6F] font-mono uppercase tracking-wider mb-1">Parsed Query</p>
          <div className="flex flex-wrap gap-1">
            {parsed.target_type && (
              <span className="rounded bg-[#6C5CE7]/10 px-1.5 py-0.5 text-[9px] text-[#6C5CE7] font-mono">
                type: {parsed.target_type}
              </span>
            )}
            {parsed.target_descriptors.map((d) => (
              <span key={d} className="rounded bg-[#00D2D3]/10 px-1.5 py-0.5 text-[9px] text-[#00D2D3] font-mono">
                {d}
              </span>
            ))}
            {parsed.target_genre && (
              <span className="rounded bg-[#FDCB6E]/10 px-1.5 py-0.5 text-[9px] text-[#FDCB6E] font-mono">
                genre: {parsed.target_genre}
              </span>
            )}
            {parsed.bpm && (
              <span className="rounded bg-[#E17055]/10 px-1.5 py-0.5 text-[9px] text-[#E17055] font-mono">
                {parsed.bpm}bpm
              </span>
            )}
          </div>
        </div>
      )}

      {!query && (
        <div className="flex flex-wrap gap-1.5">
          {EXAMPLE_QUERIES.map((q) => (
            <button key={q}
              onClick={() => handleQuickQuery(q)}
              className="rounded-full border border-[#2A2A3F] bg-[#14141F] px-2.5 py-1 text-[9px] text-[#636E72] font-mono transition-colors hover:border-[#6C5CE7]/50 hover:text-[#6C5CE7]"
            >
              {q}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
