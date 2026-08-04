// `__APP_VERSION__` is baked in by vite.config.ts from package.json, which the
// release workflow stamps with the tag's version before building. A local build
// therefore reports whatever the checked-in manifests say (0.1.0).
const version = __APP_VERSION__;
const versionLabel = `v${version} · local`;

export function useAppVersion() {
  return { version, versionLabel };
}
