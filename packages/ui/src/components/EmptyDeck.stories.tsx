import type { Meta, StoryObj } from '@storybook/react-vite';
import { EmptyDeck } from './EmptyDeck';
const meta = {
  title: 'Screens/Deck',
  component: EmptyDeck,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof EmptyDeck>;
export default meta;
export const Empty: StoryObj<typeof meta> = {};
export const RTL: StoryObj<typeof meta> = {
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
