import { createStore } from 'zustand/vanilla';
import type { DesktopSnapshot } from './model';
export type Screen = 'deck' | 'apps' | 'diagnostics' | 'settings';
interface DesktopState {
  snapshot: DesktopSnapshot | undefined;
  screen: Screen;
  error: string;
  setSnapshot: (snapshot: DesktopSnapshot) => void;
  setScreen: (screen: Screen) => void;
  setError: (error: string) => void;
}
// Each manager/switcher view owns its subscription. No cross-story singleton or
// browser persistence; Rust remains the authority for profile data and settings.
export function createDesktopStore(initialScreen: Screen = 'deck') {
  return createStore<DesktopState>()((set) => ({
    snapshot: undefined,
    screen: initialScreen,
    error: '',
    setSnapshot: (snapshot) => set({ snapshot }),
    setScreen: (screen) => set({ screen }),
    setError: (error) => set({ error }),
  }));
}
