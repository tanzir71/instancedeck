import type { Profile } from './model';
import copy from '../copy/en.json';

export function profileDetail(
  profile: Profile,
  running: boolean,
  startedAt: number | undefined,
  now: number,
): string {
  const state =
    profile.signInState === 'signed-in'
      ? copy.desktop.signedIn
      : profile.signInState === 'needs-sign-in'
        ? copy.desktop.notSignedIn
        : copy.desktop.unknownSignIn;
  if (running && startedAt && startedAt <= now) {
    const minutes = Math.floor((now - startedAt) / 60000);
    const duration =
      minutes < 60
        ? copy.desktop.minutes.replace('{n}', String(minutes))
        : copy.desktop.hours.replace('{n}', String(Math.floor(minutes / 60)));
    return copy.desktop.runningDetail
      .replace('{state}', state)
      .replace('{duration}', duration);
  }
  if (!running && profile.lastLaunchedAt) {
    const days = Math.floor(
      (now - Date.parse(profile.lastLaunchedAt)) / 86400000,
    );
    if (days > 0)
      return `${state} · ${days === 1 ? copy.desktop.usedYesterday : copy.desktop.usedDaysAgo.replace('{n}', String(days))}`;
  }
  return state;
}
