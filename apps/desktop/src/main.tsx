import React from 'react';
import { createRoot } from 'react-dom/client';
import { EmptyDeck } from '@instancedeck/ui';
import '@instancedeck/ui/styles';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <EmptyDeck />
  </React.StrictMode>,
);
