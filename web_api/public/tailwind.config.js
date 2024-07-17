/** @type {import('tailwindcss').Config} */
module.exports = {
  purge: ['./templates/**/*.hbs'],
  content: ["./static/**/*.{js,css}"],
  theme: {
    extend: {

    },
    screens: {
      'sm': {'max': '768px'},
      // => @media (max-width: 1535px) { ... }
      'lg': {'min': '769px'},
      // => @media (min-width: 768px) { ... }
    },
  },
  plugins: [require("@catppuccin/tailwindcss")({
    defaultFlavour: "frappe",
  })],
 
}

