import { useState, useEffect, useCallback, useRef } from "react";
import {
  type OneShotControls,
  type SoundClassInfo,
  type OneshotPreviewResult,
  type OneshotExportResult,
  DEFAULT_ONESHOT_CONTROLS,
  listSoundClasses,
  getOneshotDefaults,
  renderOneshot,
  exportOneshotWav,
} from "../lib/api";

const CONTROL_META: { key: keyof OneShotControls; label: string; low: string; high: string }[] = [
  { key: "brightness", label: "Brightness", low: "dark", high: "bright" },
  { key: "punch", label: "Punch", low: "soft", high: "punchy" },
  { key: "decay", label: "Decay", low: "short", high: "long" },
  { key: "distortion", label: "Distortion", low: "clean", high: "distorted" },
  { key: "transient_amount", label: "Transient", low: "none", high: "max" },
  { key: "noise_amount", label: "Noise", low: "none", high: "max" },
  { key: "body_amount", label: "Body", low: "none", high: "max" },
  { key: "stereo_width", label: "Stereo Width", low: "mono", high: "wide" },
  { key: "pitch_drop", label: "Pitch Drop", low: "none", high: "max" },
  { key: "filter_sweep", label: "Filter Sweep", low: "none", high: "max" },
];

interface OneShotPanelProps {
  onPlay: (id: string, samples: number[], sampleRate: number) => void;
}

function ControlSlider({
  label, low, high, value, onChange,
}: {
  label: string; low: string; high: string; value: number; onChange: (v: number) => void;
}) {
  return (
    <div className="flex items-center gap-2 group">
      <span className="w-16 text-[9px] font-mono text-[#636E72] text-right shrink-0">{label}</span>
      <div className="flex-1 flex flex-col gap-0.5">
        <input
          type="range" min="0" max="1" step="0.02" value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
            [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
            [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]
            [&::-webkit-slider-thumb]:shadow-[0_0_6px_rgba(108,92,231,0.5)]
            [&::-webkit-slider-thumb]:transition-all [&::-webkit-slider-thumb]:hover:scale-125"
        />
        <div className="flex justify-between px-0.5">
          <span className="text-[7px] text-[#4A4A6F] font-mono">{low}</span>
          <span className="text-[7px] text-[#4A4A6F] font-mono">{high}</span>
        </div>
      </div>
      <span className="w-8 text-right text-[10px] font-mono text-[#636E72] tabular-nums">
        {(value * 100).toFixed(0)}
      </span>
    </div>
  );
}

export function OneShotPanel({ onPlay }: OneShotPanelProps) {
  const [classes, setClasses] = useState<SoundClassInfo[]>([]);
  const [selectedClass, setSelectedClass] = useState("808");
  const [durationMs, setDurationMs] = useState(800);
  const [pitchHz, setPitchHz] = useState(55);
  const [gain, setGain] = useState(1.0);
  const [controls, setControls] = useState<OneShotControls>({ ...DEFAULT_ONESHOT_CONTROLS });
  const [preview, setPreview] = useState<OneshotPreviewResult | null>(null);
  const [exportResult, setExportResult] = useState<OneshotExportResult | null>(null);
  const [isRendering, setIsRendering] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const previewIdRef = useRef(0);

  useEffect(() => {
    listSoundClasses().then(setClasses).catch(() => {});
  }, []);

  useEffect(() => {
    getOneshotDefaults(selectedClass).then((spec) => {
      setDurationMs(spec.duration_ms);
      setPitchHz(spec.pitch_hz);
      setGain(spec.gain);
    }).catch(() => {});
  }, [selectedClass]);

  const updateControl = useCallback((key: keyof OneShotControls, value: number) => {
    setControls((prev) => ({ ...prev, [key]: value }));
  }, []);

  const handleRandomize = useCallback(() => {
    const amt = 0.3;
    function rng(seed: number, idx: number): number {
      let h = Math.imul(seed, 0x5851F42D) ^ idx;
      h = Math.imul(h ^ (h >>> 16), 0x27D4EB2D);
      h = h ^ (h >>> 15);
      return ((h >>> 0) / 4294967296);
    }
    const seed = Date.now() & 0x7fffffff;
    const offset = (idx: number) => (rng(seed, idx) * 2 - 1) * amt;
    setControls((prev) => ({
      brightness: Math.max(0, Math.min(1, prev.brightness + offset(0))),
      punch: Math.max(0, Math.min(1, prev.punch + offset(1))),
      decay: Math.max(0, Math.min(1, prev.decay + offset(2))),
      distortion: Math.max(0, Math.min(1, prev.distortion + offset(3))),
      transient_amount: Math.max(0, Math.min(1, prev.transient_amount + offset(4))),
      noise_amount: Math.max(0, Math.min(1, prev.noise_amount + offset(5))),
      body_amount: Math.max(0, Math.min(1, prev.body_amount + offset(6))),
      stereo_width: Math.max(0, Math.min(1, prev.stereo_width + offset(7))),
      pitch_drop: Math.max(0, Math.min(1, prev.pitch_drop + offset(8))),
      filter_sweep: Math.max(0, Math.min(1, prev.filter_sweep + offset(9))),
    }));
    setStatusMessage("Controls randomized");
    setTimeout(() => setStatusMessage(null), 2000);
  }, []);

  const handlePreview = useCallback(async () => {
    setIsRendering(true);
    setError(null);
    setPreview(null);
    setStatusMessage("Rendering...");
    try {
      const controlsToSend = Object.values(controls).some((v) => v !== 0.5) ? controls : null;
      const result = await renderOneshot(selectedClass, durationMs, pitchHz, gain, controlsToSend);
      setPreview(result);
      setStatusMessage(`Rendered: ${result.duration_ms.toFixed(0)}ms · peak ${result.peak.toFixed(3)} · RMS ${result.rms.toFixed(5)}`);
      const id = `oneshot-${++previewIdRef.current}`;
      onPlay(id, result.samples, result.sample_rate);
    } catch (e) {
      setError(String(e));
      setStatusMessage(null);
    } finally {
      setIsRendering(false);
    }
  }, [selectedClass, durationMs, pitchHz, gain, controls, onPlay]);

  const handleExport = useCallback(async () => {
    setIsExporting(true);
    setError(null);
    setExportResult(null);
    setStatusMessage("Exporting...");
    try {
      const controlsToSend = Object.values(controls).some((v) => v !== 0.5) ? controls : null;
      const { open } = await import("@tauri-apps/plugin-dialog");
      const file = await open({
        filters: [{ name: "WAV Audio", extensions: ["wav"] }],
        multiple: false,
        defaultPath: `${selectedClass}.wav`,
        save: true,
      });
      if (!file) {
        setIsExporting(false);
        setStatusMessage(null);
        return;
      }
      const result = await exportOneshotWav(selectedClass, durationMs, pitchHz, gain, controlsToSend, file as string);
      setExportResult(result);
      setStatusMessage(`Exported: ${result.filename}`);
      setTimeout(() => { setExportResult(null); }, 5000);
    } catch (e) {
      setError(String(e));
      setStatusMessage(null);
    } finally {
      setIsExporting(false);
    }
  }, [selectedClass, durationMs, pitchHz, gain, controls]);

  const isDefaultControls = Object.values(controls).every((v) => v === 0.5);

  return (
    <div className="space-y-4 fade-slide-up">
      {/* Sound Class Selector */}
      <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] p-4">
        <div className="flex items-center gap-3 mb-3">
          <label className="text-[10px] font-mono font-medium text-[#636E72] uppercase tracking-wider">
            Sound Class
          </label>
          <select
            value={selectedClass}
            onChange={(e) => setSelectedClass(e.target.value)}
            className="flex-1 rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-1.5 text-xs font-mono
              text-[#DFE6E9] outline-none focus:border-[#6C5CE7]/50 transition-colors appearance-none
              cursor-pointer"
          >
            {classes.map((c) => (
              <option key={c.value} value={c.value}>{c.label}</option>
            ))}
          </select>
        </div>
        {classes.find((c) => c.value === selectedClass) && (
          <p className="text-[9px] text-[#4A4A6F] font-mono leading-relaxed">
            {classes.find((c) => c.value === selectedClass)!.description}
          </p>
        )}
      </div>

      {/* Duration, Pitch, Gain */}
      <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] p-4 space-y-2">
        <div className="flex items-center gap-3">
          <label className="text-[9px] font-mono text-[#636E72] w-16 text-right shrink-0">Duration</label>
          <input
            type="range" min="10" max="5000" step="10" value={durationMs}
            onChange={(e) => setDurationMs(parseInt(e.target.value))}
            className="flex-1 h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
              [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
              [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
          />
          <span className="w-16 text-right text-[10px] font-mono text-[#636E72] tabular-nums">{durationMs}ms</span>
        </div>
        <div className="flex items-center gap-3">
          <label className="text-[9px] font-mono text-[#636E72] w-16 text-right shrink-0">Pitch</label>
          <input
            type="range" min="20" max="1000" step="1" value={pitchHz}
            onChange={(e) => setPitchHz(parseInt(e.target.value))}
            className="flex-1 h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
              [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
              [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
          />
          <span className="w-16 text-right text-[10px] font-mono text-[#636E72] tabular-nums">{pitchHz}Hz</span>
        </div>
        <div className="flex items-center gap-3">
          <label className="text-[9px] font-mono text-[#636E72] w-16 text-right shrink-0">Gain</label>
          <input
            type="range" min="0" max="2" step="0.01" value={gain}
            onChange={(e) => setGain(parseFloat(e.target.value))}
            className="flex-1 h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
              [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
              [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
          />
          <span className="w-16 text-right text-[10px] font-mono text-[#636E72] tabular-nums">{gain.toFixed(2)}x</span>
        </div>
      </div>

      {/* Controls */}
      <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] p-4 space-y-2">
        <div className="flex items-center justify-between mb-2">
          <span className="text-[10px] font-mono font-medium text-[#636E72] uppercase tracking-wider">
            Controls
          </span>
          {!isDefaultControls && (
            <button
              onClick={() => setControls({ ...DEFAULT_ONESHOT_CONTROLS })}
              className="text-[9px] font-mono text-[#4A4A6F] hover:text-[#DFE6E9] transition-colors"
            >
              Reset
            </button>
          )}
        </div>
        {CONTROL_META.map(({ key, label, low, high }) => (
          <ControlSlider
            key={key} label={label} low={low} high={high}
            value={controls[key]} onChange={(v) => updateControl(key, v)}
          />
        ))}
        <div className="flex items-center gap-2 text-[8px] text-[#2A2A3F] font-mono pt-1">
          <span className="w-2 h-2 rounded-full bg-[#6C5CE7]/30" />
          0.5 = preserve preset · lower/higher = shape away from preset
        </div>
      </div>

      {/* Buttons */}
      <div className="flex gap-2">
        <button
          onClick={handleRandomize}
          className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-2 text-[10px] font-mono
            text-[#636E72] hover:border-[#8854D0]/50 hover:text-[#DFE6E9] transition-all shrink-0"
        >
          Randomize
        </button>
        <button
          onClick={handlePreview}
          disabled={isRendering}
          className="flex-1 rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
            transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
            shadow-[0_0_12px_rgba(108,92,231,0.15)]"
        >
          {isRendering ? "Rendering..." : preview ? "Preview ▶" : "Preview ▶"}
        </button>
        <button
          onClick={handleExport}
          disabled={isExporting}
          className="rounded-lg border border-[#00D2D3]/50 bg-[#00D2D3]/10 px-4 py-2 text-[10px] font-mono
            text-[#00D2D3] transition-all hover:bg-[#00D2D3]/20 disabled:opacity-30 disabled:cursor-not-allowed shrink-0"
        >
          {isExporting ? "..." : "Export WAV"}
        </button>
      </div>

      {/* Status / Error */}
      {statusMessage && (
        <div className="rounded-lg border border-[#6C5CE7]/20 bg-[#6C5CE7]/5 px-4 py-2">
          <p className="text-[10px] text-[#6C5CE7] font-mono">{statusMessage}</p>
        </div>
      )}
      {error && (
        <div className="rounded-lg border border-[#D63031]/40 bg-[#D63031]/10 px-4 py-2">
          <p className="text-[10px] text-[#D63031] font-mono break-all">{error}</p>
        </div>
      )}
      {exportResult && (
        <div className="rounded-lg border border-[#00D2D3]/30 bg-[#00D2D3]/10 px-4 py-2">
          <p className="text-[10px] text-[#00D2D3] font-mono break-all">
            Exported — {exportResult.filename} ({exportResult.file_size_bytes} bytes)
          </p>
          <p className="text-[8px] text-[#4A4A6F] font-mono mt-1 truncate">{exportResult.path}</p>
        </div>
      )}
    </div>
  );
}
