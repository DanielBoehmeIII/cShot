import { useState, useCallback } from "react";
import {
  type EvolutionConfig, type EvolutionState, type EvolutionMember,
  type RegionLock,
  evolveSoundCommand, saveEvolutionMember,
} from "../lib/api";

interface EvolutionViewProps {
  soundId: string;
  soundPrompt: string;
  soundType: string;
  onSoundCreated: () => void;
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

const DIRECTIONS = [
  { value: "", label: "None (random)" },
  { value: "harder", label: "Harder" },
  { value: "cleaner", label: "Cleaner" },
  { value: "warmer", label: "Warmer" },
  { value: "brighter", label: "Brighter" },
  { value: "heavier", label: "Heavier" },
  { value: "lighter", label: "Lighter" },
  { value: "longer", label: "Longer" },
  { value: "shorter", label: "Shorter" },
];

export function EvolutionView({
  soundId, soundPrompt, soundType, onSoundCreated, onError, onSuccess,
}: EvolutionViewProps) {
  const [config, setConfig] = useState<EvolutionConfig>({
    generations: 5, population_size: 8, mutation_rate: 0.3, crossover_rate: 0.2,
    quality_bias: 0.6, novelty_bias: 0.3, preserve_best: true, elite_count: 2,
    region_lock: { lock_transient: false, lock_body: false, lock_tail: false, lock_sub: false, lock_noise: false },
    trait_preference: { preferred: [], disliked: [] },
  });
  const [direction, setDirection] = useState("");
  const [directionIntensity, setDirectionIntensity] = useState(0.5);
  const [isEvolving, setIsEvolving] = useState(false);
  const [state, setState] = useState<EvolutionState | null>(null);
  const [savingId, setSavingId] = useState<string | null>(null);

  const handleEvolve = useCallback(async () => {
    if (!soundId) return;
    setIsEvolving(true);
    try {
      const result = await evolveSoundCommand(
        soundId, config,
        direction || undefined,
        directionIntensity,
      );
      setState(result);
      onSuccess(`Evolution complete: ${result.generation} generations, ${result.population.length} members`);
    } catch (e) {
      onError(`Evolution failed: ${e}`);
    } finally {
      setIsEvolving(false);
    }
  }, [soundId, config, direction, directionIntensity, onError, onSuccess]);

  const handleSaveMember = useCallback(async (member: EvolutionMember) => {
    setSavingId(member.id);
    try {
      await saveEvolutionMember(member, soundType, soundPrompt);
      onSuccess("Member saved to library");
      onSoundCreated();
    } catch (e) {
      onError(`Failed to save: ${e}`);
    } finally {
      setSavingId(null);
    }
  }, [soundType, soundPrompt, onSoundCreated, onError, onSuccess]);

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Procedural Evolution
        </h3>
        <p className="text-[9px] text-[#636E72] font-mono mt-0.5">
          Evolve sounds across generations with selective inheritance
        </p>
      </div>

      <div className="p-4 space-y-3">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Generations</label>
            <input type="range" min="1" max="20" value={config.generations}
              onChange={(e) => setConfig({ ...config, generations: parseInt(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
            <span className="text-[9px] text-[#4A4A6F] font-mono">{config.generations}</span>
          </div>
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Population</label>
            <input type="range" min="4" max="24" value={config.population_size}
              onChange={(e) => setConfig({ ...config, population_size: parseInt(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
            <span className="text-[9px] text-[#4A4A6F] font-mono">{config.population_size}</span>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Mutation Rate</label>
            <input type="range" min="0" max="1" step="0.05" value={config.mutation_rate}
              onChange={(e) => setConfig({ ...config, mutation_rate: parseFloat(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Crossover Rate</label>
            <input type="range" min="0" max="1" step="0.05" value={config.crossover_rate}
              onChange={(e) => setConfig({ ...config, crossover_rate: parseFloat(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Quality Bias</label>
            <input type="range" min="0" max="1" step="0.05" value={config.quality_bias}
              onChange={(e) => setConfig({ ...config, quality_bias: parseFloat(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Novelty Bias</label>
            <input type="range" min="0" max="1" step="0.05" value={config.novelty_bias}
              onChange={(e) => setConfig({ ...config, novelty_bias: parseFloat(e.target.value) })}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
        </div>

        <div>
          <label className="text-[9px] font-mono text-[#636E72]">Direction</label>
          <select value={direction} onChange={(e) => setDirection(e.target.value)}
            className="w-full rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-1.5 text-[10px] font-mono text-[#DFE6E9] outline-none focus:border-[#6C5CE7]/50 mt-1">
            {DIRECTIONS.map((d) => (
              <option key={d.value} value={d.value}>{d.label}</option>
            ))}
          </select>
        </div>
        {direction && (
          <div>
            <label className="text-[9px] font-mono text-[#636E72]">Intensity</label>
            <input type="range" min="0.1" max="1" step="0.05" value={directionIntensity}
              onChange={(e) => setDirectionIntensity(parseFloat(e.target.value))}
              className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
                [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
            />
          </div>
        )}

        <details className="text-[9px] font-mono text-[#636E72]">
          <summary className="cursor-pointer hover:text-[#DFE6E9]">Region Locks</summary>
          <div className="mt-2 space-y-1.5 pl-2">
            {(["transient", "body", "tail", "sub", "noise"] as const).map((region) => (
              <label key={region} className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox"
                  checked={config.region_lock[`lock_${region}` as keyof RegionLock]}
                  onChange={(e) => setConfig({
                    ...config,
                    region_lock: { ...config.region_lock, [`lock_${region}`]: e.target.checked },
                  })}
                  className="w-3 h-3 accent-[#6C5CE7]"
                />
                <span className="capitalize">{region}</span>
              </label>
            ))}
          </div>
        </details>

        <button
          onClick={handleEvolve}
          disabled={isEvolving || !soundId}
          className="w-full rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
            transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
            shadow-[0_0_12px_rgba(108,92,231,0.15)]"
        >
          {isEvolving ? `Evolving...` : "Start Evolution →"}
        </button>
      </div>

      {state && (
        <div className="border-t border-[#2A2A3F]/50 p-4 space-y-3 fade-slide-up">
          <div className="flex items-center justify-between">
            <h4 className="text-[10px] font-mono text-[#6C5CE7] uppercase tracking-wider">
              Evolution Results
            </h4>
            <span className="text-[9px] text-[#4A4A6F] font-mono">
              Gen {state.generation}
            </span>
          </div>

          <div className="grid grid-cols-4 gap-2">
            {state.history.map((h, i) => (
              <div key={i} className={`rounded-lg border p-2 ${
                i === state.history.length - 1
                  ? "border-[#6C5CE7]/30 bg-[#6C5CE7]/5"
                  : "border-[#2A2A3F] bg-[#1E1E2E]"
              }`}>
                <p className="text-[8px] text-[#4A4A6F] font-mono">Gen {h.generation}</p>
                <p className="text-[9px] text-[#DFE6E9] font-mono tabular-nums">
                  {h.member_count} pop
                </p>
                <p className="text-[8px] text-[#6C5CE7] font-mono tabular-nums">
                  best: {(h.best_score * 100).toFixed(0)}%
                </p>
              </div>
            ))}
          </div>

          <div className="h-px bg-[#2A2A3F]/50" />

          <p className="text-[9px] text-[#636E72] font-mono">Population ranked by score</p>
          <div className="space-y-1 max-h-48 overflow-y-auto">
            {state.population.map((member) => (
              <div key={member.id}
                className={`flex items-center justify-between rounded-lg border p-2 ${
                  member.is_elite
                    ? "border-[#FDCB6E]/30 bg-[#FDCB6E]/5"
                    : "border-[#2A2A3F] bg-[#1E1E2E]"
                }`}>
                <div className="flex items-center gap-2">
                  <span className="text-[9px] text-[#4A4A6F] font-mono">G{member.generation}</span>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <div className="h-1.5 w-16 bg-[#2A2A3F] rounded-full overflow-hidden">
                        <div className="h-full bg-[#6C5CE7] rounded-full"
                          style={{ width: `${member.score * 100}%` }} />
                      </div>
                      {member.is_elite && <span className="text-[8px] text-[#FDCB6E]">★</span>}
                    </div>
                    <p className="text-[8px] text-[#4A4A6F] font-mono">
                      Q:{(member.quality_score * 100).toFixed(0)}% N:{(member.novelty_score * 100).toFixed(0)}%
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => handleSaveMember(member)}
                  disabled={savingId === member.id}
                  className="rounded border border-[#2A2A3F] bg-[#14141F] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] transition-colors disabled:opacity-30"
                >
                  {savingId === member.id ? "..." : "Save"}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
