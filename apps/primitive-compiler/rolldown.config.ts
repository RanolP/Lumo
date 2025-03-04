import path from 'node:path';
import { defineConfig } from 'rolldown';
import { aliasPlugin } from 'rolldown/experimental';

export default defineConfig({
  plugins: [
    aliasPlugin({
      entries: [
        { find: '@', replacement: path.resolve(import.meta.dirname, 'src') },
      ],
    }),
  ],
  external: [/node:.*/],
  input: 'src/index.ts',
  output: {
    file: 'dist/index.js',
  },
});
