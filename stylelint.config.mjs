export default {
  ignoreFiles: ['**/tokens/generated.css'],
  extends: ['stylelint-config-standard'],
  rules: {
    'property-disallowed-list': ['box-shadow', 'text-shadow'],
    'declaration-property-value-disallowed-list': {
      '/.*/': ['/gradient\\(/'],
      '/^border(?:-(?:top|right|bottom|left|width|style|color))?$/': [
        '/^(?!0$|none$).+/',
      ],
    },
    'selector-class-pattern': null,
    'custom-property-pattern': null,
    'media-feature-range-notation': 'context',
  },
};
