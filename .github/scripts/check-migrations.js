'use strict';

const fs = require('node:fs');

const PATTERNS = [
  { id: 'destructive DROP', re: /\bDROP\s+(?!NOT\s+NULL\b|DEFAULT\b|USING\b)/i },
  { id: 'RENAME', re: /\bRENAME\b/i },
  { id: 'ALTER COLUMN ... TYPE', re: /\bALTER\s+COLUMN\b[\s\S]*?\bTYPE\b/is },
];

function hasContractMarker(content) {
  return String(content ?? '')
    .split(/\r?\n/)
    .some((line) => line.includes('-- contract:'));
}

function findViolations(filePath, content) {
  if (hasContractMarker(content)) return [];
  const text = String(content ?? '');
  const found = [];
  for (const pattern of PATTERNS) {
    if (pattern.re.test(text)) found.push(pattern.id);
  }
  return found;
}

function isDestructive(filePath, content) {
  return findViolations(filePath, content).length > 0;
}

function resolveInputFiles(argv) {
  if (argv.length >= 2) return argv;
  if (argv.length === 1) {
    const arg = argv[0];
    if (!fs.existsSync(arg)) return [arg];
    const content = fs.readFileSync(arg, 'utf8');
    const lines = content
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);
    if (lines.length > 0 && lines.every((line) => fs.existsSync(line))) {
      return lines;
    }
    return [arg];
  }
  try {
    const input = fs.readFileSync(0, 'utf8');
    return input
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

function main() {
  const files = resolveInputFiles(process.argv.slice(2));
  let exitCode = 0;
  for (const file of files) {
    let content;
    try {
      content = fs.readFileSync(file, 'utf8');
    } catch (error) {
      process.stderr.write(`${file}: cannot read file: ${error.message}\n`);
      exitCode = 1;
      continue;
    }
    const violations = findViolations(file, content);
    if (violations.length > 0) {
      process.stdout.write(
        `${file}: destructive migration detected (${violations.join(', ')})\n`
      );
      exitCode = 1;
    }
  }
  process.exitCode = exitCode;
}

if (require.main === module) main();

module.exports = {
  PATTERNS,
  hasContractMarker,
  findViolations,
  isDestructive,
  resolveInputFiles,
  main,
};
