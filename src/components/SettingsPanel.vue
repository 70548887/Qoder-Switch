<template>
  <div class="max-w-2xl">
    <h2 class="text-xl font-bold mb-6">设置</h2>
    <div class="space-y-6">
      <div>
        <h3 class="font-medium mb-2">目标域名</h3>
        <p class="text-xs text-gray-500 mb-2">仅对以下域名的请求注入认证头</p>
        <div class="bg-dark-700 rounded p-3 space-y-1 text-sm font-mono">
          <div>center.qoder.sh</div>
          <div>openapi.qoder.sh</div>
        </div>
      </div>
      <div>
        <h3 class="font-medium mb-2">代理端口</h3>
        <input value="5888" class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-32" readonly />
      </div>
      <div>
        <h3 class="font-medium mb-2">数据目录</h3>
        <div class="text-sm text-gray-400 font-mono">%APPDATA%\Qoder\</div>
      </div>
    </div>
  </div>
</template>
