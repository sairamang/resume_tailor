const pdfInput = document.getElementById('pdfInput');
const analyzeButton = document.getElementById('analyzeButton');
const resultsContainer = document.getElementById('results');
const statusEl = document.getElementById('status');
const apiEndpointInput = document.getElementById('apiEndpoint');
const apiKeyInput = document.getElementById('apiKey');
const modelInput = document.getElementById('model');
const saveSettingsButton = document.getElementById('saveSettingsButton');
const configStatusEl = document.getElementById('configStatus');

const STORAGE_KEY = 'pdfAnalyzerApiSettings';

initializeSettings();

pdfInput.addEventListener('change', () => {
  analyzeButton.disabled = !pdfInput.files.length;
  resultsContainer.innerHTML = '';
  statusEl.textContent = '';
});

saveSettingsButton.addEventListener('click', async () => {
  const settings = getSettingsFromForm();
  const validationError = validateSettings(settings);
  if (validationError) {
    configStatusEl.textContent = validationError;
    configStatusEl.style.color = '#b91c1c';
    return;
  }

  await saveSettings(settings);
  configStatusEl.textContent = 'Settings saved.';
  configStatusEl.style.color = '#047857';
});

analyzeButton.addEventListener('click', async () => {
  if (!pdfInput.files.length) return;
  const settings = getSettingsFromForm();
  const validationError = validateSettings(settings);
  if (validationError) {
    statusEl.textContent = validationError;
    statusEl.style.color = '#b91c1c';
    return;
  }

  const file = pdfInput.files[0];
  resultsContainer.innerHTML = '';
  statusEl.textContent = 'Reading PDF...';
  statusEl.style.color = '#374151';

  try {
    const buffer = await readFileAsArrayBuffer(file);
    statusEl.textContent = 'Parsing text from PDF...';
    const text = await extractTextFromPdf(buffer);

    statusEl.textContent = 'Sending text to API...';
    const summary = await analyzeTextRemotely(text, settings);

    if (summary.error) {
      throw new Error(summary.error);
    }

    renderSummary(summary.summary);
    statusEl.textContent = 'Analysis complete.';
  } catch (error) {
    console.error(error);
    statusEl.textContent = `Error: ${error.message}`;
    statusEl.style.color = '#b91c1c';
  }
});

function getSettingsFromForm() {
  return {
    endpoint: apiEndpointInput.value.trim(),
    apiKey: apiKeyInput.value.trim(),
    model: modelInput.value.trim()
  };
}

function validateSettings(settings) {
  if (!settings.endpoint) return 'API endpoint is required.';
  if (!settings.model) return 'AI model is required.';
  if (!settings.apiKey) return 'API key is required.';
  if (!/^https?:\/\//i.test(settings.endpoint)) return 'API endpoint must be a valid URL.';
  return '';
}

function initializeSettings() {
  chrome.storage.local.get([STORAGE_KEY], (result) => {
    const settings = result[STORAGE_KEY] || {};
    apiEndpointInput.value = settings.endpoint || '';
    apiKeyInput.value = settings.apiKey || '';
    modelInput.value = settings.model || '';
  });
}

function saveSettings(settings) {
  return new Promise((resolve) => {
    const payload = {};
    payload[STORAGE_KEY] = settings;
    chrome.storage.local.set(payload, resolve);
  });
}

function readFileAsArrayBuffer(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

async function extractTextFromPdf(data) {
  if (!window.pdfjsLib || !pdfjsLib.getDocument) {
    throw new Error('PDF.js failed to load.');
  }

  pdfjsLib.GlobalWorkerOptions.workerSrc = chrome.runtime.getURL('pdf.worker.min.js');
  const loadingTask = pdfjsLib.getDocument({ data });
  const pdf = await loadingTask.promise;
  let fullText = '';

  for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
    const page = await pdf.getPage(pageNumber);
    const content = await page.getTextContent();
    const pageText = content.items.map((item) => item.str).join(' ');
    fullText += `\n\n=== Page ${pageNumber} ===\n${pageText}`;
  }

  return fullText.trim();
}

async function analyzeTextRemotely(text, settings) {
  return new Promise((resolve) => {
    chrome.runtime.sendMessage(
      {
        type: 'analyze-pdf',
        endpoint: settings.endpoint,
        apiKey: settings.apiKey,
        model: settings.model,
        text
      },
      (response) => {
        resolve(response || { error: 'No response from background worker' });
      }
    );
  });
}

function renderSummary(summary) {
  resultsContainer.innerHTML = '';

  const card = document.createElement('div');
  card.className = 'comment-card';

  const title = document.createElement('strong');
  title.textContent = 'API Summary';

  const detail = document.createElement('p');
  detail.textContent = summary;

  card.appendChild(title);
  card.appendChild(detail);
  resultsContainer.appendChild(card);
}
