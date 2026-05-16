import { useState, useCallback, useRef } from "react";

export interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "info";
  duration?: number;
}

const TOAST_DEFAULTS = { success: 3000, error: 6000, info: 3000 };

let toastCounter = 0;

export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const addToast = useCallback(
    (message: string, type: Toast["type"] = "info", duration?: number) => {
      const id = `toast-${++toastCounter}`;
      const ms = duration ?? TOAST_DEFAULTS[type];
      const toast: Toast = { id, message, type, duration: ms };
      setToasts((prev) => [...prev, toast]);
      const timer = setTimeout(() => removeToast(id), ms);
      timersRef.current.set(id, timer);
      return id;
    },
    [removeToast],
  );

  const success = useCallback(
    (message: string) => addToast(message, "success"),
    [addToast],
  );
  const error = useCallback(
    (message: string) => addToast(message, "error"),
    [addToast],
  );
  const info = useCallback(
    (message: string) => addToast(message, "info"),
    [addToast],
  );

  return { toasts, addToast, removeToast, success, error, info };
}
