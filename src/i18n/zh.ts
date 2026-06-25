export default {
  nav: {
    accounts: '账号管理',
    metrics: '仪表盘',
    logs: '请求日志',
    quota: '额度查询',
    domains: '域名管理',
    guide: '使用引导',
    settings: '设置'
  },
  status: {
    running: '运行中',
    stopped: '已停止',
    certInstalled: '证书已安装',
    certNotInstalled: '未安装证书',
    proxyOn: '代理开启',
    proxyOff: '代理关闭'
  },
  accounts: {
    title: '账号管理',
    add: '添加账号',
    remove: '删除',
    switch: '接入',
    placeholder: '输入 Token（每行一个）',
    autoSwitch: '自动切换',
    autoSwitchDesc: '余量不足时自动接入下一个账号',
    batchAdd: '批量添加'
  },
  metrics: {
    title: '统计仪表盘',
    totalRequests: '总请求数',
    intercepted: '已拦截',
    authFailures: '认证失败',
    rotations: '自动轮换',
    interceptRate: '拦截率',
    uptime: '运行时间',
    traffic: '代理流量',
    reset: '重置统计'
  },
  logs: {
    title: '请求日志',
    refresh: '刷新',
    time: '时间',
    method: '方法',
    host: '域名',
    path: '路径',
    status: '状态',
    injected: '注入',
    empty: '暂无请求日志'
  },
  quota: {
    title: '额度查询',
    refresh: '刷新额度',
    planName: '套餐',
    daysRemaining: '剩余天数',
    expired: '已过期',
    expiringSoon: '即将过期',
    available: '可用',
    noData: '暂无数据，点击刷新查询'
  },
  domains: {
    title: '域名管理',
    discovered: '已发现域名',
    targets: '拦截目标',
    addToTarget: '添加到目标',
    removeFromTarget: '移除',
    save: '保存配置',
    refresh: '刷新'
  },
  config: {
    title: '设置',
    port: '代理端口',
    portHint: '修改端口需重启生效',
    strategy: '轮换策略',
    strategySequential: '顺序轮换',
    strategyRandom: '随机选择',
    strategyPriority: '按优先级',
    strategyLeastUsed: '最少使用',
    strategyByExpiry: '按到期时间',
    autoRotate: '自动轮换',
    autoStart: '启动时自动开启代理',
    maxRetry: '最大重试次数',
    language: '界面语言',
    save: '保存配置'
  },
  guide: {
    title: '使用引导',
    step1: '启动代理服务',
    step1Desc: '点击"启动代理"按钮开启本地 HTTPS 代理',
    step2: '安装 CA 证书',
    step2Desc: '点击"安装证书"将 CA 证书添加到系统信任库',
    step3: '添加账号',
    step3Desc: '输入 Qoder Token 添加到账号列表',
    step4: '配置 IDE 代理',
    step4Desc: '在 Qoder IDE 中设置 HTTP 代理为 http://127.0.0.1:5888',
    step5: '开始使用',
    step5Desc: '正常使用 Qoder IDE，代理会自动注入认证信息'
  },
  common: {
    confirm: '确定',
    cancel: '取消',
    save: '保存',
    delete: '删除',
    refresh: '刷新',
    loading: '加载中...',
    success: '操作成功',
    error: '操作失败',
    version: '版本',
    checkUpdate: '检查更新'
  }
}
