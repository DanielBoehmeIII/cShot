const STORAGE_KEY = "cshot_session";

interface SessionData {
  recentPrompts: string[];
  favoriteTags: Record<string, number>;
  favoriteRecipes: string[];
  mostExportedTransforms: Record<string, number>;
  rejectedTransforms: string[];
  lastExportFolder: string;
  lastUsedSettings: Record<string, string>;
}

function load(): SessionData {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { }
  return {
    recentPrompts: [],
    favoriteTags: {},
    favoriteRecipes: [],
    mostExportedTransforms: {},
    rejectedTransforms: [],
    lastExportFolder: "",
    lastUsedSettings: {},
  };
}

function save(data: SessionData): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
}

let session = load();

export function getRecentPrompts(): string[] {
  return session.recentPrompts;
}

export function addRecentPrompt(prompt: string): void {
  session.recentPrompts = [prompt, ...session.recentPrompts.filter(p => p !== prompt)].slice(0, 20);
  save(session);
}

export function getFavoriteTags(): Record<string, number> {
  return session.favoriteTags;
}

export function addFavoriteTags(tags: string[]): void {
  for (const tag of tags) {
    session.favoriteTags[tag] = (session.favoriteTags[tag] || 0) + 1;
  }
  save(session);
}

export function getFavoriteRecipes(): string[] {
  return session.favoriteRecipes;
}

export function getMostExportedTransforms(): Record<string, number> {
  return session.mostExportedTransforms;
}

export function addExportedTransform(name: string): void {
  session.mostExportedTransforms[name] = (session.mostExportedTransforms[name] || 0) + 1;
  save(session);
}

export function addRejectedTransform(name: string): void {
  if (!session.rejectedTransforms.includes(name)) {
    session.rejectedTransforms.push(name);
  }
  save(session);
}

export function getRejectedTransforms(): string[] {
  return session.rejectedTransforms;
}

export function getLastExportFolder(): string {
  return session.lastExportFolder;
}

export function setLastExportFolder(folder: string): void {
  session.lastExportFolder = folder;
  save(session);
}

export function getLastUsedSettings(): Record<string, string> {
  return session.lastUsedSettings;
}

export function setLastUsedSetting(key: string, value: string): void {
  session.lastUsedSettings[key] = value;
  save(session);
}

export function clearAll(): void {
  session = {
    recentPrompts: [],
    favoriteTags: {},
    favoriteRecipes: [],
    mostExportedTransforms: {},
    rejectedTransforms: [],
    lastExportFolder: "",
    lastUsedSettings: {},
  };
  localStorage.removeItem(STORAGE_KEY);
}

export function clearRecentPrompts(): void {
  session.recentPrompts = [];
  save(session);
}
