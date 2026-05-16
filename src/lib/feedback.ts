export async function submitFeedback(
  soundId: string,
  thumbsUp: boolean,
  thumbsDown: boolean,
  usable?: boolean | null,
  note?: string | null,
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("submit_feedback", {
    soundId,
    thumbsUp,
    thumbsDown,
    usable: usable ?? null,
    note: note ?? null,
  });
}
