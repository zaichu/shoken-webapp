import { chromium } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const AUTH_DIR = path.join(__dirname, '../.auth');
const AUTH_FILE = path.join(AUTH_DIR, 'storage-state.json');
const USER_DATA_DIR = path.join(AUTH_DIR, 'user-data');

/**
 * ログイン状態を保存するためのスクリプト
 *
 * 永続的なユーザーデータディレクトリを使用してGoogleログイン制限を回避
 */
async function main() {
  // ディレクトリ作成
  if (!fs.existsSync(AUTH_DIR)) {
    fs.mkdirSync(AUTH_DIR, { recursive: true });
  }

  console.log('\n====================================');
  console.log('永続ブラウザを起動します...');
  console.log('====================================\n');

  // 永続的なコンテキストでブラウザを起動
  const context = await chromium.launchPersistentContext(USER_DATA_DIR, {
    headless: false,
    channel: 'chrome',
    args: ['--disable-blink-features=AutomationControlled'],
  });

  const page = context.pages()[0] || await context.newPage();
  await page.goto('http://localhost:8081/');

  console.log('ブラウザでログインしてください。');
  console.log('ログイン完了後、このターミナルでEnterを押してください。');
  console.log('====================================\n');

  // stdinからの入力を待機
  await new Promise<void>((resolve) => {
    process.stdin.once('data', () => resolve());
  });

  // ログイン状態を保存
  await context.storageState({ path: AUTH_FILE });
  console.log(`\nログイン状態を保存しました: ${AUTH_FILE}`);

  await context.close();
}

main().catch(console.error);
