import esbuild from 'esbuild';
import chalk from 'chalk';
const { blueBright } = chalk;

esbuild.serve({
    servedir: '.',
    onRequest({ method, path, status }) {
        console.log((`${blueBright(method)} ${status} ${path}`));
    }
}, {
    entryPoints: ['thing.js'],
    bundle: true,
    color: true,
    define: {
        global: '{}',
        msCrypto: '{}',
        process: '{}',
        Buffer: '{}'
    },
    logLevel: 'info',
    outfile: 'bundled.js',
    external: ['fs', 'path', 'child_process']
}).then(console.log.bind(console));
