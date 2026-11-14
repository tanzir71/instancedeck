import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { DesktopApp, GlobalSwitcher } from '@instancedeck/ui';
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { desktopClient } from './client';
import '@instancedeck/ui/styles';

function SwitcherRoot() {
  const [openCount, setOpenCount] = useState(0);
  useEffect(() => {
    if (!isTauri()) return;
    let active = true;
    let stop: (() => void) | undefined;
    void listen('switcher-open', () => {
      if (active) setOpenCount((count) => count + 1);
    })
      .then((value) => {
        if (active) stop = value;
        else value();
      })
      .catch((error) =>
        console.error('Switcher event subscription failed', error),
      );
    return () => {
      active = false;
      stop?.();
    };
  }, []);
  return <GlobalSwitcher key={openCount} client={desktopClient} />;
}

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {(
      isTauri()
        ? getCurrentWebviewWindow().label === 'switcher'
        : new URLSearchParams(window.location.search).get('view') === 'switcher'
    ) ? (
      <SwitcherRoot />
    ) : (
      <DesktopApp client={desktopClient} />
    )}
  </React.StrictMode>,
);
