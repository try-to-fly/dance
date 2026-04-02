#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const packageJsonPath = path.join(repoRoot, 'package.json');
const cargoTomlPath = path.join(repoRoot, 'src-tauri', 'Cargo.toml');
const tauriConfigPath = path.join(repoRoot, 'src-tauri', 'tauri.conf.json');
const cargoLockPath = path.join(repoRoot, 'src-tauri', 'Cargo.lock');

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function normalizeVersion(value) {
  if (!value) {
    throw new Error('Missing version. Pass --version <semver> or use --check without a version.');
  }

  const normalized = value.startsWith('v') ? value.slice(1) : value;
  const semverPattern = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

  if (!semverPattern.test(normalized)) {
    throw new Error(`Invalid semver version: ${value}`);
  }

  return normalized;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, JSON.stringify(value, null, 2) + '\n');
}

function readCargoToml() {
  return fs.readFileSync(cargoTomlPath, 'utf8');
}

function writeCargoToml(content) {
  fs.writeFileSync(cargoTomlPath, content);
}

function readCargoPackageName() {
  const cargoToml = readCargoToml();
  const match = cargoToml.match(/^name = "(.+)"$/m);

  if (!match) {
    throw new Error('Unable to read package name from src-tauri/Cargo.toml');
  }

  return match[1];
}

function readCurrentVersions() {
  const packageJson = readJson(packageJsonPath);
  const tauriConfig = readJson(tauriConfigPath);
  const cargoToml = readCargoToml();
  const cargoVersionMatch = cargoToml.match(/^version = "(.+)"$/m);
  const cargoPackageName = readCargoPackageName();
  const cargoLock = fs.readFileSync(cargoLockPath, 'utf8');
  const cargoLockPattern = new RegExp(
    `\\[\\[package\\]\\]\\nname = "${escapeRegExp(cargoPackageName)}"\\nversion = "(.+)"`,
    'm'
  );
  const cargoLockVersionMatch = cargoLock.match(cargoLockPattern);

  if (!cargoVersionMatch) {
    throw new Error('Unable to read version from src-tauri/Cargo.toml');
  }

  if (!cargoLockVersionMatch) {
    throw new Error(`Unable to read ${cargoPackageName} version from src-tauri/Cargo.lock`);
  }

  return {
    packageJson: packageJson.version,
    cargoToml: cargoVersionMatch[1],
    tauriConfig: tauriConfig.version,
    cargoLock: cargoLockVersionMatch[1],
  };
}

function syncVersionFiles(version) {
  const normalizedVersion = normalizeVersion(version);
  const packageJson = readJson(packageJsonPath);
  const tauriConfig = readJson(tauriConfigPath);
  const cargoPackageName = readCargoPackageName();

  packageJson.version = normalizedVersion;
  writeJson(packageJsonPath, packageJson);

  const cargoToml = readCargoToml().replace(
    /^version = ".*"$/m,
    `version = "${normalizedVersion}"`
  );
  writeCargoToml(cargoToml);

  tauriConfig.version = normalizedVersion;
  writeJson(tauriConfigPath, tauriConfig);

  const cargoLockPattern = new RegExp(
    `(\\[\\[package\\]\\]\\nname = "${escapeRegExp(cargoPackageName)}"\\nversion = ").*(")`,
    'm'
  );
  const cargoLock = fs.readFileSync(cargoLockPath, 'utf8');

  if (!cargoLockPattern.test(cargoLock)) {
    throw new Error(`Unable to update ${cargoPackageName} version in src-tauri/Cargo.lock`);
  }

  fs.writeFileSync(cargoLockPath, cargoLock.replace(cargoLockPattern, `$1${normalizedVersion}$2`));

  return normalizedVersion;
}

function checkVersionConsistency(expectedVersion) {
  const versions = readCurrentVersions();
  const expected = expectedVersion ? normalizeVersion(expectedVersion) : versions.packageJson;
  const mismatches = Object.entries(versions).filter(([, value]) => value !== expected);

  if (mismatches.length > 0) {
    console.error(`Version mismatch detected. Expected ${expected}.`);
    for (const [name, value] of Object.entries(versions)) {
      console.error(`- ${name}: ${value}`);
    }
    process.exit(1);
  }

  console.log(`All version files are aligned at ${expected}.`);
}

function parseArgs(argv) {
  const args = { check: false, version: undefined };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === '--check') {
      args.check = true;
      continue;
    }

    if (arg === '--version') {
      args.version = argv[index + 1];
      index += 1;
      continue;
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return args;
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  if (args.check) {
    checkVersionConsistency(args.version);
    return;
  }

  const version = syncVersionFiles(args.version);
  console.log(`Synced package.json, Cargo.toml, tauri.conf.json, and Cargo.lock to ${version}.`);
}

main();
