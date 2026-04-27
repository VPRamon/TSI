import { lstat, symlink } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceNodeModules = path.resolve(scriptDir, '../../../node_modules');
const frontendNodeModules = path.resolve(scriptDir, '../node_modules');

try {
  const stat = await lstat(workspaceNodeModules);
  if (stat.isSymbolicLink() || stat.isDirectory()) {
    process.exit(0);
  }
} catch (error) {
  if (error && typeof error === 'object' && 'code' in error && error.code !== 'ENOENT') {
    throw error;
  }
}

await symlink(path.relative(path.dirname(workspaceNodeModules), frontendNodeModules), workspaceNodeModules, 'dir');
