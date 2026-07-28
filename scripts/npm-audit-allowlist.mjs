#!/usr/bin/env node
import { spawnSync } from 'node:child_process';

const allowedAdvisories = new Map([
  [
    'GHSA-qwww-vcr4-c8h2',
    'React Router RSC Mode advisory. This frontend is a Vite SPA and does not use React Router RSC/server action features.',
  ],
]);

const result = spawnSync('npm', ['audit', '--json', '--audit-level=high'], {
  cwd: process.cwd(),
  encoding: 'utf8',
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

let report;
try {
  report = JSON.parse(result.stdout || '{}');
} catch {
  console.error(result.stdout);
  console.error(result.stderr);
  process.exit(1);
}

const findings = [];
for (const vulnerability of Object.values(report.vulnerabilities ?? {})) {
  for (const item of vulnerability.via ?? []) {
    if (typeof item === 'string') {
      continue;
    }

    const advisoryId = item.url?.split('/').pop();
    if (advisoryId && allowedAdvisories.has(advisoryId)) {
      console.warn(`Allowed npm audit advisory: ${advisoryId} - ${allowedAdvisories.get(advisoryId)}`);
      continue;
    }

    findings.push({
      package: vulnerability.name,
      advisory: advisoryId ?? item.source ?? 'unknown',
      severity: item.severity ?? vulnerability.severity,
      title: item.title ?? 'unknown advisory',
      url: item.url,
    });
  }
}

if (findings.length > 0) {
  console.error('Disallowed npm audit findings:');
  for (const finding of findings) {
    console.error(`- [${finding.severity}] ${finding.package}: ${finding.title} (${finding.advisory})`);
    if (finding.url) {
      console.error(`  ${finding.url}`);
    }
  }
  process.exit(1);
}

console.log('npm audit passed with configured allowlist.');
