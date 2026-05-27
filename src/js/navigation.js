/**
 * 导航模块
 *
 * 管理安装向导的步骤切换（Step 1~4）。
 * 控制步骤指示器的 active / done 状态和页面可见性。
 */

/** 当前步骤编号（1~4） */
let currentStep = 1;

/**
 * 跳转到指定步骤
 * @param {number} step - 目标步骤编号
 */
export function goToStep(step) {
  // 隐藏所有页面
  document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));

  // 更新步骤指示器状态
  document.querySelectorAll('.step').forEach(s => {
    const sn = parseInt(s.dataset.step);
    s.classList.remove('active', 'done');
    if (sn < step) s.classList.add('done');
    if (sn === step) s.classList.add('active');
  });

  // 显示目标页面
  document.getElementById(`page-${step}`).classList.add('active');
  currentStep = step;
}

/**
 * 获取当前步骤编号
 * @returns {number}
 */
export function getCurrentStep() {
  return currentStep;
}
