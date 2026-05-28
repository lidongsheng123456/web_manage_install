/**
 * 环境检测模块
 *
 * 调用后端 detect_environment 命令扫描系统中已安装的开发工具，
 * 渲染检测结果卡片（绿=已安装且版本匹配 / 黄=已安装但版本不同 / 红=未安装），
 * 并维护 installFlags 控制后续安装哪些组件。
 *
 * 关键改进：检测时将用户在 Step 1 选择的版本号传给后端，
 * 后端用实时注册表 PATH 执行命令，确保刚安装的组件也能检测到。
 */

const { invoke } = window.__TAURI__.core;

/** 各组件是否需要安装的开关（附加工具由 Step1 勾选控制，不参与检测） */
export let installFlags = { nodejs: true, jdk: true, maven: true, mysql: true };

/** 最近一次的检测结果 */
let detectResults = [];

/**
 * 执行环境检测
 *
 * 读取 Step 1 中用户选择的版本号，传给后端作为期望版本，
 * 后端使用注册表实时 PATH 检测，解决安装后检测不到的问题。
 */
export async function runDetection() {
  const container = document.getElementById('detect-results');
  container.innerHTML = `<div class="detect-loading"><div class="spinner"></div><span>正在扫描环境...</span></div>`;

  /* 读取用户选择的期望版本 */
  const nodeVersion = document.getElementById('ver-nodejs')?.value || '20.19.0';
  const jdkVersion = document.getElementById('ver-jdk')?.value || '17';
  const mavenVersion = document.getElementById('ver-maven')?.value || '3.9.6';
  const mysqlVersion = document.getElementById('ver-mysql')?.value || '8.0.36';

  /* MySQL 版本只取前缀 "8.0" 用于匹配 */
  const mysqlPrefix = mysqlVersion.split('.').slice(0, 2).join('.');

  try {
    detectResults = await invoke('detect_environment', {
      nodeVersion,
      jdkVersion,
      mavenVersion,
      mysqlVersion: mysqlPrefix,
    });
    renderDetectResults(detectResults);
    document.getElementById('btn-next-2').disabled = false;
  } catch (e) {
    container.innerHTML = `<div class="detect-loading" style="color:var(--error)">检测失败: ${e}</div>`;
  }
}

/**
 * 渲染检测结果列表
 * @param {Array} results - 后端返回的 ComponentStatus 数组
 */
function renderDetectResults(results) {
  const container = document.getElementById('detect-results');

  /** 组件名 → 内部标识 */
  const nameMap = { 'Node.js': 'nodejs', 'JDK': 'jdk', 'Maven': 'maven', 'MySQL': 'mysql' };
  /** 组件名 → [CSS 类, 官方图标路径] */
  const iconMap = {
    'Node.js': ['node', '/assets/logos/nodejs.svg'],
    'JDK': ['jdk', '/assets/logos/jdk.png'],
    'Maven': ['maven', '/assets/logos/maven.png'],
    'MySQL': ['mysql', '/assets/logos/mysql.svg'],
  };

  container.innerHTML = results.map(r => {
    const key = nameMap[r.name] || r.name.toLowerCase();
    const [iconClass, iconSrc] = iconMap[r.name] || ['node', '/assets/logos/nodejs.svg'];
    let dotClass, actionClass, actionText, statusText;

    if (r.installed && r.versionMatch) {
      /* 版本匹配 → 绿灯，默认跳过 */
      dotClass = 'green';
      actionClass = 'skip';
      actionText = '已安装';
      statusText = `v${r.version} - 版本匹配`;
      installFlags[key] = false;
    } else if (r.installed && !r.versionMatch) {
      /* 已安装但版本不同 → 黄灯 */
      dotClass = 'yellow';
      actionClass = 'upgrade';
      actionText = '将重新安装';
      statusText = `当前 v${r.version}，需要 ${r.expectedVersion}`;
      installFlags[key] = true;
    } else {
      /* 未安装 → 红灯 */
      dotClass = 'red';
      actionClass = 'will-install';
      actionText = '将安装';
      statusText = `未检测到，需要 ${r.expectedVersion}`;
      installFlags[key] = true;
    }

    return `
      <div class="detect-item">
        <div class="comp-icon ${iconClass} small"><img src="${iconSrc}" alt="${r.name}" /></div>
        <div class="status-dot ${dotClass}"></div>
        <div class="detect-info">
          <strong>${r.name}</strong>
          <span>${statusText}</span>
        </div>
        <label class="skip-checkbox">
          <input type="checkbox" ${installFlags[key] ? 'checked' : ''} data-comp="${key}" onchange="toggleInstall(this)"/>
          安装
        </label>
        <span class="detect-action ${actionClass}">${actionText}</span>
      </div>
    `;
  }).join('');
}

/**
 * 切换某组件的安装开关（由 checkbox 触发）
 * @param {HTMLInputElement} el - 复选框元素
 */
window.toggleInstall = function(el) {
  const comp = el.dataset.comp;
  installFlags[comp] = el.checked;
};

/**
 * 重置安装开关为全选
 */
export function resetFlags() {
  installFlags = { nodejs: true, jdk: true, maven: true, mysql: true };
}
