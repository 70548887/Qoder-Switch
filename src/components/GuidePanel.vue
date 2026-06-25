<template>
  <div class="max-w-2xl">
    <h2 class="text-xl font-bold mb-6">基本流程</h2>
    <div class="space-y-4">
      <Step :num="1" title="安装证书" desc="点击【安装证书】→ 右上角显示绿色「已安装」即可" />
      <Step :num="2" title="添加账号" desc="粘贴密钥到输入框 → 点击【添加】（多个用空格或换行分隔）" />
      <Step :num="3" title="启动代理" desc="点击【启动代理】→ 按钮变红即成功（默认端口 5888）" />
      <Step :num="4" title="接入账号" desc="在列表中点击要使用的账号的【接入】→ 状态变为「当前」" />
      <Step :num="5" title="配置 Qoder IDE" desc="设置 → 高级 → 网络代理 → 代理地址: http://127.0.0.1:5888" />
    </div>
    <div class="mt-6 p-4 bg-red-500/10 border border-red-500/30 rounded text-sm text-red-300">
      注意：必须通过代理方式启动才能生效，直接双击打开不会走代理。
    </div>
  </div>
</template>

<script setup lang="ts">
import Step from './Step.vue'
</script>
