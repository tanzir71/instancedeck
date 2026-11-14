import type { Preview } from '@storybook/react-vite';
import { withThemeByDataAttribute } from '@storybook/addon-themes';
import '../src/styles.css';

const preview: Preview = {
  parameters: {
    layout: 'padded',
    a11y: { test: 'error' },
    viewport: {
      options: {
        desktop: {
          name: 'Desktop 1440',
          styles: { width: '1440px', height: '900px' },
        },
        laptop: {
          name: 'Laptop 1024',
          styles: { width: '1024px', height: '768px' },
        },
        tablet: {
          name: 'Tablet 768',
          styles: { width: '768px', height: '1024px' },
        },
        mobile: {
          name: 'Mobile 390',
          styles: { width: '390px', height: '844px' },
        },
      },
    },
  },
  decorators: [
    withThemeByDataAttribute({
      themes: { light: 'light' },
      defaultTheme: 'light',
      attributeName: 'data-theme',
    }),
  ],
};
export default preview;
