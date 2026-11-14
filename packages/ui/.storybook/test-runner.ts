import type { TestRunnerConfig } from '@storybook/test-runner';
import AxeBuilder from '@axe-core/playwright';

const config: TestRunnerConfig = {
  async postVisit(page) {
    const { violations } = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze();
    if (violations.length)
      throw new Error(
        JSON.stringify(
          violations.map(({ id, nodes }) => ({
            id,
            targets: nodes.map((n) => n.target),
          })),
          null,
          2,
        ),
      );
  },
};
export default config;
