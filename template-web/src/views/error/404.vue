<template>
  <div class="error-page">
    <div class="error-bg">
      <div class="bg-shape shape-1"></div>
      <div class="bg-shape shape-2"></div>
    </div>
    <div class="error-content">
      <h1 class="error-code">404</h1>
      <p class="error-desc">抱歉，您访问的页面不存在</p>
      <p class="error-sub">请检查您输入的网址是否正确，或返回首页</p>
      <div class="error-actions">
        <a-button type="primary" size="large" @click="$router.push('/')">返回首页</a-button>
        <a-button size="large" @click="$router.back()">返回上页</a-button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.error-page {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  overflow: hidden;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.error-bg {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.bg-shape {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.4;
}

.shape-1 {
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, rgba(24, 144, 255, 0.4), transparent 70%);
  top: -100px;
  right: -80px;
}

.shape-2 {
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, rgba(114, 46, 209, 0.3), transparent 70%);
  bottom: -80px;
  left: -60px;
}

.error-content {
  position: relative;
  z-index: 1;
  text-align: center;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  padding: 48px 64px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.08);
}

.error-code {
  font-size: 96px;
  font-weight: 800;
  line-height: 1;
  margin: 0 0 16px;
  background: linear-gradient(135deg, #1890ff, #722ed1);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.error-desc {
  font-size: 20px;
  color: rgba(0, 0, 0, 0.65);
  margin: 0 0 8px;
  font-weight: 500;
}

.error-sub {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.45);
  margin: 0 0 32px;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}
</style>
