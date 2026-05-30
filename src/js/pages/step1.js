/**
 * Step1 页面控制器
 *
 * 负责基础配置页面的逻辑：
 * - 安装路径选择
 * - MySQL 密码设置
 * - 版本目录加载与选择
 * - 模拟部署开关
 * - IDEA/Navicat 激活快捷入口
 */

import { getState, setInstallPath, setMysqlPassword, setDryRun, setVersion } from '../state/appState.js';
import { loadVersionCatalog } from '../services/versionService.js';
import { open } from '../services/tauriApi.js';
import { handleActivation } from '../components/ActivationCard.js';
import { VERSION_SELECT_IDS } from '../config/constants.js';
import { $ } from '../utils/dom.js';

/**
 * 初始化 Step1 页面
 * @param {function} onNext - 点击下一步的回调
 */
export function initStep1(onNext) {
  bindBrowseButton();
  bindActivationShortcuts();
  bindFormSync();
  loadVersions();

  $('btn-next-1').addEventListener('click', () => {
    syncStateFromForm();
    onNext();
  });
}

function bindBrowseButton() {
  $('btn-browse').addEventListener('click', async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        $('install-path').value = selected;
        setInstallPath(selected);
      }
    } catch (e) {
      console.error('浏览目录失败', e);
    }
  });
}

function bindActivationShortcuts() {
  const btnIdea = $('btn-activate-idea-home');
  const btnNavicat = $('btn-activate-navicat-home');
  const resultEl = $('activation-result-home');

  if (btnIdea) {
    btnIdea.addEventListener('click', () => handleActivation('idea', btnIdea, resultEl));
  }
  if (btnNavicat) {
    btnNavicat.addEventListener('click', () => handleActivation('navicat', btnNavicat, resultEl));
  }
}

function bindFormSync() {
  const pathInput = $('install-path');
  const pwdInput = $('mysql-password');
  const dryRunInput = $('dry-run');

  if (pathInput) pathInput.addEventListener('change', () => setInstallPath(pathInput.value));
  if (pwdInput) pwdInput.addEventListener('change', () => setMysqlPassword(pwdInput.value));
  if (dryRunInput) dryRunInput.addEventListener('change', () => setDryRun(dryRunInput.checked));
}

function syncStateFromForm() {
  setInstallPath($('install-path').value);
  setMysqlPassword($('mysql-password').value);
  setDryRun($('dry-run').checked);

  Object.entries(VERSION_SELECT_IDS).forEach(([comp, selectId]) => {
    const select = $(selectId);
    if (select) setVersion(comp, select.value);
  });
}

function loadVersions() {
  const selectIds = Object.values(VERSION_SELECT_IDS);

  selectIds.forEach(id => {
    const select = $(id);
    if (!select) return;
    select.innerHTML = '<option value="">正在加载版本...</option>';
    select.disabled = true;
  });
  $('btn-next-1').disabled = true;

  loadVersionCatalog()
    .then(catalog => {
      populateSelect('ver-nodejs', catalog.nodejs);
      populateSelect('ver-jdk', catalog.jdk);
      populateSelect('ver-maven', catalog.maven);
      populateSelect('ver-mysql', catalog.mysql);

      selectIds.forEach(id => {
        const select = $(id);
        if (select) select.disabled = false;
      });
      $('btn-next-1').disabled = false;
    })
    .catch(err => {
      selectIds.forEach(id => {
        const select = $(id);
        if (!select) return;
        select.innerHTML = '<option value="">版本加载失败</option>';
        select.disabled = true;
      });
      $('btn-next-1').disabled = true;
      console.error('加载版本目录失败', err);
    });
}

function populateSelect(selectId, versions) {
  const select = $(selectId);
  if (!select || !versions) return;
  select.innerHTML = versions.map(v =>
    `<option value="${v.value}" ${v.default ? 'selected' : ''} title="${v.source || ''}">${v.label}</option>`
  ).join('');
}
