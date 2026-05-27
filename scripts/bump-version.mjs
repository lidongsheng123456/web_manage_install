#!/usr/bin/env node
/**
 * 一键同步更新三处版本号：
 *   - package.json
 *   - src-tauri/tauri.conf.json
 *   - src-tauri/Cargo.toml
 *
 * 用法：
 *   npm run bump 1.2.3        # 指定版本号
 *   npm run bump -- patch     # 自增 patch (1.0.0 → 1.0.1)
 *   npm run bump -- minor     # 自增 minor (1.0.0 → 1.1.0)
 *   npm run bump -- major     # 自增 major (1.0.0 → 2.0.0)
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const PKG = resolve(root, 'package.json');
const TAURI = resolve(root, 'src-tauri/tauri.conf.json');
const CARGO = resolve(root, 'src-tauri/Cargo.toml');

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf-8'));
}

function writeJson(path, data) {
  writeFileSync(path, JSON.stringify(data, null, 2) + '\n', 'utf-8');
}

function bumpSemver(current, type) {
  const [major, minor, patch] = current.split('.').map(Number);
  switch (type) {
    case 'major': return `${major + 1}.0.0`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    default:      return null;
  }
}

const arg = process.argv[2];
if (!arg) {
  const pkg = readJson(PKG);
  console.log(`当前版本: ${pkg.version}`);
  console.log('用法: npm run bump <version|patch|minor|major>');
  process.exit(0);
}

const pkg = readJson(PKG);
const current = pkg.version;
const semverRe = /^\d+\.\d+\.\d+$/;

let newVer;
if (semverRe.test(arg)) {
  newVer = arg;
} else {
  newVer = bumpSemver(current, arg);
  if (!newVer) {
    console.error(`无效参数: "${arg}"，请使用 <版本号> 或 patch/minor/major`);
    process.exit(1);
  }
}

if (newVer === current) {
  console.log(`版本未变: ${current}`);
  process.exit(0);
}

// 1. package.json
pkg.version = newVer;
writeJson(PKG, pkg);
console.log(`✔ package.json          ${current} → ${newVer}`);

// 2. tauri.conf.json
const tauri = readJson(TAURI);
tauri.version = newVer;
writeJson(TAURI, tauri);
console.log(`✔ tauri.conf.json       ${current} → ${newVer}`);

// 3. Cargo.toml
let cargo = readFileSync(CARGO, 'utf-8');
cargo = cargo.replace(
  /^version\s*=\s*"[^"]*"/m,
  `version = "${newVer}"`
);
writeFileSync(CARGO, cargo, 'utf-8');
console.log(`✔ Cargo.toml            ${current} → ${newVer}`);

console.log(`\n版本已更新为 ${newVer}，打包后 exe 文件属性将显示此版本号`);
