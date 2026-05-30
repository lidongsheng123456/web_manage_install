/**
 * 应用常量配置
 *
 * 集中管理组件名映射、图标路径、默认配置等，
 * 避免魔法字符串散落在各模块中。
 */

import nodeLogo from '../../assets/logos/nodejs.svg';
import jdkLogo from '../../assets/logos/jdk.png';
import mavenLogo from '../../assets/logos/maven.png';
import mysqlLogo from '../../assets/logos/mysql.svg';
import ideaLogo from '../../assets/logos/idea.svg';
import navicatLogo from '../../assets/logos/navicat.ico';
import redisLogo from '../../assets/logos/redis.svg';

/**
 * 核心环境组件元数据
 */
export const CORE_COMPONENTS = [
  { id: 'nodejs', name: 'Node.js', iconClass: 'node', iconSrc: nodeLogo, verifyCmd: 'node -v' },
  { id: 'jdk', name: 'JDK', iconClass: 'jdk', iconSrc: jdkLogo, verifyCmd: 'java -version' },
  { id: 'maven', name: 'Maven', iconClass: 'maven', iconSrc: mavenLogo, verifyCmd: 'mvn -v' },
  { id: 'mysql', name: 'MySQL', iconClass: 'mysql', iconSrc: mysqlLogo, verifyCmd: 'mysql -V' },
];

/**
 * 附加工具组件元数据
 */
export const BUNDLED_TOOLS = [
  {
    id: 'idea',
    name: 'IntelliJ IDEA 2023.3.8',
    iconClass: 'idea',
    iconSrc: ideaLogo,
    badge: '手动安装',
    activatable: true,
  },
  {
    id: 'navicat',
    name: 'Navicat Premium 17',
    iconClass: 'navicat',
    iconSrc: navicatLogo,
    badge: '手动安装',
    activatable: true,
  },
  {
    id: 'redis',
    name: 'Redis 5.0.14',
    iconClass: 'redis',
    iconSrc: redisLogo,
    badge: '免安装',
    activatable: false,
  },
];

/**
 * 组件 ID → 版本选择器 DOM ID 映射
 */
export const VERSION_SELECT_IDS = {
  nodejs: 'ver-nodejs',
  jdk: 'ver-jdk',
  maven: 'ver-maven',
  mysql: 'ver-mysql',
};

/**
 * 组件 ID → 显示名称生成（带版本号）
 */
export function getComponentDisplayName(id, versions) {
  const prefix = { nodejs: 'Node.js', jdk: 'JDK', maven: 'Maven', mysql: 'MySQL' };
  if (prefix[id] && versions[id]) {
    return `${prefix[id]} ${id === 'nodejs' ? 'v' : ''}${versions[id]}`;
  }
  const tool = BUNDLED_TOOLS.find(t => t.id === id);
  return tool ? tool.name : id;
}

/**
 * 全部组件的快速查找映射
 */
export const ALL_COMPONENTS_MAP = Object.fromEntries(
  [...CORE_COMPONENTS, ...BUNDLED_TOOLS].map(c => [c.id, c])
);
