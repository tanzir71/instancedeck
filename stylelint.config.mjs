export default {
  ignoreFiles: ['**/tokens/generated.css'],
  extends: ['stylelint-config-standard'],
  rules: {
    'property-disallowed-list': ['box-shadow', 'text-shadow'],
    'declaration-property-value-disallowed-list': {
      '/.*/': ['/gradient\\(/'],
      '/^border(?!.*(?:radius|collapse|spacing))/': ['/^(?!0$|none$).+/'],
    },
    'selector-class-pattern': null,
    'custom-property-pattern': null,
    'color-no-hex': true,
    'media-feature-range-notation': 'context',
  },
};
