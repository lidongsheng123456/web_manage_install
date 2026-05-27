/**
 * 应用入口模块
 *
 * 组装各功能模块，绑定页面事件，初始化版本选择器。
 * 模块依赖关系：
 *   main.js → navigation.js   (步骤跳转)
 *           → detect.js       (环境检测)
 *           → installer.js    (下载安装)
 *           → results.js      (结果展示)
 *           → versions.js     (版本配置)
 */

import { goToStep } from './js/navigation.js';
import { runDetection, installFlags, resetFlags } from './js/detect.js';
import { runInstall } from './js/installer.js';
import { renderResults, initResultEvents } from './js/results.js';
import { NODE_VERSIONS, JDK_VERSIONS, MAVEN_VERSIONS, MYSQL_VERSIONS } from './js/versions.js';

const { open } = window.__TAURI__.dialog;

// ─── 初始化版本选择器 ──────────────────────────────────────────

/**
 * 用版本数据填充 <select> 下拉框
 * @param {string} selectId - select 元素的 id
 * @param {Array}  versions - 版本配置数组
 */
function populateVersionSelect(selectId, versions) {
  const select = document.getElementById(selectId);
  if (!select) return;
  select.innerHTML = versions.map(v =>
    `<option value="${v.value}" ${v.default ? 'selected' : ''}>${v.label}</option>`
  ).join('');
}

populateVersionSelect('ver-nodejs', NODE_VERSIONS);
populateVersionSelect('ver-jdk', JDK_VERSIONS);
populateVersionSelect('ver-maven', MAVEN_VERSIONS);
populateVersionSelect('ver-mysql', MYSQL_VERSIONS);

// ─── 检查本地附加资源可用性 ─────────────────────────────────
const { invoke: _invoke } = window.__TAURI__.core;
(async function checkBundledResources() {
  try {
    const resources = await _invoke('check_bundled_resources');
    let anyAvailable = false;
    for (const [name, available] of resources) {
      const chk = document.getElementById(`chk-${name}`);
      const hint = document.getElementById(`hint-${name}`);
      if (chk) {
        chk.disabled = !available;
        if (!available) {
          chk.checked = false;
          if (hint) hint.textContent = '（未找到安装包）';
        } else {
          anyAvailable = true;
          if (hint) hint.textContent = '✓ 已就绪';
        }
      }
    }
    if (anyAvailable) {
      document.getElementById('bundled-tools').style.display = '';
    }
  } catch (e) {
    console.warn('检查附加资源失败', e);
  }
})();

// ─── Step 1: 配置页事件 ────────────────────────────────────────

/** 浏览按钮 → 选择安装目录 */
document.getElementById('btn-browse').addEventListener('click', async () => {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      document.getElementById('install-path').value = selected;
    }
  } catch (e) {
    console.error('浏览目录失败', e);
  }
});

/** 下一步 → 进入环境检测 */
document.getElementById('btn-next-1').addEventListener('click', () => {
  goToStep(2);
  runDetection();
});

// ─── Step 2: 检测页事件 ────────────────────────────────────────

/** 上一步 → 返回配置 */
document.getElementById('btn-prev-2').addEventListener('click', () => goToStep(1));

/** 开始安装 → 进入安装页 */
document.getElementById('btn-next-2').addEventListener('click', () => {
  goToStep(3);
  runInstall();
});

// ─── Step 3: 安装页事件 ────────────────────────────────────────

/** 查看结果 → 进入结果页 */
document.getElementById('btn-next-3').addEventListener('click', () => {
  goToStep(4);
  renderResults();
});

// ─── Step 4: 结果页事件 ────────────────────────────────────────

/**
 * 重置所有 UI 状态并回到 Step 1
 */
function resetAndGoHome() {
  resetFlags();
  window._installResults = null;

  /* 重置按钮状态 */
  document.getElementById('btn-next-2').disabled = true;
  document.getElementById('btn-next-3').disabled = true;
  document.getElementById('btn-cancel-install').style.display = 'none';
  document.getElementById('btn-cancel-install').disabled = false;
  document.getElementById('btn-cancel-install').textContent = '取消安装';
  document.getElementById('btn-rollback').style.display = 'none';
  document.getElementById('btn-back-to-config').style.display = 'none';

  /* 清空安装日志 */
  document.getElementById('log-content').innerHTML = '';

  /* 重置进度卡片（含附加工具） */
  document.querySelectorAll('.progress-card').forEach(c => c.classList.remove('done', 'error'));
  document.querySelectorAll('.prog-bar').forEach(b => { b.style.width = '0%'; b.className = 'prog-bar'; });
  document.querySelectorAll('.prog-status').forEach(s => { s.textContent = '等待中'; s.className = 'prog-status'; });
  document.querySelectorAll('.prog-detail').forEach(d => { d.textContent = ''; });
  ['prog-idea', 'prog-navicat', 'prog-redis'].forEach(id => {
    const el = document.getElementById(id);
    if (el) el.style.display = 'none';
  });

  /* 重置验证结果 */
  document.querySelectorAll('.verify-cmd-result').forEach(r => { r.textContent = ''; r.className = 'verify-cmd-result'; });

  goToStep(1);
}

initResultEvents(resetAndGoHome);
