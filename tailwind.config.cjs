const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.{html,js,svelte,ts}", "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}"],
    theme: {
        extend: {
            fontFamily: {
                sans: ["Inter Variable", ...defaultTheme.fontFamily.sans]
            },
            fontSize: {
                xxs: "0.875rem", // 14px
                "3xs": "0.75rem",
                ...defaultTheme.fontSize
            },
            animation: {
                "spin-once": "spin 1s linear"
            },
            keyframes: {
                spin: {
                    from: { transform: "rotate(0deg)" },
                    to: { transform: "rotate(-180deg)" }
                }
            }
        }
    },
    plugins: [require("flowbite/plugin"), require("@tailwindcss/typography")]
};
