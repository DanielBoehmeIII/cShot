import { useState, useCallback } from "react";
import {
  autoTagSound, suggestFilename, suggestPack,
  type AutoTags, type PackSuggestion,
} from "../lib/api";

interface WorkflowPanelProps {
  soundId: string | null;
  allSoundIds: string[];
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function WorkflowPanel({ soundId, allSoundIds, onError, onSuccess }: WorkflowPanelProps) {
  const [autoTags, setAutoTags] = useState<AutoTags | null>(null);
  const [filenameSuggestion, setFilenameSuggestion] = useState<string | null>(null);
  const [packSuggestion, setPackSuggestion] = useState<PackSuggestion | null>(null);
  const [loadingTags, setLoadingTags] = useState(false);
  const [loadingFilename, setLoadingFilename] = useState(false);
  const [loadingPack, setLoadingPack] = useState(false);

  const handleAutoTag = useCallback(async () => {
    if (!soundId) return;
    setLoadingTags(true);
    try {
      const tags = await autoTagSound(soundId);
      setAutoTags(tags);
      onSuccess("Auto-tagged");
    } catch (e) {
      onError(`Auto-tag failed: ${e}`);
    } finally {
      setLoadingTags(false);
    }
  }, [soundId, onError, onSuccess]);

  const handleSuggestFilename = useCallback(async () => {
    if (!soundId) return;
    setLoadingFilename(true);
    try {
      const name = await suggestFilename(soundId);
      setFilenameSuggestion(name);
      onSuccess("Filename suggested");
    } catch (e) {
      onError(`Filename suggestion failed: ${e}`);
    } finally {
      setLoadingFilename(false);
    }
  }, [soundId, onError, onSuccess]);

  const handleSuggestPack = useCallback(async () => {
    if (allSoundIds.length < 2) {
      onError("Need at least 2 sounds for a pack suggestion");
      return;
    }
    setLoadingPack(true);
    try {
      const suggestion = await suggestPack(allSoundIds);
      setPackSuggestion(suggestion);
      onSuccess("Pack suggestion ready");
    } catch (e) {
      onError(`Pack suggestion failed: ${e}`);
    } finally {
      setLoadingPack(false);
    }
  }, [allSoundIds, onError, onSuccess]);

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Workflow Automation
        </h3>
        <p className="text-[9px] text-[#636E72] font-mono mt-0.5">
          Auto-tag, name, and pack suggestions
        </p>
      </div>

      <div className="p-4 space-y-3">
        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-[9px] font-mono text-[#636E72]">Auto-Tag</span>
            <button onClick={handleAutoTag} disabled={loadingTags || !soundId}
              className="rounded border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] disabled:opacity-30 transition-colors">
              {loadingTags ? "..." : "Tag"}
            </button>
          </div>
          {autoTags && (
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 space-y-1">
              <div className="flex items-center gap-1">
                <span className="text-[8px] text-[#4A4A6F] font-mono">Type:</span>
                <span className="text-[9px] text-[#6C5CE7] font-mono">{autoTags.sound_type}</span>
              </div>
              {autoTags.descriptors.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {autoTags.descriptors.map(d => (
                    <span key={d} className="rounded bg-[#00D2D3]/10 px-1.5 py-0.5 text-[8px] text-[#00D2D3] font-mono">{d}</span>
                  ))}
                </div>
              )}
              <div className="flex items-center gap-2 text-[8px] text-[#4A4A6F] font-mono">
                <span>Role: {autoTags.mix_role}</span>
                <span>Energy: {autoTags.energy_level}</span>
                <span>Dur: {autoTags.duration_label}</span>
              </div>
            </div>
          )}
        </div>

        <div className="h-px bg-[#2A2A3F]/50" />

        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-[9px] font-mono text-[#636E72]">Filename Suggestion</span>
            <button onClick={handleSuggestFilename} disabled={loadingFilename || !soundId}
              className="rounded border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] disabled:opacity-30 transition-colors">
              {loadingFilename ? "..." : "Suggest"}
            </button>
          </div>
          {filenameSuggestion && (
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2">
              <code className="text-[9px] text-[#00D2D3] font-mono">{filenameSuggestion}</code>
            </div>
          )}
        </div>

        <div className="h-px bg-[#2A2A3F]/50" />

        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-[9px] font-mono text-[#636E72]">Pack Suggestion</span>
            <button onClick={handleSuggestPack} disabled={loadingPack || allSoundIds.length < 2}
              className="rounded border border-[#2A2A3F] bg-[#1E1E2E] px-2 py-0.5 text-[8px] font-mono text-[#636E72] hover:text-[#DFE6E9] disabled:opacity-30 transition-colors"
              title={allSoundIds.length < 2 ? "Need 2+ sounds" : ""}>
              {loadingPack ? "..." : "Suggest"}
            </button>
          </div>
          {packSuggestion && (
            <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 space-y-1">
              <p className="text-[9px] text-[#DFE6E9] font-mono">{packSuggestion.title}</p>
              <div className="grid grid-cols-4 gap-1 text-[8px] font-mono">
                {(["has_kick", "has_snare", "has_hat", "has_clap", "has_perc", "has_bass", "has_fx"] as const).map((role) => (
                  <div key={role} className={`rounded px-1 py-0.5 text-center ${packSuggestion[role] ? "bg-[#00B894]/10 text-[#00B894]" : "bg-[#2A2A3F]/50 text-[#4A4A6F]"}`}>
                    {role.replace("has_", "")}
                  </div>
                ))}
              </div>
              {packSuggestion.missing_roles.length > 0 && (
                <p className="text-[8px] text-[#E17055] font-mono">
                  Missing: {packSuggestion.missing_roles.join(", ")}
                </p>
              )}
              <div className="flex items-center gap-1">
                <span className="text-[8px] text-[#4A4A6F] font-mono">Cohesion:</span>
                <div className="flex-1 h-1.5 bg-[#2A2A3F] rounded-full overflow-hidden">
                  <div className="h-full bg-[#6C5CE7] rounded-full"
                    style={{ width: `${packSuggestion.cohesion_score * 100}%` }} />
                </div>
                <span className="text-[8px] text-[#6C5CE7] font-mono">
                  {(packSuggestion.cohesion_score * 100).toFixed(0)}%
                </span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
