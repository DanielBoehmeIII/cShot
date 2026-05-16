interface LoadingSpinnerProps {
  message?: string;
}

export function LoadingSpinner({ message }: LoadingSpinnerProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-16">
      <div
        className="spinner w-12 h-12 rounded-full border-4 border-[#2A2A3F] border-t-[#6C5CE7]"
      />
      <p className="text-sm text-[#636E72] font-mono">
        {message || "generating..."}
      </p>
    </div>
  );
}
