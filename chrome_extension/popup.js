const openAnalyzerButton = document.getElementById('openAnalyzer');
const statusEl = document.getElementById('status');

openAnalyzerButton.addEventListener('click', () => {
  chrome.tabs.create({ url: chrome.runtime.getURL('analyze.html') });
  statusEl.textContent = 'Opening analyzer page...';
});
