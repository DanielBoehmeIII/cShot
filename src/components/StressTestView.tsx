import { useState, useCallback } from "react";
import { runStressTest, validateExport } from "../lib/api";

interface StressTestViewProps {
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function StressTestView({ onError, onSuccess }: StressTestViewProps) {
  const [iterations, setIterations] = useState(10);
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<any>(null);
  const [validationId, setValidationId] = useState("");
  const [validationResult, setValidationResult] = useState<any>(null);
  const [validating, setValidating] = useState(false);

  const handleRunStress = useCallback(async () => {
    setRunning(true);
    setResults(null);
    try {
      const result = await runStressTest(iterations);
      setResults(result);
      if (result.failed > 0) {
        onError(`${result.failed} of ${result.total_tests} tests failed`);
      } else {
        onSuccess(`All ${result.total_tests} stress tests passed!`);
      }
    } catch (e) {
      onError(`Stress test failed: ${e}`);
    } finally {
      setRunning(false);
    }
  }, [iterations, onError, onSuccess]);

  const handleValidate = useCallback(async () => {
    if (!validationId.trim()) return;
    setValidating(true);
    setValidationResult(null);
    try {
      const result = await validateExport(validationId.trim());
      setValidationResult(result);
      if (result.is_valid) {
        onSuccess("Export validation passed");
      } else {
        onError("Export validation failed");
      }
    } catch (e) {
      onError(`Validation failed: ${e}`);
    } finally {
      setValidating(false);
    }
  }, [validationId, onError, onSuccess]);

  return (
    <div className="rounded-xl border border-[#2A2A3F] bg-[#14141F] overflow-hidden fade-slide-up">
      <div className="px-4 py-2.5 border-b border-[#2A2A3F]/50">
        <h3 className="text-[10px] font-mono font-medium uppercase tracking-wider text-[#6C5CE7]">
          Stress Test & Validation
        </h3>
        <p className="text-[9px] text-[#636E72] font-mono mt-0.5">Run generation & export quality tests</p>
      </div>

      <div className="p-4 space-y-3">
        <div>
          <label className="text-[9px] font-mono text-[#636E72]">Iterations: {iterations}</label>
          <input type="range" min="5" max="100" step="5" value={iterations}
            onChange={(e) => setIterations(parseInt(e.target.value))}
            className="w-full h-1 appearance-none bg-[#2A2A3F] rounded-full cursor-pointer
              [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2.5 [&::-webkit-slider-thumb]:h-2.5
              [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[#6C5CE7]"
          />
        </div>

        <button onClick={handleRunStress} disabled={running}
          className="w-full rounded-lg bg-[#6C5CE7] px-4 py-2 text-xs font-medium text-white font-mono
            transition-all hover:bg-[#7C6CF7] disabled:opacity-30 disabled:cursor-not-allowed
            shadow-[0_0_12px_rgba(108,92,231,0.15)]">
          {running ? "Running..." : "Run Stress Test"}
        </button>

        {results && (
          <div className="rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-3 space-y-1.5 fade-slide-up">
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Total Tests</span>
              <span className="text-[9px] text-[#DFE6E9] font-mono">{results.total_tests}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Passed</span>
              <span className="text-[9px] text-[#00B894] font-mono">{results.passed}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Failed</span>
              <span className={`text-[9px] font-mono ${results.failed > 0 ? "text-[#D63031]" : "text-[#00B894]"}`}>
                {results.failed}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Avg Gen Time</span>
              <span className="text-[9px] text-[#DFE6E9] font-mono">{results.avg_generation_time_ms.toFixed(1)}ms</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Silent Outputs</span>
              <span className={`text-[9px] font-mono ${results.silent_outputs > 0 ? "text-[#E17055]" : "text-[#00B894]"}`}>
                {results.silent_outputs}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-[9px] text-[#636E72] font-mono">Clipped Outputs</span>
              <span className={`text-[9px] font-mono ${results.clipped_outputs > 0 ? "text-[#E17055]" : "text-[#00B894]"}`}>
                {results.clipped_outputs}
              </span>
            </div>
          </div>
        )}

        <div className="h-px bg-[#2A2A3F]/50" />

        <div>
          <label className="text-[9px] font-mono text-[#636E72]">Export Validation</label>
          <div className="flex gap-2 mt-1">
            <input type="text" value={validationId}
              onChange={(e) => setValidationId(e.target.value)}
              placeholder="Sound ID to validate"
              className="flex-1 rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] px-3 py-1.5 text-[10px] font-mono text-[#DFE6E9] placeholder:text-[#636E72] outline-none focus:border-[#6C5CE7]/50"
            />
            <button onClick={handleValidate} disabled={validating || !validationId.trim()}
              className="rounded-lg bg-[#6C5CE7] px-3 py-1.5 text-[10px] font-mono text-white hover:bg-[#7C6CF7] disabled:opacity-30 transition-colors">
              {validating ? "..." : "Validate"}
            </button>
          </div>
          {validationResult && (
            <div className="mt-2 rounded-lg border border-[#2A2A3F] bg-[#1E1E2E] p-2 text-[9px] font-mono space-y-1">
              <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${validationResult.is_valid ? "bg-[#00B894]" : "bg-[#D63031]"}`} />
                <span className={validationResult.is_valid ? "text-[#00B894]" : "text-[#D63031]"}>
                  {validationResult.is_valid ? "Valid" : "Invalid"}
                </span>
              </div>
              <InfoRow label="Duration" value={`${validationResult.duration_ms.toFixed(0)}ms`} />
              <InfoRow label="Peak" value={validationResult.peak.toFixed(3)} />
              <InfoRow label="RMS" value={validationResult.rms.toFixed(3)} />
              {validationResult.has_clipping && <InfoRow label="Clipping" value="Yes" />}
              {validationResult.is_silent && <InfoRow label="Silent" value="Yes" />}
              {validationResult.warnings?.length > 0 && (
                <div className="text-[8px] text-[#E17055]">
                  {validationResult.warnings.map((w: string, i: number) => (
                    <p key={i}>⚠ {w}</p>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-[#4A4A6F]">{label}</span>
      <span className="text-[#DFE6E9]">{value}</span>
    </div>
  );
}
