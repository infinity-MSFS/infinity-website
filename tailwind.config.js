/** @type {import('tailwindcss').Config} */
module.exports = {
  presets: [require("../infinity-ui-wasm/style/tailwind-preset.js")],
  content: [
    "./index.html",
    "./src/**/*.rs",
    "../infinity-ui-wasm/crates/infinity-ui/src/**/*.rs",
  ],
};
