import type { Meta, StoryObj } from '@storybook/react-vite';
import { ScaffoldPage, SiteFooter } from './ScaffoldPage';
const meta = {
  title: 'Marketing/Scaffold',
  component: ScaffoldPage,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof ScaffoldPage>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Desktop1440: Story = {
  globals: { viewport: { value: 'desktop' } },
};
export const Laptop1024: Story = { globals: { viewport: { value: 'laptop' } } };
export const Tablet768: Story = { globals: { viewport: { value: 'tablet' } } };
export const Mobile390: Story = { globals: { viewport: { value: 'mobile' } } };
export const Footer: Story = { render: () => <SiteFooter /> };
