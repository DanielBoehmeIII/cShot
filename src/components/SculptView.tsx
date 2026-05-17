import { useState, useCallback } from "react";
import { sculptSound, type SculptControls, type SoundResult } from "../lib/api";

interface SculptViewProps {
  soundId: string | null;
  onResult: (result: SoundResult) => void;
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function SculptView({ soundId, onResult, onError, onSuccess }: SculptViewProps) {
  const [controls, setControls] = useState<SculptControls>({
    transient_intensity: 0.5, tail_length: 0.5, brightness: 0.5,
    distortion: 0.0, density: 0.5, tonal_noise_balance: 0.5,
    sub_amount: 0.3, body_thickness: 0.5, attack_sharpness: 0.5, stereo_width: 0.0,
  });
  const [isApplying, setIsApplying] = useState(false);
  const [activeSection, setActiveSection] = useState<string | null>(null);

  const updateControl = useCallback((key: keyof SculptControls, value: number) => {
    setControls(prev => ({ ...prev, [key]: value }));
  }, []);

  const handleApply = useCallback(async () => {
    if (!soundId) return;
    setIsApplying(true);
    try {
      const result = await sculptSound(soundId, controls);
      onResult(result);
      onSuccess("Sculpting applied");
    } catch (e) {
      onError(`Sculpting failed: ${e}`);
    } finally {
      setIsApplying(false);
    }
  }, [soundId, controls, onResult, onError, onSuccess]);

  const SLIDERS: Array<{
    key: keyof SculptControls; label: string; section: string;
    min: number; max: number; step: number;
  }> = [
    { key: "transient_intensity", label: "Transient Intensity", section: "Transient", min: 0, max: 1, step: 0.05 },
    { key: "attack_sharpness", label: "Attack Sharpness", section: "Transient", min: 0, max: 1, step: 0.05 },
    { key: "tail_length", label: "Tail Length", section: "Envelope", min: 0, max: 1, step: 0.05 },
    { key: "density", label: "Density", section: "Envelope", min: 0, max: 1, step: 0.05 },
    { key: "brightness", label: "Brightness", section: "Tone", min: 0, max: 1, step: 0.05 },
    { key: "body_thickness", label: "Body Thickness", section: "Tone", min: 0, max: 1, step: 0.05 },
    { key: "distortion", label: "Distortion", section: "Character", min: 0, max: 1, step: 0.05 },
    { key: "tonal_noise_balance", label: "Tonal/Noise Balance", section: "Character", min: 0, max: 1, step: 0.05 },
    { key: "sub_amount", label: "Sub Amount", section: "Low End", min: 0, max: 1, step: 0.05 },
    { key: "stereo_width", label: "Stereo Width", section: "Spatial", min: 0, max: 1, step: 0.05 },
  ];

  const sections = [...new Set(SLIDERS.map(s => s.section))];

  if (!soundId) return null;

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Sound Sculpting
        </h3>
      </div>

      <div className="p-4 space-y-3">
        <div className="flex gap-1.5">
          {sections.map(s => (
            <button key={s}
              onClick={() => setActiveSection(activeSection === s ? null : s)}
              className={`rounded px-2 py-1 text-[9px] font-mono transition-all ${
                activeSection === s
                  ? "bg-[#6C5CE7]/20 text-[#6C5CE7] border border-[#6C5CE7]/30"
                  : "bg-[#1E1E2E] text-[#636E72] border border-[#2A2A3F] hover:border-[#3A3A5F]"
              }`}
            >
              {s}
            </button>
          ))}
        </div>

        <div className="space-y-2 max-h-48 overflow-y-auto">
          {SLIDERS.filter(s => !activeSection || s.section === activeSection).map(({ key, label, min, max, step }) => (
            <div key={key}>
              <div className="flex justify-between">
                <label className="text-[9px] font-mono text-[#636E72]">{label}</label>
                <span className="text-[9px] text-[#4A4A6F] font-mono tabular-nums">
                  {controls[key].toFixed(2)}
                </span>
              </div>
              <input type="range" min={min} max={max} step={step} value={controls[key]}
                onChange={(e) => updateControl(key, parseFloat(e.target.value))}
                className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                  [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                  [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
              />
            </div>
          ))}
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleApply}
            disabled={isApplying || !soundId}
            className="flex-1 rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
              transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
              shadow-[0_0_12px_rgba(108,92,231,0.15)]"
          >
            {isApplying ? "Applying..." : "Apply Sculpt →"}
          </button>
        </div>
      </div>
    </div>
  );
}
