#!/usr/bin/env node

import {
  GATEWAY_PACKAGE_TARGETS,
  listGatewayPackageTargets,
} from './lib/drive-topology.mjs';

function printHelp() {
  console.log(`Usage: node scripts/print-gateway-package-matrix.mjs [--profile standalone|cloud-config|all]

Print Drive gateway packaging targets for CI and release automation.
`);
}

function main() {
  const argv = process.argv.slice(2);
  if (argv.includes('--help') || argv.includes('-h')) {
    printHelp();
    process.exit(0);
  }

  let profile = 'all';
  const profileIndex = argv.indexOf('--profile');
  if (profileIndex >= 0) {
    profile = argv[profileIndex + 1] ?? 'all';
  }

  const targets = listGatewayPackageTargets(profile);
  console.log(JSON.stringify({ profile, targets, all: GATEWAY_PACKAGE_TARGETS }, null, 2));
}

main();
