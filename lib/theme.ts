export interface Theme {
  id: string;
  name: string;
  background: string;
  accent: string;
  light?: boolean;
}

export const THEMES: Theme[] = [
  { id: "ember", name: "Ember", background: "#2e2d2b", accent: "#feb453" },
  { id: "midnight", name: "Midnight", background: "#171c28", accent: "#7aa2f7" },
  { id: "forest", name: "Forest", background: "#212921", accent: "#9bd47c" },
  { id: "crimson", name: "Crimson", background: "#2b2124", accent: "#f2889c" },
  {
    id: "paper",
    name: "Paper",
    background: "#f3efe7",
    accent: "#b97a24",
    light: true,
  },
];

export const DEFAULT_THEME = "ember";
const STORAGE_KEY = "qopy-theme";

export const isTheme = (id: string): boolean =>
  THEMES.some((theme) => theme.id === id);

export const getStoredTheme = (): string => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && isTheme(stored)) return stored;
  } catch {
    // localStorage unavailable (private mode / early init)
  }
  return DEFAULT_THEME;
};

export const applyTheme = (id: string): void => {
  const theme = isTheme(id) ? id : DEFAULT_THEME;
  document.documentElement.setAttribute("data-theme", theme);
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    // ignore
  }
};
