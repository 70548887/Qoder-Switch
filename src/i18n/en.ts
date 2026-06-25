export default {
  nav: {
    accounts: 'Accounts',
    metrics: 'Dashboard',
    logs: 'Logs',
    quota: 'Quota',
    domains: 'Domains',
    guide: 'Guide',
    settings: 'Settings'
  },
  status: {
    running: 'Running',
    stopped: 'Stopped',
    certInstalled: 'Cert Installed',
    certNotInstalled: 'No Certificate',
    proxyOn: 'Proxy On',
    proxyOff: 'Proxy Off'
  },
  accounts: {
    title: 'Account Management',
    add: 'Add Account',
    remove: 'Remove',
    switch: 'Activate',
    placeholder: 'Enter tokens (one per line)',
    autoSwitch: 'Auto Switch',
    autoSwitchDesc: 'Auto switch to next account when quota runs out',
    batchAdd: 'Batch Add'
  },
  metrics: {
    title: 'Dashboard',
    totalRequests: 'Total Requests',
    intercepted: 'Intercepted',
    authFailures: 'Auth Failures',
    rotations: 'Auto Rotations',
    interceptRate: 'Intercept Rate',
    uptime: 'Uptime',
    traffic: 'Traffic',
    reset: 'Reset'
  },
  logs: {
    title: 'Request Logs',
    refresh: 'Refresh',
    time: 'Time',
    method: 'Method',
    host: 'Host',
    path: 'Path',
    status: 'Status',
    injected: 'Injected',
    empty: 'No request logs yet'
  },
  quota: {
    title: 'Quota',
    refresh: 'Refresh Quota',
    planName: 'Plan',
    daysRemaining: 'Days Left',
    expired: 'Expired',
    expiringSoon: 'Expiring Soon',
    available: 'Available',
    noData: 'No data, click refresh to query'
  },
  domains: {
    title: 'Domain Management',
    discovered: 'Discovered Domains',
    targets: 'Intercept Targets',
    addToTarget: 'Add to Target',
    removeFromTarget: 'Remove',
    save: 'Save',
    refresh: 'Refresh'
  },
  config: {
    title: 'Settings',
    port: 'Proxy Port',
    portHint: 'Restart required after changing port',
    strategy: 'Rotation Strategy',
    strategySequential: 'Sequential',
    strategyRandom: 'Random',
    strategyPriority: 'Priority',
    strategyLeastUsed: 'Least Used',
    strategyByExpiry: 'By Expiry',
    autoRotate: 'Auto Rotate',
    autoStart: 'Start proxy on launch',
    maxRetry: 'Max Retries',
    language: 'Language',
    save: 'Save Settings'
  },
  guide: {
    title: 'Getting Started',
    step1: 'Start Proxy',
    step1Desc: 'Click "Start Proxy" to launch the local HTTPS proxy',
    step2: 'Install Certificate',
    step2Desc: 'Install the CA certificate to system trust store',
    step3: 'Add Accounts',
    step3Desc: 'Enter your Qoder tokens to add accounts',
    step4: 'Configure IDE',
    step4Desc: 'Set HTTP proxy to http://127.0.0.1:5888 in Qoder IDE',
    step5: 'Start Using',
    step5Desc: 'Use Qoder IDE normally, proxy handles authentication'
  },
  common: {
    confirm: 'Confirm',
    cancel: 'Cancel',
    save: 'Save',
    delete: 'Delete',
    refresh: 'Refresh',
    loading: 'Loading...',
    success: 'Success',
    error: 'Failed',
    version: 'Version',
    checkUpdate: 'Check Update'
  }
}
