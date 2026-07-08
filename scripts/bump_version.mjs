import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');

const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Usage: node bump_version.mjs <new_version>');
  console.error('Example: node bump_version.mjs 1.2.0');
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error('Error: Version must be in x.y.z format (e.g., 1.2.0)');
  process.exit(1);
}

console.log(`Bumping version to ${newVersion}...\n`);

const filesToUpdate = [
  {
    path: 'package.json',
    type: 'json',
    update: (content) => {
      const json = JSON.parse(content);
      const oldVersion = json.version;
      json.version = newVersion;
      return { newContent: JSON.stringify(json, null, 2) + '\n', oldVersion };
    }
  },
  {
    path: 'src-tauri/tauri.conf.json',
    type: 'json',
    update: (content) => {
      const json = JSON.parse(content);
      const oldVersion = json.version;
      json.version = newVersion;
      return { newContent: JSON.stringify(json, null, 2) + '\n', oldVersion };
    }
  },
  {
    path: 'src-tauri/Cargo.toml',
    type: 'regex',
    update: (content) => {
      let oldVersion = 'unknown';
      const newContent = content.replace(/^(version\s*=\s*")([^"]+)(")/m, (match, p1, p2, p3) => {
        oldVersion = p2;
        return `${p1}${newVersion}${p3}`;
      });
      return { newContent, oldVersion };
    }
  },
  {
    path: 'src/components/SettingsView.vue',
    type: 'regex',
    update: (content) => {
      let oldVersion = 'unknown';
      const newContent = content.replace(/(v\d+\.\d+\.\d+)/g, (match) => {
        oldVersion = match.substring(1);
        return `v${newVersion}`;
      });
      return { newContent, oldVersion };
    }
  }
];

let success = true;

for (const fileDef of filesToUpdate) {
  const filePath = path.join(ROOT_DIR, fileDef.path);
  try {
    if (!fs.existsSync(filePath)) {
      console.warn(`[SKIP] File not found: ${fileDef.path}`);
      continue;
    }
    
    const content = fs.readFileSync(filePath, 'utf-8');
    const { newContent, oldVersion } = fileDef.update(content);
    
    if (content !== newContent) {
      fs.writeFileSync(filePath, newContent, 'utf-8');
      console.log(`[SUCCESS] ${fileDef.path} : ${oldVersion} -> ${newVersion}`);
    } else {
      console.log(`[UNCHANGED] ${fileDef.path} is already at or doesn't contain version format`);
    }
  } catch (e) {
    console.error(`[ERROR] Failed to update ${fileDef.path}:`, e.message);
    success = false;
  }
}

if (success) {
  console.log('\n✅ All versions successfully bumped!');
} else {
  console.error('\n❌ Some files failed to update.');
  process.exit(1);
}
