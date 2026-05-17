import { useState, useEffect } from "react";

interface ReleaseInfoProps {
  onClose: () => void;
}

export function ReleaseInfo({ onClose }: ReleaseInfoProps) {
  const [capabilities, setCapabilities] = useState<string[][]>([]);

  useEffect(() => {
    async function load() {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const caps: string[][] = await invoke("get_capability_summary");
        setCapabilities(caps);
      } catch {}
    }
    load();
  }, []);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="rounded-2xl border border-[#2A2A3F] bg-[#1A1A2E] p-8 max-w-lg w-full mx-4 shadow-2xl max-h-[80vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-sm font-semibold text-[#DFE6E9] font-mono">cShot</h2>
            <p className="text-[10px] text-[#636E72] font-mono mt-0.5">AI Promptable Sound Design · v0.1.0-beta</p>
          </div>
          <button onClick={onClose} className="text-[#636E72] hover:text-[#DFE6E9] p-1">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <p className="text-[10px] text-[#4A4A6F] font-mono leading-relaxed mb-6">
          cShot is a promptable sound design instrument — describe sounds in natural language,
          generate them instantly, recreate references, evolve variations, and organize your
          library. All processing is local, private, and fast.
        </p>

        <div className="space-y-3">
          <h3 className="text-[9px] font-mono text-[#6C5CE7] uppercase tracking-wider">Capabilities</h3>
          <div className="grid grid-cols-1 gap-1.5">
            {capabilities.map(([key, value]) => (
              <div key={key} className="flex items-start gap-2">
                <span className="w-1.5 h-1.5 rounded-full bg-[#6C5CE7] mt-1 shrink-0" />
                <div>
                  <span className="text-[9px] font-mono text-[#DFE6E9]">{key}</span>
                  <p className="text-[8px] text-[#4A4A6F] font-mono">{value}</p>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="mt-6 pt-4 border-t border-[#2A2A3F]/50">
          <p className="text-[8px] text-[#2A2A3F] font-mono text-center">
            Local-first · No cloud required · All processing local
          </p>
        </div>
      </div>
    </div>
  );
}
