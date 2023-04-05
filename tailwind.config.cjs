const defaultTheme = require('tailwindcss/defaultTheme');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', ...defaultTheme.fontFamily.sans]
      },
      fontSize: {
        xxs: '0.875rem', // 14px
        '3xs': '0.75rem',
        ...defaultTheme.fontSize
      }
    },
  },
  plugins: []
};
