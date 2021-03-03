import esbuild from 'esbuild';

esbuild.serve({
    servedir: '.',
    onRequest: console.log.bind(console),
}, {
    entryPoints: ['thing.js'],
    bundle: true,
    define: {
        global: '{}',
        msCrypto: '{}',
        process: '{}',
        Buffer: '{}'
    },
    outfile: 'bundled.js',
    external: ['fs', 'path', 'child_process']
}).then(console.log.bind(console));
