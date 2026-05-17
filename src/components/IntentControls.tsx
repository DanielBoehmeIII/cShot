import { useState, useEffect, useCallback, useMemo } from "react";
import {
  type CreativeIntentProfile,
  getIntentPresets,
} from "../lib/api";

interface IntentControlsProps {
  profile: CreativeIntentProfile;
  onProfileChange: (profile: CreativeIntentProfile) => void;
  onGenerateWithIntent: () => void;
  isGenerating: boolean;
  isEnabled: boolean;
  onToggle: () => void;
}

const INTENT_LABELS: Record<keyof CreativeIntentProfile, { label: string; short: string; desc: string }> = {
  energy: { label: "Energy", short: "EN", desc: "Low → High" },
  aggression: { label: "Aggression", short: "AG", desc: "Soft → Aggressive" },
  polish: { label: "Polish", short: "PO", desc: "Raw → Polished" },
  realism: { label: "Realism", short: "RE", desc: "Synthetic → Natural" },
  experimentalism: { label: "Experimental", short: "EX", desc: "Safe → Experimental" },
  analog_feel: { label: "Analog Feel", short: "AN", desc: "Digital → Analog" },
  cinematic_scale: { label: "Cinematic Scale", short: "CI", desc: "Intimate → Cinematic" },
  density: { label: "Density", short: "DE", desc: "Sparse → Dense" },
  impact: { label: "Impact", short: "IM", desc: "Light → Heavy" },
};

const PROFILE_KEYS = Object.keys(INTENT_LABELS) as Array<keyof CreativeIntentProfile>;

function IntentSlider({
  name, value, onChange, label, short, desc,
}: {
  name: keyof CreativeIntentProfile; value: number; onChange: (name: keyof CreativeIntentProfile, value: number) => void;
  label: string; short: string; desc: string;
}) {
  const label_low = desc.split(" → ")[0] || "low";
  const label_high = desc.split(" → ")[1] || "high";
  return (
    <div className="flex items-center gap-2 group">
      <span className="w-6 text-[9px] font-mono text-[#4A4A6F] text-center shrink-0">{short}</span>
      <div className="flex-1 flex flex-col gap-0.5">
        <div className="flex justify-between px-0.5">
          <span className="text-[8px] text-[#4A4A6F] font-mono">{label_low}</span>
          <span className="text-[8px] text-[#6C5CE7] font-mono">{label}</span>
          <span className="text-[8px] text-[#4A4A6F] font-mono">{label_high}</span>
        </div>
        <input
          type="range" min="0" max="1" step="0.05" value={value}
          onChange={(e) => onChange(name, parseFloat(e.target.value))}
          className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
            [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
            [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]
            [&::-webkit-slider-thumb]:shadow-[0_0_6px_rgba(108,92,231,0.5)]
            [&::-webkit-slider-thumb]:transition-all [&::-webkit-slider-thumb]:hover:scale-125"
        />
      </div>
      <span className="w-8 text-right text-[10px] font-mono text-[#636E72] tabular-nums">
        {(value * 100).toFixed(0)}
      </span>
    </div>
  );
}

function IntentRadar({ profile, size = 120 }: { profile: CreativeIntentProfile; size?: number }) {
  const cx = size / 2;
  const cy = size / 2;
  const r = size / 2 - 12;
  const angleStep = (2 * Math.PI) / PROFILE_KEYS.length;

  const points = useMemo(() => {
    return PROFILE_KEYS.map((key, i) => {
      const angle = -Math.PI / 2 + i * angleStep;
      const val = profile[key];
      const x = cx + r * val * Math.cos(angle);
      const y = cy + r * val * Math.sin(angle);
      return `${x},${y}`;
    }).join(" ");
  }, [profile]);

  const gridLines = [0.25, 0.5, 0.75, 1.0];
  const labels = PROFILE_KEYS.map((key, i) => {
    const angle = -Math.PI / 2 + i * angleStep;
    const lx = cx + (r + 14) * Math.cos(angle);
    const ly = cy + (r + 14) * Math.sin(angle);
    return { short: INTENT_LABELS[key].short, x: lx, y: ly, key };
  });

  return (
    <svg width={size} height={size} className="shrink-0">
      {gridLines.map((g) => (
        <polygon
          key={g}
          points={PROFILE_KEYS.map((_, i) => {
            const angle = -Math.PI / 2 + i * angleStep;
            const x = cx + r * g * Math.cos(angle);
            const y = cy + r * g * Math.sin(angle);
            return `${x},${y}`;
          }).join(" ")}
          fill="none" stroke="#2A2A3F" strokeWidth="0.5"
        />
      ))}
      <polygon points={points} fill="rgba(108,92,231,0.15)" stroke="#6C5CE7" strokeWidth="1.5" />
      {labels.map(({ short, x, y }) => (
        <text key={short} x={x} y={y} textAnchor="middle" dominantBaseline="middle"
          fill="#4A4A6F" fontSize="8" fontFamily="monospace">
          {short}
        </text>
      ))}
    </svg>
  );
}

export function IntentControls({
  profile, onProfileChange, onGenerateWithIntent, isGenerating, isEnabled, onToggle,
}: IntentControlsProps) {
  const [presets, setPresets] = useState<[string, CreativeIntentProfile][]>([]);
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const [searchPreset, setSearchPreset] = useState("");

  useEffect(() => {
    getIntentPresets().then(setPresets).catch(() => {});
  }, []);

  const handleSliderChange = useCallback(
    (name: keyof CreativeIntentProfile, value: number) => {
      onProfileChange({ ...profile, [name]: value });
      setActivePreset(null);
    },
    [profile, onProfileChange],
  );

  const applyPreset = useCallback(
    (name: string, preset: CreativeIntentProfile) => {
      onProfileChange(preset);
      setActivePreset(name);
    },
    [onProfileChange],
  );

  const average = useMemo(() =>
    Object.values(profile).reduce((a, b) => a + b, 0) / PROFILE_KEYS.length,
  [profile]);

  const filteredPresets = useMemo(() => {
    if (!searchPreset.trim()) return presets;
    const q = searchPreset.toLowerCase();
    return presets.filter(([name]) => name.replace(/_/g, " ").toLowerCase().includes(q));
  }, [presets, searchPreset]);

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden">
      <button
        onClick={onToggle}
        className={`w-full flex items-center justify-between px-4 py-2.5 transition-colors ${
          isEnabled ? "bg-[#6C5CE7]/10" : "hover:bg-[#1E1E2E]"
        }`}
      >
        <div className="flex items-center gap-2">
          <span className={`text-[10px] font-mono font-medium uppercase tracking-wider ${
            isEnabled ? "text-[#6C5CE7]" : "text-[#636E72]"
          }`}>
            Creative Intent
          </span>
          <span className={`text-[9px] font-mono tabular-nums ${
            isEnabled ? "text-[#6C5CE7]" : "text-[#2A2A3F]"
          }`}>
            {(average * 100).toFixed(0)}%
          </span>
        </div>
        <div className={`w-3 h-3 rounded-full border transition-colors ${
          isEnabled ? "bg-[#6C5CE7] border-[#6C5CE7]" : "bg-transparent border-[#2A2A3F]"
        }`} />
      </button>

      {isEnabled && (
        <div className="px-4 pb-4 space-y-3 fade-slide-up">
          <div className="h-px bg-[#2A2A3F]/50" />

          <div className="flex items-center gap-3">
            <IntentRadar profile={profile} size={100} />
            <div className="flex-1 space-y-1">
              <p className="text-[9px] font-mono text-[#636E72] uppercase tracking-wider">Intent Profile</p>
              {PROFILE_KEYS.map((key) => (
                <div key={key} className="flex items-center gap-1.5">
                  <span className="text-[8px] text-[#4A4A6F] font-mono w-4">{INTENT_LABELS[key].short}</span>
                  <div className="flex-1 h-1.5 bg-[#2A2A3F] rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-150"
                      style={{
                        width: `${profile[key] * 100}%`,
                        backgroundColor: profile[key] > 0.6 ? "#6C5CE7" : profile[key] > 0.3 ? "#8854D0" : "#4A4A6F",
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="h-px bg-[#2A2A3F]/50" />

          <div>
            <input
              type="text"
              value={searchPreset}
              onChange={(e) => setSearchPreset(e.target.value)}
              placeholder="Search presets..."
              className="w-full rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-1.5 text-[10px] font-mono text-[#DFE6E9] placeholder:text-[#636E72] outline-none focus:border-[#6C5CE7]/50 transition-colors mb-2"
            />
            <div className="flex flex-wrap gap-1">
              {filteredPresets.map(([name, preset]) => (
                <button
                  key={name}
                  onClick={() => applyPreset(name, preset)}
                  className={`rounded px-2 py-0.5 text-[9px] font-mono transition-all ${
                    activePreset === name
                      ? "bg-[#6C5CE7]/20 text-[#6C5CE7] border border-[#6C5CE7]/30"
                      : "bg-[#1E1E2E] text-[#636E72] border border-[#2A2A3F] hover:border-[#3A3A5F] hover:text-[#DFE6E9]"
                  }`}
                >
                  {name.replace(/_/g, " ")}
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-1">
            {PROFILE_KEYS.map((key) => (
              <IntentSlider
                key={key} name={key} value={profile[key]}
                onChange={handleSliderChange} {...INTENT_LABELS[key]}
              />
            ))}
          </div>

          <button
            onClick={onGenerateWithIntent}
            disabled={isGenerating}
            className="w-full rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
              transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
              shadow-[0_0_12px_rgba(108,92,231,0.15)]"
          >
            {isGenerating ? "Generating..." : "Generate with Intent →"}
          </button>

          <div className="flex items-center gap-2 text-[8px] text-[#2A2A3F] font-mono">
            <span className="w-2 h-2 rounded-full bg-[#6C5CE7]/30" />
            Intent profile maps to brightness, saturation, attack, sub, noise & duration
          </div>
        </div>
      )}
    </div>
  );
}
