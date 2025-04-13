// web/vite.config.ts
import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react-swc'
import wasm from 'vite-plugin-wasm'
import tsconfigPaths from 'vite-tsconfig-paths'
import checker from 'vite-plugin-checker'

const fullReload = () => ({
    name: 'full-reload',
    handleHotUpdate({server}) {
        server.ws.send({
            type: 'full-reload',
            path: '*',
        })
        // Return an empty array to indicate that no modules are being updated,
        // which effectively skips the default HMR process.
        return []
    },
})

const checkTs = () => checker({typescript: true})

const checkEslint = () =>
    checker({
        eslint: {
            lintCommand: 'eslint --quiet "./{src,bindgen}/**/*.{ts,tsx}"',
            dev: {
                logLevel: ['error'],
            },
        },
    })

export default defineConfig({
    plugins: [react(), fullReload(), wasm(), tsconfigPaths()],
})
