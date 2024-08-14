import {nextui} from "@nextui-org/react";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@nextui-org/theme/dist/**/*.{js,ts,jsx,tsx}"
    ],
    theme: {
        extend: {},
    },
    darkMode: "class",
    plugins: [nextui({
        themes: {
            light: {
                colors: {
                    primary: {
                        DEFAULT: "#f13848",
                        foreground: "#fff",
                    },
                    secondary: "#2b2b2b",
                    background: "#e3e3ea",

                }
            },
            dark: {
                colors: {
                    primary: "#ff3247",
                    secondary: "#eaeaea",
                    background: "#18181b",
                }
            },
        }
    })]
}