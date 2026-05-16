import { useRef, useCallback, useState } from "react";

const MAX_CACHED_BUFFERS = 20;

export function useAudioPlayback() {
  const audioContextRef = useRef<AudioContext | null>(null);
  const sourceRef = useRef<AudioBufferSourceNode | null>(null);
  const currentIdRef = useRef<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [activeId, setActiveId] = useState<string | null>(null);
  const bufferCacheRef = useRef<Map<string, AudioBuffer>>(new Map());
  const cacheOrderRef = useRef<string[]>([]);

  const getContext = useCallback(() => {
    if (!audioContextRef.current || audioContextRef.current.state === "closed") {
      audioContextRef.current = new AudioContext();
    }
    return audioContextRef.current;
  }, []);

  const cacheBuffer = useCallback((id: string, buffer: AudioBuffer) => {
    const cache = bufferCacheRef.current;
    const order = cacheOrderRef.current;

    if (cache.has(id)) {
      const idx = order.indexOf(id);
      if (idx >= 0) order.splice(idx, 1);
      order.push(id);
      return;
    }

    if (cache.size >= MAX_CACHED_BUFFERS) {
      const oldest = order.shift();
      if (oldest) cache.delete(oldest);
    }

    cache.set(id, buffer);
    order.push(id);
  }, []);

  const getCachedBuffer = useCallback((id: string): AudioBuffer | undefined => {
    return bufferCacheRef.current.get(id);
  }, []);

  const play = useCallback(
    async (soundId: string, samples: number[], sampleRate: number) => {
      setIsLoading(true);
      try {
        const ctx = getContext();

        if (ctx.state === "suspended") {
          await ctx.resume();
        }

        if (sourceRef.current) {
          try {
            sourceRef.current.stop();
          } catch {
            /* already stopped */
          }
          sourceRef.current.disconnect();
          sourceRef.current = null;
        }

        let buffer = getCachedBuffer(soundId);

        if (!buffer) {
          if (!samples || samples.length === 0) {
            throw new Error("No audio data to play");
          }
          buffer = ctx.createBuffer(1, samples.length, sampleRate);
          const channelData = buffer.getChannelData(0);
          for (let i = 0; i < samples.length; i++) {
            channelData[i] = samples[i];
          }
          cacheBuffer(soundId, buffer);
        }

        const source = ctx.createBufferSource();
        source.buffer = buffer;
        source.connect(ctx.destination);
        source.onended = () => {
          setIsPlaying(false);
          setActiveId(null);
          currentIdRef.current = null;
        };
        source.start();
        sourceRef.current = source;
        currentIdRef.current = soundId;
        setActiveId(soundId);
        setIsPlaying(true);
        setIsLoading(false);
      } catch (e) {
        setIsLoading(false);
        setIsPlaying(false);
        setActiveId(null);
        currentIdRef.current = null;
        throw e;
      }
    },
    [getContext, getCachedBuffer, cacheBuffer],
  );

  const preload = useCallback(
    async (soundId: string, samples: number[], sampleRate: number) => {
      if (getCachedBuffer(soundId)) return;
      const ctx = getContext();
      const buffer = ctx.createBuffer(1, samples.length, sampleRate);
      const channelData = buffer.getChannelData(0);
      for (let i = 0; i < samples.length; i++) {
        channelData[i] = samples[i];
      }
      cacheBuffer(soundId, buffer);
    },
    [getContext, getCachedBuffer, cacheBuffer],
  );

  const stop = useCallback(() => {
    if (sourceRef.current) {
      try {
        sourceRef.current.stop();
      } catch {
        /* already stopped */
      }
      sourceRef.current.disconnect();
      sourceRef.current = null;
    }
    currentIdRef.current = null;
    setActiveId(null);
    setIsPlaying(false);
    setIsLoading(false);
  }, []);

  const stopIfPlaying = useCallback(
    (soundId: string) => {
      if (currentIdRef.current === soundId) {
        stop();
      }
    },
    [stop],
  );

  return { play, stop, stopIfPlaying, preload, isPlaying, isLoading, activeId };
}
