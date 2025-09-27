#!/usr/bin/env node
const { spawnSync } = require('child_process');

const pkgs = [
  'astraweave-render',
  'visual_3d',
  'weaving_playground',
  'physics_demo3d',
  'terrain_demo',
  'cutscene_render_demo',
  'unified_showcase'
];

function run(cmd, args) {
  const p = spawnSync(cmd, args, { stdio: 'inherit', shell: true });
  if (p.status !== 0) process.exit(p.status ?? 1);
}

console.log('== AstraWeave Graphics Check ==');
for (const pkg of pkgs) {
  console.log(`-- cargo check -p ${pkg}`);
  run('cargo', ['check', '-p', pkg]);
}
console.log('All graphics checks completed.');
