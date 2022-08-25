/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["../htom_core/**/*_page.rs"],
  theme: {
    extend: {},
  },
  plugins: [require("@tailwindcss/forms")],
};
