import { useState, useCallback } from "react";
import { recreateAdvanced, type AdvancedRecreationConfig, type SoundResult } from "../lib/api";

interface RecreationViewProps {
  soundId: string | null;
  onResult: (result: SoundResult) => void;
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

const MODES = [
  { value: "Closest", label: "Closest", desc: "Highest fidelity to original", icon: "🎯" },
  { value: "Cleaner", label: "Cleaner", desc: "Reduce noise & saturation", icon: "✨" },
  { value: "Harder", label: "Harder", desc: "More aggressive & punchy", icon: "💥" },
  { value: "MoreModern", label: "More Modern", desc: "Tighter, cleaner, brighter", icon: "⚡" },
  { value: "MoreAnalog", label: "More Analog", desc: "Warm saturation & drift", icon: "📻" },
];

const MODE_META: Record<string, { short: string; color: string }> = {
  Closest: { short: "CL", color: "#6C5CE7" },
  Cleaner: { short: "CLN", color: "#00D2D3" },
  Harder: { short: "HD", color: "#D63031" },
  MoreModern: { short: "MD", color: "#FDCB6E" },
  MoreAnalog: { short: "AN", color: "#E17055" },
};

export function RecreationView({ soundId, onResult, onError, onSuccess }: RecreationViewProps) {
  const [mode, setMode] = useState("Closest");
  const [fidelity, setFidelity] = useState(0.7);
  const [preserveTransient, setPreserveTransient] = useState(true);
  const [preserveBody, setPreserveBody] = useState(true);
  const [preserveTail, setPreserveTail] = useState(false);
  const [subReconstruction, setSubReconstruction] = useState(0.5);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleRecreate = useCallback(async () => {
    if (!soundId) return;
    setIsGenerating(true);
    try {
      const config: AdvancedRecreationConfig = {
        mode, fidelity,
        transient_preservation: preserveTransient ? 0.8 : 0.0,
        body_preservation: preserveBody ? 0.6 : 0.0,
        tail_preservation: preserveTail ? 0.5 : 0.0,
        spectral_matching: fidelity,
        sub_reconstruction: subReconstruction,
        transient_timing_align: true,
        harmonic_profile_match: true,
        tail_texture_match: mode === "MoreAnalog",
      };
      const [result] = await recreateAdvanced(soundId, config);
      onResult(result);
      onSuccess(`${MODES.find(m => m.value === mode)?.label} recreation complete`);
    } catch (e) {
      onError(`Recreation failed: ${e}`);
    } finally {
      setIsGenerating(false);
    }
  }, [soundId, mode, fidelity, preserveTransient, preserveBody, preserveTail, subReconstruction, onResult, onError, onSuccess]);

  if (!soundId) return null;

  const meta = MODE_META[mode] || MODE_META.Closest;

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Advanced Recreation
        </h3>
      </div>

      <div className="p-4 space-y-3">
        <div className="grid grid-cols-5 gap-1.5">
          {MODES.map((m) => (
            <button key={m.value}
              onClick={() => setMode(m.value)}
              className={`rounded-lg border p-2 text-center transition-all ${
                mode === m.value
                  ? "bg-[#6C5CE7]/10 border-[#6C5CE7]/40"
                  : "bg-[#1E1E2E] border-[#2A2A3F] hover:border-[#3A3A5F]"
              }`}
            >
              <span className="text-sm">{m.icon}</span>
              <p className={`text-[8px] font-mono mt-0.5 ${
                mode === m.value ? "text-[#6C5CE7]" : "text-[#636E72]"
              }`}>
                {m.label}
              </p>
            </button>
          ))}
        </div>

        <div className="flex items-center justify-between rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2">
          <div>
            <p className={`text-[11px] font-mono font-medium`} style={{ color: meta.color }}>
              {meta.short}
            </p>
            <p className="text-[9px] text-[#636E72] font-mono">
              {MODES.find(m => m.value === mode)?.desc}
            </p>
          </div>
        </div>

        <div>
          <label className="text-[9px] font-mono text-[#636E72]">Fidelity: {(fidelity * 100).toFixed(0)}%</label>
          <input type="range" min="0.1" max="1" step="0.05" value={fidelity}
            onChange={(e) => setFidelity(parseFloat(e.target.value))}
            className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
              [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
              [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
          />
        </div>

        <details className="text-[9px] font-mono text-[#636E72]">
          <summary className="cursor-pointer hover:text-[#DFE6E9]">Preserve Regions</summary>
          <div className="mt-2 space-y-1.5">
            {[
              ["Transient", preserveTransient, setPreserveTransient],
              ["Body", preserveBody, setPreserveBody],
              ["Tail", preserveTail, setPreserveTail],
            ].map(([label, checked, setter]) => (
              <label key={label as string} className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" checked={checked as boolean}
                  onChange={(e) => (setter as (v: boolean) => void)(e.target.checked)}
                  className="w-3 h-3 accent-[#6C5CE7]" />
                <span className="capitalize">{label as string}</span>
              </label>
            ))}
          </div>
          <div className="mt-2">
            <label className="text-[8px] text-[#4A4A6F] font-mono">
              Sub Reconstruction: {(subReconstruction * 100).toFixed(0)}%
            </label>
            <input type="range" min="0" max="1" step="0.05" value={subReconstruction}
              onChange={(e) => setSubReconstruction(parseFloat(e.target.value))}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
        </details>

        <button
          onClick={handleRecreate}
          disabled={isGenerating}
          className="w-full rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
            transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
            shadow-[0_0_12px_rgba(108,92,231,0.15)]"
        >
          {isGenerating ? "Recreating..." : "Recreate →"}
        </button>
      </div>
    </div>
  );
}
