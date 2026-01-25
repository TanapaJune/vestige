import { defineConfig } from '@rstest/core';

export default defineConfig({
  testMatch: ['**/*.test.ts'],
  setupFiles: ['./src/__tests__/setup.ts'],
  coverage: {
    include: ['src/**/*.ts'],
    exclude: ['src/__tests__/**', 'src/**/*.d.ts'],
  },
});
