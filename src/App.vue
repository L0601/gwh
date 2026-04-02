<template>
  <div class="app-shell">
    <aside class="sidebar">
      <h1>编辑接单台账</h1>
      <RouterLink class="nav-link" to="/">任务管理</RouterLink>
      <RouterLink class="nav-link" to="/contacts">对接人</RouterLink>
      <RouterLink class="nav-link" to="/stats">统计报表</RouterLink>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
