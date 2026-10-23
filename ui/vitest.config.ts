import { defineConfig } from "vitest/config";
import { fileURLToPath } from "node:url";

export default defineConfig({
  root: fileURLToPath(new URL("..", import.meta.url)),
  test: {
    environment: "node",
    include: ["ui/tests/**/*.test.ts", "scripts/**/*.test.mjs"],
  },
});
