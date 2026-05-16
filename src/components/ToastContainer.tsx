import { type Toast } from "../hooks/useToast";

interface ToastContainerProps {
  toasts: Toast[];
  onRemove: (id: string) => void;
}

const TYPE_STYLES = {
  success: "border-[#00B894]/40 bg-[#00B894]/10 text-[#00B894]",
  error: "border-[#D63031]/40 bg-[#D63031]/10 text-[#D63031]",
  info: "border-[#6C5CE7]/40 bg-[#6C7CE7]/10 text-[#DFE6E9]",
};

export function ToastContainer({ toasts, onRemove }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2 max-w-sm">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`fade-slide-up rounded-xl border px-4 py-3 backdrop-blur-md shadow-lg ${TYPE_STYLES[toast.type]}`}
        >
          <div className="flex items-center justify-between gap-3">
            <p className="text-xs font-mono leading-relaxed">{toast.message}</p>
            <button
              onClick={() => onRemove(toast.id)}
              className="shrink-0 text-current opacity-60 hover:opacity-100 transition-opacity"
            >
              <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
