import { getJestConfig } from '@storybook/test-runner';
import { fileURLToPath } from 'node:url';

export default {
  ...getJestConfig(),
  roots: [fileURLToPath(new URL('../src', import.meta.url))],
  modulePathIgnorePatterns: ['<rootDir>/artifacts/', '<rootDir>/target/'],
};
