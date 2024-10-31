import typography from "@tailwindcss/typography";
import flowbite from "flowbite/plugin";
import type { Config } from "tailwindcss";
import defaultTheme from "tailwindcss/defaultTheme";

export default {
    content: ["./src/**/*.{html,js,svelte,ts}", "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}"],

    theme: {
        extend: {
            fontFamily: {
                sans: ["Inter Variable", ...defaultTheme.fontFamily.sans],
                mono: ["JetBrains Mono Variable", ...defaultTheme.fontFamily.mono]
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

    plugins: [flowbite, typography]
} satisfies Config;
