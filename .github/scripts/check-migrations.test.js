'use strict';

const { test } = require('node:test');
const assert = require('node:assert/strict');

const mod = require('./check-migrations.js');

test('detects DROP COLUMN', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t DROP COLUMN c;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('destructive DROP'));
});

test('detects DROP without COLUMN keyword', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t DROP name;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('destructive DROP'));
});

test('detects RENAME COLUMN', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t RENAME COLUMN a TO b;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('RENAME'));
});

test('detects RENAME without COLUMN keyword', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t RENAME name TO display;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('RENAME'));
});

test('detects ALTER COLUMN ... RENAME TO', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t ALTER COLUMN a RENAME TO b;');
  assert.ok(v.length > 0);
});

test('detects RENAME TO (table rename)', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t RENAME TO new_name;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('RENAME'));
});

test('detects ALTER COLUMN ... TYPE', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t ALTER COLUMN a TYPE text;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('ALTER COLUMN ... TYPE'));
});

test('detects DROP TABLE', () => {
  const v = mod.findViolations('m.sql', 'DROP TABLE t;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('destructive DROP'));
});

test('detects DROP INDEX', () => {
  const v = mod.findViolations('m.sql', 'DROP INDEX idx;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('destructive DROP'));
});

test('detects DROP CONSTRAINT', () => {
  const v = mod.findViolations('m.sql', 'ALTER TABLE t DROP CONSTRAINT c;');
  assert.ok(v.length > 0);
  assert.ok(v.includes('destructive DROP'));
});

test('DROP NOT NULL is safe', () => {
  assert.equal(
    mod.findViolations('m.sql', 'ALTER TABLE t ALTER COLUMN c DROP NOT NULL;').length,
    0
  );
});

test('DROP DEFAULT is safe', () => {
  assert.equal(
    mod.findViolations('m.sql', 'ALTER TABLE t ALTER COLUMN c DROP DEFAULT;').length,
    0
  );
});

test('contract marker suppresses violations', () => {
  const sql = '-- contract: intentional\nALTER TABLE t DROP COLUMN c;';
  assert.deepEqual(mod.findViolations('m.sql', sql), []);
  assert.equal(mod.isDestructive('m.sql', sql), false);
});

test('contract marker after violations suppresses', () => {
  const sql = 'ALTER TABLE t DROP COLUMN c;\n-- contract: ok';
  assert.equal(mod.findViolations('m.sql', sql).length, 0);
});

test('contract marker middle suppresses', () => {
  const sql = 'ALTER TABLE t DROP COLUMN c;\n-- contract: ok\nALTER TABLE t DROP INDEX i;';
  assert.equal(mod.findViolations('m.sql', sql).length, 0);
});

test('CREATE TABLE is safe', () => {
  assert.equal(mod.findViolations('m.sql', 'CREATE TABLE t (id int);').length, 0);
});

test('ADD COLUMN is safe', () => {
  assert.equal(
    mod.findViolations('m.sql', 'ALTER TABLE t ADD COLUMN c int;').length,
    0
  );
});

test('CREATE INDEX is safe', () => {
  assert.equal(mod.findViolations('m.sql', 'CREATE INDEX i ON t(c);').length, 0);
});

test('marker must be contract comment', () => {
  const sql = '-- note: not contract\nALTER TABLE t DROP COLUMN c;';
  assert.ok(mod.findViolations('m.sql', sql).length > 0);
});
