const esbuild = require('esbuild');

const isWatch = process.argv.includes('--watch');

async function main() {
  const ctx = await esbuild.context({
    entryPoints: ['src/extension.ts'],
    bundle: true,
    outfile: 'out/extension.js',
    external: ['vscode'],
    format: 'cjs',
    platform: 'node',
    target: 'node18',
    sourcemap: true,
    minify: !isWatch,
    logLevel: 'info',
  });

  if (isWatch) {
    console.log('[esbuild] Watching for changes...');
    await ctx.watch();
  } else {
    await ctx.rebuild();
    await ctx.dispose();
    console.log('[esbuild] Build succeeded.');
  }
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
