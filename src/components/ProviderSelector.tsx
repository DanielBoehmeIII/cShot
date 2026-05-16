import { useState, useEffect, useCallback } from "react";

interface ProviderInfo {
  name: string;
  display_name: string;
  is_available: boolean;
  reason_unavailable: string | null;
  supports_reference_audio: boolean;
  max_duration_seconds: number;
  estimated_latency_ms: number;
  estimated_cost_cents: number;
  requires_api_key: boolean;
  requires_network: boolean;
}

export function ProviderSelector() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [activeProvider, setActiveProvider] = useState("mock-dsp");
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const [list, active] = await Promise.all([
          invoke<ProviderInfo[]>("get_generation_providers"),
          invoke<string>("get_active_provider"),
        ]);
        setProviders(list);
        setActiveProvider(active);
      } catch {
        setProviders([]);
      } finally {
        setIsLoading(false);
      }
    }
    load();
  }, []);

  const handleSelect = useCallback(async (name: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_active_provider", { providerName: name });
      setActiveProvider(name);
      setIsOpen(false);
    } catch {}
  }, []);

  const active = providers.find((p) => p.name === activeProvider);

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`flex items-center gap-2 rounded-lg border px-2.5 py-1.5 text-[10px] font-mono transition-all ${
          isOpen
            ? "border-[#6C5CE7]/50 bg-[#6C5CE7]/10"
            : "border-[#2A2A3F] bg-[#1E1E2E] hover:border-[#3A3A5F]"
        }`}
        disabled={isLoading}
      >
        {isLoading ? (
          <div className="spinner w-3 h-3 rounded-full border-2 border-[#636E72] border-t-[#6C5CE7]" />
        ) : (
          <>
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                active?.is_available ? "bg-[#00B894]" : "bg-[#D63031]"
              }`}
            />
            <span className="text-[#636E72]">
              {active?.display_name || activeProvider}
            </span>
            <svg
              className={`w-3 h-3 text-[#636E72] transition-transform ${isOpen ? "rotate-180" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M6 9l6 6 6-6" />
            </svg>
          </>
        )}
      </button>

      {isOpen && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />
          <div className="absolute right-0 top-full mt-1 z-50 w-72 rounded-xl border border-[#2A2A3F] bg-[#1A1A2E] shadow-2xl backdrop-blur-xl overflow-hidden">
            <div className="p-2 border-b border-[#2A2A3F]/50">
              <p className="text-[10px] text-[#636E72] font-mono uppercase tracking-wider px-2 py-1">
                Generation Provider
              </p>
            </div>
            <div className="p-1">
              {providers.map((p) => (
                <button
                  key={p.name}
                  onClick={() => p.is_available && handleSelect(p.name)}
                  disabled={!p.is_available}
                  className={`w-full rounded-lg px-3 py-2.5 text-left transition-all ${
                    activeProvider === p.name
                      ? "bg-[#6C5CE7]/15 border border-[#6C5CE7]/30"
                      : "border border-transparent hover:bg-[#14141F]"
                  } ${!p.is_available ? "opacity-40 cursor-not-allowed" : ""}`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span
                        className={`w-2 h-2 rounded-full ${
                          p.is_available ? "bg-[#00B894]" : "bg-[#D63031]"
                        }`}
                      />
                      <span className="text-xs font-mono text-[#DFE6E9]">
                        {p.display_name}
                      </span>
                    </div>
                    {activeProvider === p.name && (
                      <span className="text-[#6C5CE7] text-xs">✓</span>
                    )}
                  </div>
                  <div className="mt-1.5 flex items-center gap-3 text-[10px] text-[#636E72] font-mono">
                    <span>{p.estimated_latency_ms}ms</span>
                    {p.estimated_cost_cents > 0 && (
                      <span>~${(p.estimated_cost_cents / 100).toFixed(2)}/gen</span>
                    )}
                    {p.requires_network && <span>🌐</span>}
                    {p.supports_reference_audio && <span>ref</span>}
                  </div>
                  {!p.is_available && p.reason_unavailable && (
                    <p className="mt-1 text-[9px] text-[#D63031] font-mono leading-tight">
                      {p.reason_unavailable}
                    </p>
                  )}
                </button>
              ))}
            </div>
            <div className="p-2 border-t border-[#2A2A3F]/50">
              <p className="text-[9px] text-[#2A2A3F] font-mono text-center">
                {active?.requires_api_key
                  ? "Set API keys in .env file"
                  : activeProvider === "mock-dsp"
                    ? "Local DSP · No API key needed"
                    : ""}
              </p>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
