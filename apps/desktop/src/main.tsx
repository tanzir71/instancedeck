import React from 'react';
import { createRoot } from 'react-dom/client';
import { DesktopApp } from '@instancedeck/ui';
import { desktopClient } from './client';
import '@instancedeck/ui/styles';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <DesktopApp client={desktopClient} />
  </React.StrictMode>,
);
