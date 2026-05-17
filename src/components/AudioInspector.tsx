import { useState, useEffect, useCallback } from "react";
import type { AudioAnalysis } from "../lib/api";
import { getAudioAnalysis } from "../lib/api";

interface AudioInspectorProps {
  soundId: string;
  isVisible: boolean;
  onClose: () => void;
}

function formatMs(ms: number): string {
  if (ms >= 1000) return `${(ms / 1000).toFixed(2)}s`;
  return `${Math.round(ms)}ms`;
}

function formatDb(val: number): string {
  return `${val.toFixed(1)} dB`;
}

function formatFloat(val: number): string {
  return val.toFixed(3);
}

export function AudioInspector({ soundId, isVisible, onClose }: AudioInspectorProps) {
  const [analysis, setAnalysis] = useState<AudioAnalysis | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(["level", "temporal"]));

  useEffect(() => {
    if (!isVisible) return;
    setIsLoading(true);
    getAudioAnalysis(soundId)
      .then(setAnalysis)
      .catch(() => setAnalysis(null))
      .finally(() => setIsLoading(false));
  }, [soundId, isVisible]);

  const toggleSection = useCallback((section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) next.delete(section);
      else next.add(section);
      return next;
    });
  }, []);

  if (!isVisible) return null;

  return (
    <div className="mt-4 rounded-xl border border-[#2A2A3F]/50 bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-[#2A2A3F]/30">
        <span className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#636E72]">
          Audio Inspector
        </span>
        <button onClick={onClose} className="text-[#636E72] hover:text-[#DFE6E9] transition-colors">
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-8">
          <div className="spinner w-5 h-5 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
        </div>
      )}

      {!isLoading && !analysis && (
        <div className="px-4 py-6 text-center">
          <p className="text-[10px] text-[#636E72] font-mono">Analysis not available</p>
        </div>
      )}

      {!isLoading && analysis && (
        <div className="px-4 py-3 space-y-1 max-h-96 overflow-y-auto font-mono text-[10px] text-[#636E72]">
          {/* Sound type hint */}
          <div className="mb-2 px-2 py-1 rounded bg-[#1E1E2E] text-[#6C5CE7] text-[11px]">
            Detected: {analysis.sound_type_hint ?? "unknown"}
            {analysis.has_pitch && analysis.pitch_estimate && (
              <span className="text-[#636E72] ml-2">
                · ~{Math.round(analysis.pitch_estimate)}Hz
              </span>
            )}
          </div>

          {/* Level Section */}
          <Section title="Level" section="level" expanded={expandedSections.has("level")} onToggle={toggleSection}>
            <div className="space-y-1">
              <Metric label="Peak" value={`${formatDb(20.0 * Math.log10(analysis.peak + 1e-10))}`} />
              <Metric label="RMS" value={`${formatDb(20.0 * Math.log10(analysis.rms + 1e-10))}`} />
              <Metric label="Crest Factor" value={formatFloat(analysis.crest_factor)} />
              <Metric label="Loudness" value={`${analysis.loudness_lufs.toFixed(1)} LUFS`} />
              <Metric label="Noise Floor" value={formatDb(analysis.noise_floor_db)} />
              {analysis.has_clipping && (
                <div className="text-[#D63031] text-[10px]">CLIPPING ({analysis.clipping_count} samples)</div>
              )}
              {analysis.is_silent && <div className="text-[#D63031] text-[10px]">SILENT</div>}
            </div>
          </Section>

          {/* Temporal Section */}
          <Section title="Temporal" section="temporal" expanded={expandedSections.has("temporal")} onToggle={toggleSection}>
            <div className="space-y-1">
              <Metric label="Duration" value={formatMs(analysis.duration_ms)} />
              <Metric label="Attack" value={formatMs(analysis.attack_ms)} />
              <Metric label="Decay" value={formatMs(analysis.decay_ms)} />
              <Metric label="Tail" value={formatMs(analysis.tail_ms)} />
              {analysis.has_leading_silence && (
                <Metric label="Lead Silence" value={formatMs(analysis.leading_silence_ms)} />
              )}
              {analysis.has_trailing_silence && (
                <Metric label="Trail Silence" value={formatMs(analysis.trailing_silence_ms)} />
              )}
              {analysis.envelope.length > 0 && (
                <div className="mt-1">
                  <p className="text-[9px] text-[#2A2A3F] mb-1">Envelope</p>
                  <svg viewBox="0 0 {analysis.envelope.length} 32" className="w-full h-8">
                    <defs>
                      <linearGradient id="env-grad" x1="0" y1="0" x2="1" y2="0">
                        <stop offset="0%" stopColor="#6C5CE7" stopOpacity="0.6" />
                        <stop offset="100%" stopColor="#00D2D3" stopOpacity="0.6" />
                      </linearGradient>
                    </defs>
                    <path
                      d={analysis.envelope.map((v, i) => `${i === 0 ? "M" : "L"} ${i} ${32 - v * 32}`).join(" ")}
                      fill="none"
                      stroke="url(#env-grad)"
                      strokeWidth="1.5"
                    />
                  </svg>
                </div>
              )}
            </div>
          </Section>

          {/* Spectral Section */}
          <Section title="Spectral" section="spectral" expanded={expandedSections.has("spectral")} onToggle={toggleSection}>
            <div className="space-y-1">
              <Metric label="Centroid" value={`${Math.round(analysis.spectral_centroid)} Hz`} />
              <Metric label="Rolloff" value={`${Math.round(analysis.spectral_rolloff)} Hz`} />
              <Metric label="Brightness" value={formatFloat(analysis.brightness)} />
              <Metric label="ZCR" value={formatFloat(analysis.zero_crossing_rate)} />
              <Metric label="Sub Energy" value={formatFloat(analysis.sub_energy_ratio)} />
              <Metric label="Noise Estimate" value={formatFloat(analysis.noise_estimate)} />
              {analysis.spectral_profile.length > 0 && (
                <div className="mt-1">
                  <p className="text-[9px] text-[#2A2A3F] mb-1">Spectral Profile</p>
                  <svg viewBox="0 0 {analysis.spectral_profile.length} 24" className="w-full h-6">
                    <path
                      d={analysis.spectral_profile.map((v, i) => `${i === 0 ? "M" : "L"} ${i} ${24 - v * 24}`).join(" ")}
                      fill="none"
                      stroke="#6C5CE7"
                      strokeWidth="1"
                      opacity="0.6"
                    />
                  </svg>
                </div>
              )}
            </div>
          </Section>

          {/* Transient Section */}
          <Section title="Transient" section="transient" expanded={expandedSections.has("transient")} onToggle={toggleSection}>
            <div className="space-y-1">
              <Metric label="Strength" value={formatFloat(analysis.transient_strength)} />
              <Metric label="Onsets" value={`${analysis.transient_count}`} />
              {analysis.onset_times_ms.length > 0 && (
                <div className="text-[9px] text-[#2A2A3F]">
                  Onsets: {analysis.onset_times_ms.map((t) => formatMs(t)).join(", ")}
                </div>
              )}
            </div>
          </Section>

          <div className="pt-1 text-[9px] text-[#2A2A3F] text-center">
            {analysis.sample_rate}Hz · {analysis.channels}ch
          </div>
        </div>
      )}
    </div>
  );
}

function Section({ title, section, expanded, onToggle, children }: {
  title: string;
  section: string;
  expanded: boolean;
  onToggle: (s: string) => void;
  children: React.ReactNode;
}) {
  return (
    <div>
      <button
        onClick={() => onToggle(section)}
        className="flex items-center justify-between w-full px-2 py-1.5 rounded hover:bg-[#1E1E2E] transition-colors"
      >
        <span className="text-[10px] font-medium text-[#636E72] uppercase tracking-wider">{title}</span>
        <svg
          className={`w-3 h-3 text-[#636E72] transition-transform ${expanded ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>
      {expanded && <div className="px-3 py-1">{children}</div>}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[#636E72]">{label}</span>
      <span className="text-[#DFE6E9]">{value}</span>
    </div>
  );
}
