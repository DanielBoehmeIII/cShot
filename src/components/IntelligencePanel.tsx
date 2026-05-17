import { useState, useCallback } from "react";
import { analyzeAudioIntelligence } from "../lib/api";

interface IntelligencePanelProps {
  soundId: string | null;
  onError: (msg: string) => void;
}

const ROLE_ICONS: Record<string, string> = {
  KickDrum: "🥁", SnareDrum: "🔄", ClosedHat: "🔔", OpenHat: "🔔",
  Clap: "👏", Tom: "🪘", Percussion: "🥁", BassHit: "🎸",
  FxSound: "✨", Vocal: "🎤", Synth: "🎹", Texture: "🌊",
  Hybrid: "🔀", Unknown: "❓",
};

const MIX_ROLE_COLORS: Record<string, string> = {
  Foundation: "#6C5CE7", Backbone: "#00D2D3", Groove: "#FDCB6E",
  Accent: "#E17055", Fill: "#00B894", Texture: "#A29BFE",
  Effect: "#D63031", TonalElement: "#55EFC4", Unknown: "#636E72",
};

export function IntelligencePanel({ soundId, onError }: IntelligencePanelProps) {
  const [loading, setLoading] = useState(false);
  const [report, setReport] = useState<any>(null);

  const handleAnalyze = useCallback(async () => {
    if (!soundId) return;
    setLoading(true);
    try {
      const result = await analyzeAudioIntelligence(soundId);
      setReport(result);
    } catch (e) {
      onError(`Analysis failed: ${e}`);
    } finally {
      setLoading(false);
    }
  }, [soundId, onError]);

  if (!soundId) return null;

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50 flex items-center justify-between">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Audio Intelligence
        </h3>
        <button
          onClick={handleAnalyze}
          disabled={loading}
          className="rounded border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] transition-colors disabled:opacity-30"
        >
          {loading ? "..." : "Analyze"}
        </button>
      </div>

      {report && (
        <div className="p-4 space-y-3 fade-slide-up">
          <div className="flex items-center gap-3">
            <span className="text-xl">{ROLE_ICONS[report.genre_role] || "❓"}</span>
            <div>
              <p className="text-[10px] font-mono text-[#DFE6E9]">{report.genre_role}</p>
              <p className="text-[8px] text-[#636E72] font-mono">
                Confidence: {(report.confidence * 100).toFixed(0)}%
              </p>
            </div>
            <div className="ml-auto text-right">
              <span className="text-[8px] text-[#4A4A6F] font-mono">Mix Role</span>
              <p className="text-[9px] font-mono" style={{ color: MIX_ROLE_COLORS[report.mix_role] || "#636E72" }}>
                {report.mix_role}
              </p>
            </div>
          </div>

          <div className="h-px bg-[#2A2A3F]/50" />

          <div className="grid grid-cols-3 gap-2">
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 text-center">
              <p className="text-[8px] text-[#4A4A6F] font-mono">Impact</p>
              <p className="text-sm text-[#6C5CE7] font-mono tabular-nums">
                {(report.impact_prediction?.impact_score * 100).toFixed(0)}%
              </p>
            </div>
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 text-center">
              <p className="text-[8px] text-[#4A4A6F] font-mono">Punch</p>
              <p className="text-sm text-[#00D2D3] font-mono tabular-nums">
                {(report.impact_prediction?.punch_factor * 100).toFixed(0)}%
              </p>
            </div>
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 text-center">
              <p className="text-[8px] text-[#4A4A6F] font-mono">Weight</p>
              <p className="text-sm text-[#FDCB6E] font-mono tabular-nums">
                {(report.impact_prediction?.weight * 100).toFixed(0)}%
              </p>
            </div>
          </div>

          <div className="h-px bg-[#2A2A3F]/50" />

          <div className="grid grid-cols-2 gap-3">
            <div>
              <p className="text-[9px] text-[#636E72] font-mono mb-1">Transient</p>
              <div className="space-y-1">
                <InfoRow label="Type" value={report.transient_classification?.primary_type} />
                <InfoRow label="Count" value={report.transient_classification?.transient_count} />
                <InfoRow label="Sharpness" value={`${(report.transient_classification?.attack_sharpness * 100).toFixed(0)}%`} />
                <InfoRow label="Multi-hit" value={report.transient_classification?.has_multiple_hits ? "Yes" : "No"} />
              </div>
            </div>
            <div>
              <p className="text-[9px] text-[#636E72] font-mono mb-1">Spectral</p>
              <div className="space-y-1">
                <InfoRow label="Tonal" value={`${(report.tonal_noise_decomposition?.tonal_ratio * 100).toFixed(0)}%`} />
                <InfoRow label="Noise" value={`${(report.tonal_noise_decomposition?.noise_ratio * 100).toFixed(0)}%`} />
                <InfoRow label="Complexity" value={`${(report.spectral_complexity?.complexity_score * 100).toFixed(0)}%`} />
                <InfoRow label="Flatness" value={`${(report.spectral_complexity?.spectral_flatness * 100).toFixed(0)}%`} />
              </div>
            </div>
          </div>
        </div>
      )}

      {!report && (
        <div className="p-4 text-center">
          <p className="text-[9px] text-[#636E72] font-mono">Click Analyze to see the intelligence report</p>
        </div>
      )}
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: any }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[8px] text-[#4A4A6F] font-mono">{label}</span>
      <span className="text-[9px] text-[#DFE6E9] font-mono">{String(value ?? "-")}</span>
    </div>
  );
}
