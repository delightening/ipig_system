/**
 * ESM resolve hook：把 `cloudflare:email` 導到本地 stub。
 *
 * 刻意不引入任何 devDependency（vitest / miniflare 之類）——新增依賴屬必問，
 * 而 Node 內建的 `node:test` + `node:module` register 已經夠用。
 */
const STUB = new URL("./stub-cloudflare-email.mjs", import.meta.url).href;

export async function resolve(specifier, context, next) {
  if (specifier === "cloudflare:email") {
    return { url: STUB, shortCircuit: true };
  }
  return next(specifier, context);
}
