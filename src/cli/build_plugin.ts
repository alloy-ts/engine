/**
 * BuildPlugin interface for AlloyScript build-time transforms.
 * Entirely separate from RuntimePlugin.
 */

export interface BuildContext {
  filename: string;
  debug: boolean;
  target: "desktop" | "server" | "mobile";
}

export interface BuildPlugin {
  name: string;
  dependsOn?: string[];

  /**
   * Called before esbuild bundling or CPS transform.
   * Useful for shim injection or JSX transformation.
   */
  before_bundle?: (source: string, ctx: BuildContext) => string | Promise<string>;

  /**
   * Called after all transformations are complete.
   */
  after_bundle?: (source: string, ctx: BuildContext) => string | Promise<string>;
}

/**
 * Sorts plugins based on their dependencies using Kahn's algorithm (topological sort).
 */
export function sortPlugins(plugins: BuildPlugin[]): BuildPlugin[] {
  const result: BuildPlugin[] = [];
  const inDegree = new Map<string, number>();
  const adj = new Map<string, string[]>();
  const nameToPlugin = new Map<string, BuildPlugin>();

  for (const p of plugins) {
    nameToPlugin.set(p.name, p);
    if (!inDegree.has(p.name)) inDegree.set(p.name, 0);
    if (!adj.has(p.name)) adj.set(p.name, []);
  }

  for (const p of plugins) {
    if (p.dependsOn) {
      for (const dep of p.dependsOn) {
        if (!nameToPlugin.has(dep)) continue; // Ignore unknown dependencies for now or throw?
        inDegree.set(p.name, (inDegree.get(p.name) || 0) + 1);
        adj.get(dep)!.push(p.name);
      }
    }
  }

  const queue: string[] = [];
  for (const [name, degree] of inDegree.entries()) {
    if (degree === 0) queue.push(name);
  }

  while (queue.length > 0) {
    const u = queue.shift()!;
    result.push(nameToPlugin.get(u)!);
    for (const v of adj.get(u)!) {
      inDegree.set(v, inDegree.get(v)! - 1);
      if (inDegree.get(v) === 0) queue.push(v);
    }
  }

  if (result.length !== plugins.length) {
    throw new Error("Circular dependency detected in BuildPlugins");
  }

  return result;
}

export async function runBeforeBundle(plugins: BuildPlugin[], source: string, ctx: BuildContext): Promise<string> {
  const sorted = sortPlugins(plugins);
  for (const p of sorted) {
    if (p.before_bundle) {
      source = await p.before_bundle(source, ctx);
    }
  }
  return source;
}

export async function runAfterBundle(plugins: BuildPlugin[], source: string, ctx: BuildContext): Promise<string> {
  const sorted = sortPlugins(plugins);
  for (const p of sorted) {
    if (p.after_bundle) {
      source = await p.after_bundle(source, ctx);
    }
  }
  return source;
}
