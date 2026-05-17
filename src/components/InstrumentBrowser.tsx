import { useState, useEffect, useCallback } from "react";
import { getBuiltinPresets, applyPresetMacro } from "../lib/api";

interface InstrumentBrowserProps {
  onSelectPreset: (preset: any) => void;
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function InstrumentBrowser({ onSelectPreset, onError, onSuccess }: InstrumentBrowserProps) {
  const [presets, setPresets] = useState<any[]>([]);
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);
  const [macroValues, setMacroValues] = useState<number[]>([0.5, 0.5, 0.5, 0.5]);
  const [morphPosition] = useState(0.5);
  const [category, setCategory] = useState<string>("all");
  const [view, setView] = useState<"list" | "detail">("list");

  useEffect(() => {
    getBuiltinPresets().catch(() => []).then((builtinPresets) => {
      setPresets(builtinPresets);
    }).catch(() => {});
  }, []);

  const handleSelectPreset = useCallback((name: string) => {
    setSelectedPreset(name);
    setView("detail");
    setMacroValues([0.5, 0.5, 0.5, 0.5]);
  }, []);

  const handleApplyPreset = useCallback(async () => {
    if (!selectedPreset) return;
    const preset = presets.find((p: any) => p.name === selectedPreset);
    if (!preset) return;
    try {
      const applied = await applyPresetMacro(preset, {
        morph_position: morphPosition,
        macro_values: macroValues as [number, number, number, number],
        random_lock: [],
      });
      onSelectPreset(applied);
      onSuccess(`Applied: ${selectedPreset}`);
    } catch (e) {
      onError(`Failed to apply preset: ${e}`);
    }
  }, [selectedPreset, presets, morphPosition, macroValues, onSelectPreset, onError, onSuccess]);

  const categories = ["all", ...new Set(presets.map((p: any) => p.category))];
  const filteredPresets = category === "all"
    ? presets
    : presets.filter((p: any) => p.category === category);

  const selected = presets.find((p: any) => p.name === selectedPreset);

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50 flex items-center justify-between">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Instrument Presets
        </h3>
        <button onClick={() => setView(view === "list" ? "detail" : "list")}
          className="rounded border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] transition-colors">
          {view === "list" ? "Detail" : "List"}
        </button>
      </div>

      <div className="p-4 space-y-3">
        {view === "list" && (
          <>
            <div className="flex gap-1.5 flex-wrap">
              {categories.map(c => (
                <button key={c}
                  onClick={() => setCategory(c)}
                  className={`rounded px-2 py-0.5 text-[9px] font-mono transition-all ${
                    category === c
                      ? "bg-[#6C5CE7]/20 text-[#6C5CE7] border border-[#6C5CE7]/30"
                      : "bg-[#1E1E2E] text-[#636E72] border border-[#2A2A3F]"
                  }`}>
                  {c === "all" ? "All" : c}
                </button>
              ))}
            </div>

            <div className="grid grid-cols-2 gap-2">
              {filteredPresets.map((p: any, i: number) => (
                <button key={p.name || i}
                  onClick={() => handleSelectPreset(p.name)}
                  className={`rounded-lg border p-3 text-left transition-all ${
                    selectedPreset === p.name
                      ? "bg-[#6C5CE7]/10 border-[#6C5CE7]/40"
                      : "bg-[#1E1E2E] border-[#2A2A3F] hover:border-[#3A3A5F]"
                  }`}>
                  <p className="text-[10px] font-mono font-medium text-[#DFE6E9]">{p.name}</p>
                  <p className="text-[8px] text-[#636E72] font-mono mt-0.5">{p.category}</p>
                  <p className="text-[8px] text-[#4A4A6F] font-mono mt-0.5 line-clamp-2">{p.description}</p>
                  {p.macro_mappings && p.macro_mappings.length > 0 && (
                    <p className="text-[8px] text-[#6C5CE7] font-mono mt-1">
                      {p.macro_mappings.length} macros
                    </p>
                  )}
                </button>
              ))}
            </div>
          </>
        )}

        {view === "detail" && selected && (
          <div className="space-y-3 fade-slide-up">
            <button onClick={() => setView("list")}
              className="text-[9px] text-[#636E72] font-mono hover:text-[#DFE6E9] transition-colors">
              ← Back to presets
            </button>

            <div>
              <p className="text-xs font-mono font-medium text-[#DFE6E9]">{selected.name}</p>
              <p className="text-[9px] text-[#636E72] font-mono">{selected.category} · {selected.sound_type}</p>
            </div>

            <p className="text-[9px] text-[#4A4A6F] font-mono leading-relaxed">{selected.description}</p>

            {selected.macro_mappings && selected.macro_mappings.length > 0 && (
              <>
                <div className="h-px bg-[#2A2A3F]/50" />
                <p className="text-[9px] text-[#636E72] font-mono uppercase tracking-wider">Macro Controls</p>
                {selected.macro_mappings.map((mapping: any) => (
                  <div key={mapping.macro_index}>
                    <div className="flex justify-between">
                      <label className="text-[9px] font-mono text-[#636E72]">
                        {mapping.name} (M{mapping.macro_index + 1})
                      </label>
                      <span className="text-[9px] text-[#4A4A6F] font-mono tabular-nums">
                        {(macroValues[mapping.macro_index] * 100).toFixed(0)}%
                      </span>
                    </div>
                    <input type="range" min="0" max="1" step="0.05"
                      value={macroValues[mapping.macro_index]}
                      onChange={(e) => {
                        const vals = [...macroValues];
                        vals[mapping.macro_index] = parseFloat(e.target.value);
                        setMacroValues(vals);
                      }}
                      className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                        [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                        [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
                    />
                  </div>
                ))}
              </>
            )}

            <button
              onClick={handleApplyPreset}
              className="w-full rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
                transition-all hover:bg-[#7C6CF7] shadow-[0_0_12px_rgba(108,92,231,0.15)]"
            >
              Apply Preset → Generator
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
