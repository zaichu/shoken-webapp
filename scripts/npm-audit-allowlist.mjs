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
  if (!result.stdout) {
    console.error(result.stderr || 'npm audit produced no JSON output.');
    process.exit(1);
  }
  report = JSON.parse(result.stdout);
} catch {
  console.error(result.stdout);
  console.error(result.stderr);
  process.exit(1);
}

if (result.signal) {
  console.error(`npm audit terminated by signal: ${result.signal}`);
  process.exit(1);
}

if (report.error) {
  console.error(`npm audit failed: ${report.error.summary ?? 'unknown error'}`);
  if (report.error.detail) {
    console.error(report.error.detail);
  }
  process.exit(1);
}

if (!report.vulnerabilities || typeof report.vulnerabilities !== 'object') {
  console.error('npm audit JSON did not include a vulnerabilities object.');
  process.exit(1);
}

const failingSeverities = new Set(['high', 'critical']);
const findings = [];
for (const vulnerability of Object.values(report.vulnerabilities)) {
  if (!failingSeverities.has(vulnerability.severity)) {
    continue;
  }
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
